use super::TYPE_HINT_NICHE;
use crate::{io::NbtIoError, raw};
use serde::{
    de::{
        self,
        value::CowStrDeserializer,
        DeserializeSeed,
        EnumAccess,
        IntoDeserializer,
        MapAccess,
        SeqAccess,
        VariantAccess,
        Visitor,
    },
    forward_to_deserialize_any,
};
use std::{
    borrow::Cow,
    convert::TryFrom,
    error::Error,
    fmt::{self, Display, Formatter},
    io::{self, Cursor, ErrorKind, Read},
    marker::PhantomData,
};

/// The deserializer type for reading binary NBT data.
pub struct Deserializer<'a, R, B> {
    reader: &'a mut R,
    _buffered: PhantomData<B>,
}

impl<'a, R: Read> Deserializer<'a, R, Unbuffered> {
    /// Attempts to construct a new deserializer with the given reader. If the data in the reader
    /// does not start with a valid compound tag, an error is returned. Otherwise, the root name
    /// is returned along with the deserializer.
    pub fn new(reader: &'a mut R) -> Result<(Self, String), NbtIoError> {
        if raw::read_u8(reader)? != 0xA {
            return Err(NbtIoError::MissingRootTag);
        }

        let root_name = raw::read_string(reader)?;
        Ok((
            Deserializer {
                reader,
                _buffered: PhantomData,
            },
            root_name,
        ))
    }
}

impl<'a, 'buffer> Deserializer<'a, Cursor<&'buffer [u8]>, BufferedCursor<'buffer>>
where
// This is just here explicitly to clarify what's going on. In reality, having a reference to
// a cursor in and of itself is a certificate of this constraint 'buffer: 'a
{
    /// Similar to [`new`], however constructing a deserializer with this method will allow for
    /// data to be borrowed during deserialization.
    ///
    /// If the data in the reader does not start with a valid compound tag, an error is returned.
    /// Otherwise, the root name is returned along with the deserializer.
    ///
    /// [`new`]: crate::serde::Deserializer::new
    pub fn from_cursor(
        reader: &'a mut Cursor<&'buffer [u8]>,
    ) -> Result<(Self, Cow<'buffer, str>), NbtIoError> {
        if raw::read_u8(reader)? != 0xA {
            return Err(NbtIoError::MissingRootTag);
        }

        let root_name_len = raw::read_u16(reader)? as usize;
        let bytes = read_bytes_from_cursor(reader, root_name_len)?;

        let root_name = match cesu8::from_java_cesu8(bytes) {
            Ok(string) => string,
            Err(_) => return Err(NbtIoError::InvalidCesu8String),
        };

        Ok((
            Deserializer {
                reader,
                _buffered: PhantomData,
            },
            root_name,
        ))
    }
}

impl<'de, 'a, 'buffer, R, B> de::Deserializer<'de> for Deserializer<'a, R, B>
where
    'de: 'a,
    'buffer: 'de,
    R: Read,
    B: BufferSpecialization<'buffer>,
{
    type Error = NbtIoError;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct identifier ignored_any
    }

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        DeserializeTag::<_, B, 0xA>::new(self.reader).deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        DeserializeTag::<_, B, 0xA>::new(self.reader).deserialize_enum(name, variants, visitor)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

#[inline]
fn drive_visitor_seq_const<'de, 'a, 'buffer, R, V, B, const TAG_ID: u8>(
    reader: &'a mut R,
    visitor: V,
) -> Result<V::Value, NbtIoError>
where
    'de: 'a,
    'buffer: 'de,
    R: Read,
    V: Visitor<'de>,
    B: BufferSpecialization<'buffer>,
{
    match TAG_ID {
        0x7 => {
            let len = raw::read_i32(reader)? as usize;
            visitor.visit_seq(DeserializeSeq::<_, _, 0x1, TAG_ID>::new(
                DeserializeTag::<_, B, 0x1>::new(reader),
                len,
            ))
        }
        0x9 => drive_visitor_seq_tag::<_, _, B>(reader, visitor),
        0xB => {
            let len = raw::read_i32(reader)? as usize;
            visitor.visit_seq(DeserializeSeq::<_, _, 0x3, TAG_ID>::new(
                DeserializeTag::<_, B, 0x3>::new(reader),
                len,
            ))
        }
        0xC => {
            let len = raw::read_i32(reader)? as usize;
            visitor.visit_seq(DeserializeSeq::<_, _, 0x4, TAG_ID>::new(
                DeserializeTag::<_, B, 0x4>::new(reader),
                len,
            ))
        }
        _ => Err(NbtIoError::ExpectedSeq),
    }
}

fn drive_visitor_seq_tag<'de, 'a, 'buffer, R, V, B>(
    reader: &'a mut R,
    visitor: V,
) -> Result<V::Value, NbtIoError>
where
    'de: 'a,
    'buffer: 'de,
    R: Read,
    V: Visitor<'de>,
    B: BufferSpecialization<'buffer>,
{
    let id = raw::read_u8(reader)?;
    let len = raw::read_i32(reader)? as usize;

    macro_rules! drive_visitor {
        ($($id:literal)*) => {
            match id {
                0x0 => {
                    if len == 0 {
                        visitor.visit_seq(DeserializeSeq::<_, _, 0x0, 0x9>::new(DeserializeTag::<_, B, 0x0>::new(reader), len))
                    } else {
                        Err(NbtIoError::InvalidTagId(0))
                    }
                }
                $( $id => visitor.visit_seq(DeserializeSeq::<_, _, $id, 0x9>::new(DeserializeTag::<_, B, $id>::new(reader), len)), )*
                _ => Err(NbtIoError::InvalidTagId(id))
            }
        };
    }

    drive_visitor!(0x1 0x2 0x3 0x4 0x5 0x6 0x7 0x8 0x9 0xA 0xB 0xC)
}

struct DeserializeEnum<'a, R, B, const TAG_ID: u8> {
    reader: &'a mut R,
    variant: Cow<'a, str>,
    _buffered: PhantomData<B>,
}

impl<'a, R, B, const TAG_ID: u8> DeserializeEnum<'a, R, B, TAG_ID> {
    #[inline]
    fn new(reader: &'a mut R, variant: Cow<'a, str>) -> Self {
        DeserializeEnum {
            reader,
            variant,
            _buffered: PhantomData,
        }
    }
}

impl<'de, 'a, 'buffer, R, B, const TAG_ID: u8> EnumAccess<'de> for DeserializeEnum<'a, R, B, TAG_ID>
where
    'de: 'a,
    'buffer: 'de,
    R: Read,
    B: BufferSpecialization<'buffer>,
{
    type Error = NbtIoError;
    type Variant = DeserializeVariant<'a, R, B, TAG_ID>;

    #[inline]
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where V: DeserializeSeed<'de> {
        let de: CowStrDeserializer<'a, Self::Error> = self.variant.into_deserializer();
        Ok((seed.deserialize(de)?, DeserializeVariant::new(self.reader)))
    }
}

struct DeserializeVariant<'a, R, B, const TAG_ID: u8> {
    reader: &'a mut R,
    _buffered: PhantomData<B>,
}

impl<'a, 'buffer, R, B, const TAG_ID: u8> DeserializeVariant<'a, R, B, TAG_ID>
where
    R: Read,
    B: BufferSpecialization<'buffer>,
{
    #[inline]
    fn new(reader: &'a mut R) -> Self {
        DeserializeVariant {
            reader,
            _buffered: PhantomData,
        }
    }
}

impl<'de, 'a, 'buffer, R, B, const TAG_ID: u8> VariantAccess<'de>
    for DeserializeVariant<'a, R, B, TAG_ID>
where
    'de: 'a,
    'buffer: 'de,
    R: Read,
    B: BufferSpecialization<'buffer>,
{
    type Error = NbtIoError;

    #[cold]
    fn unit_variant(self) -> Result<(), Self::Error> {
        unimplemented!("Unit variant should have been handled by deserialize_enum")
    }

    #[inline]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where T: DeserializeSeed<'de> {
        seed.deserialize(&mut DeserializeTag::<_, B, TAG_ID>::new(self.reader))
    }

    #[inline]
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        drive_visitor_seq_const::<_, _, B, TAG_ID>(self.reader, visitor)
    }

    #[inline]
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if TAG_ID == 0xA {
            visitor.visit_map(DeserializeMap::<_, B>::new(self.reader))
        } else {
            Err(NbtIoError::TagTypeMismatch {
                expected: 0xA,
                found: TAG_ID,
            })
        }
    }
}

struct DeserializeSeq<'a, R, B, const TAG_ID: u8, const LIST_ID: u8> {
    inner: DeserializeTag<'a, R, B, TAG_ID>,
    remaining: usize,
    dispatch_state: TypeHintDispatchState,
    _buffered: PhantomData<B>,
}

impl<'a, 'buffer, R, B, const TAG_ID: u8, const LIST_ID: u8>
    DeserializeSeq<'a, R, B, TAG_ID, LIST_ID>
where
    R: Read,
    B: BufferSpecialization<'buffer>,
{
    #[inline]
    fn new(inner: DeserializeTag<'a, R, B, TAG_ID>, len: usize) -> Self {
        DeserializeSeq {
            inner,
            remaining: len,
            dispatch_state: TypeHintDispatchState::Waiting,
            _buffered: PhantomData,
        }
    }
}

impl<'de, 'a, 'buffer, R, B, const TAG_ID: u8, const LIST_ID: u8> SeqAccess<'de>
    for DeserializeSeq<'a, R, B, TAG_ID, LIST_ID>
where
    'de: 'a,
    'buffer: 'de,
    R: Read,
    B: BufferSpecialization<'buffer>,
{
    type Error = NbtIoError;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where T: de::DeserializeSeed<'de> {
        // This will optimize out
        if TAG_ID == 0 {
            return Ok(None);
        }

        // This is necessary for the LLVM to consider inlining the outer function
        #[inline(never)]
        fn handle_hint_dispatch<'de, T, const LIST_ID: u8>(
            state: &mut TypeHintDispatchState,
            seed: T,
        ) -> Result<Option<T::Value>, NbtIoError>
        where
            T: de::DeserializeSeed<'de>,
        {
            match state {
                TypeHintDispatchState::Ready => {
                    *state = TypeHintDispatchState::Sent;
                    return Ok(seed.deserialize(TypeHintDeserializer::<LIST_ID>).ok());
                }
                TypeHintDispatchState::Sent => return Ok(None),
                _ => unreachable!(),
            }
        }

        if self.remaining == 0 {
            // If this method gets called again, we'll deserialize a type hint
            if self.dispatch_state == TypeHintDispatchState::Waiting {
                self.dispatch_state = TypeHintDispatchState::Ready;
                return Ok(None);
            } else {
                return handle_hint_dispatch::<_, LIST_ID>(&mut self.dispatch_state, seed);
            }
        }

        self.remaining -= 1;
        seed.deserialize(&mut self.inner).map(Some)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        if TAG_ID == 0 {
            Some(0)
        } else {
            Some(self.remaining)
        }
    }
}

#[derive(PartialEq, Eq)]
enum TypeHintDispatchState {
    Waiting,
    Ready,
    Sent,
}

struct DeserializeMap<'a, R, B> {
    reader: &'a mut R,
    tag_id: u8,
    _buffered: PhantomData<B>,
}

impl<'a, 'buffer, R, B> DeserializeMap<'a, R, B>
where
    R: Read,
    B: BufferSpecialization<'buffer>,
{
    #[inline]
    fn new(reader: &'a mut R) -> Self {
        DeserializeMap {
            reader,
            tag_id: 0,
            _buffered: PhantomData,
        }
    }

    fn drive_value_visitor<'de, V>(
        &mut self,
        tag_id: u8,
        seed: V,
    ) -> Result<V::Value, <Self as MapAccess<'de>>::Error>
    where
        'de: 'a,
        'buffer: 'de,
        V: DeserializeSeed<'de>,
    {
        macro_rules! drive_visitor {
            ($($id:literal)*) => {
                match tag_id {
                    $( $id => seed.deserialize(&mut DeserializeTag::<_, B, $id>::new(self.reader)), )*
                    _ => Err(NbtIoError::InvalidTagId(tag_id))
                }
            };
        }

        drive_visitor!(0x1 0x2 0x3 0x4 0x5 0x6 0x7 0x8 0x9 0xA 0xB 0xC)
    }
}

impl<'de, 'a, 'buffer, R, B> MapAccess<'de> for DeserializeMap<'a, R, B>
where
    'de: 'a,
    'buffer: 'de,
    R: Read,
    B: BufferSpecialization<'buffer>,
{
    type Error = NbtIoError;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where K: de::DeserializeSeed<'de> {
        self.tag_id = raw::read_u8(self.reader)?;

        if self.tag_id == 0 {
            return Ok(None);
        }

        let mut de = DeserializeTag::<_, B, 0x8>::new(self.reader);
        seed.deserialize(&mut de).map(Some)
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where V: de::DeserializeSeed<'de> {
        self.drive_value_visitor(self.tag_id, seed)
    }
}

pub struct DeserializeTag<'a, R, B, const TAG_ID: u8> {
    reader: &'a mut R,
    _buffered: PhantomData<B>,
}

impl<'a, 'buffer, R, B, const TAG_ID: u8> DeserializeTag<'a, R, B, TAG_ID>
where
    R: Read,
    B: BufferSpecialization<'buffer>,
{
    #[inline]
    fn new(reader: &'a mut R) -> DeserializeTag<'a, R, B, TAG_ID> {
        DeserializeTag {
            reader,
            _buffered: PhantomData,
        }
    }
}

impl<'de, 'a, 'buffer, 'b, R, B, const TAG_ID: u8> de::Deserializer<'de>
    for &'b mut DeserializeTag<'a, R, B, TAG_ID>
where
    'de: 'a,
    'buffer: 'de,
    R: Read,
    B: BufferSpecialization<'buffer>,
{
    type Error = NbtIoError;

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u16 u32 u64 u128 char f32 f64 string
    }

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        match TAG_ID {
            0x1 => visitor.visit_i8(raw::read_i8(self.reader)?),
            0x2 => visitor.visit_i16(raw::read_i16(self.reader)?),
            0x3 => visitor.visit_i32(raw::read_i32(self.reader)?),
            0x4 => visitor.visit_i64(raw::read_i64(self.reader)?),
            0x5 => visitor.visit_f32(raw::read_f32(self.reader)?),
            0x6 => visitor.visit_f64(raw::read_f64(self.reader)?),
            0x7 => {
                let len = raw::read_i32(self.reader)? as usize;
                visitor.visit_seq(DeserializeSeq::<_, _, 0x1, 0x7>::new(
                    DeserializeTag::<_, B, 0x1>::new(self.reader),
                    len,
                ))
            }
            0x8 => visitor.visit_string(raw::read_string(self.reader)?),
            0x9 => drive_visitor_seq_tag::<_, _, B>(self.reader, visitor),
            0xA => visitor.visit_map(DeserializeMap::<_, B>::new(self.reader)),
            0xB => {
                let len = raw::read_i32(self.reader)? as usize;
                visitor.visit_seq(DeserializeSeq::<_, _, 0x3, 0xB>::new(
                    DeserializeTag::<_, B, 0x3>::new(self.reader),
                    len,
                ))
            }
            0xC => {
                let len = raw::read_i32(self.reader)? as usize;
                visitor.visit_seq(DeserializeSeq::<_, _, 0x4, 0xC>::new(
                    DeserializeTag::<_, B, 0x4>::new(self.reader),
                    len,
                ))
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        if TAG_ID == 0x1 {
            visitor.visit_bool(raw::read_bool(self.reader)?)
        } else {
            self.deserialize_any(visitor)
        }
    }

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        if TAG_ID == 0x1 {
            visitor.visit_u8(raw::read_u8(self.reader)?)
        } else {
            self.deserialize_any(visitor)
        }
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        if TAG_ID == 0x7 {
            let len = raw::read_i32(self.reader)? as usize;
            let mut array = vec![0u8; len];
            self.reader.read_exact(&mut array)?;
            visitor.visit_byte_buf(array)
        } else {
            self.deserialize_any(visitor)
        }
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        if TAG_ID == 0x7 {
            let len = raw::read_i32(self.reader)? as usize;

            if B::BUFFERED {
                // Safety: R is `&'a mut Cursor<&'buffer [u8]>` and `B` is
                // `BufferedCursor<'buffer>` by the constructor `Deserializer::from_cursor`
                visitor.visit_borrowed_bytes(unsafe { B::read_bytes(self.reader, len) }?)
            } else {
                let mut array = vec![0u8; len];
                self.reader.read_exact(&mut array)?;
                visitor.visit_bytes(&array)
            }
        } else {
            self.deserialize_any(visitor)
        }
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        if TAG_ID == 0x8 {
            if B::BUFFERED {
                let len = raw::read_u16(self.reader)? as usize;
                // Safety: R is `&'a mut Cursor<&'buffer [u8]>` and `B` is
                // `BufferedCursor<'buffer>` by the constructor `Deserializer::from_cursor`
                let bytes: &'de [u8] = unsafe { B::read_bytes(self.reader, len) }?;

                let string = match cesu8::from_java_cesu8(bytes) {
                    Ok(string) => string,
                    Err(_) => return Err(NbtIoError::InvalidCesu8String),
                };

                match string {
                    Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
                    Cow::Owned(string) => visitor.visit_string(string),
                }
            } else {
                let mut dest = Vec::new();
                match raw::read_string_into(self.reader, &mut dest)? {
                    Cow::Borrowed(string) => visitor.visit_str(string),
                    Cow::Owned(string) => visitor.visit_string(string),
                }
            }
        } else {
            self.deserialize_any(visitor)
        }
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        drive_visitor_seq_const::<_, _, B, TAG_ID>(self.reader, visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        visitor.visit_map(DeserializeMap::<_, B>::new(self.reader))
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match TAG_ID {
            // Unit variant
            0x1 => visitor.visit_enum(
                variants
                    .get(raw::read_i8(self.reader)? as usize)
                    .ok_or(NbtIoError::InvalidEnumVariant)?
                    .into_deserializer(),
            ),
            0x2 => visitor.visit_enum(
                variants
                    .get(raw::read_i16(self.reader)? as usize)
                    .ok_or(NbtIoError::InvalidEnumVariant)?
                    .into_deserializer(),
            ),
            0x3 => visitor.visit_enum(
                variants
                    .get(raw::read_i32(self.reader)? as usize)
                    .ok_or(NbtIoError::InvalidEnumVariant)?
                    .into_deserializer(),
            ),
            0x8 => {
                let mut dest = Vec::new();
                visitor
                    .visit_enum(raw::read_string_into(self.reader, &mut dest)?.into_deserializer())
            }
            // Newtype, tuple, and struct variants
            0xA => {
                let id = raw::read_u8(self.reader)?;
                let mut buf = Vec::new();
                let variant = raw::read_string_into(self.reader, &mut buf)?;

                macro_rules! drive_visitor {
                    ($($id:literal)*) => {
                        match id {
                            $( $id => visitor.visit_enum(DeserializeEnum::<_, B, $id>::new(self.reader, variant)), )*
                            _ => Err(NbtIoError::InvalidTagId(id))
                        }
                    };
                }

                let result = drive_visitor!(0x1 0x2 0x3 0x4 0x5 0x6 0x7 0x8 0x9 0xA 0xB 0xC)?;

                let end = raw::read_u8(self.reader)?;
                if end != 0x0 {
                    return Err(NbtIoError::TagTypeMismatch {
                        expected: 0x0,
                        found: end,
                    });
                }

                Ok(result)
            }
            _ => Err(NbtIoError::ExpectedEnum),
        }
    }

    #[inline]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }
}

struct TypeHintDeserializer<const TAG_ID: u8>;

impl<'de, const TAG_ID: u8> de::Deserializer<'de> for TypeHintDeserializer<TAG_ID> {
    type Error = TypeHintDeserializerError;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(TypeHintDeserializerError)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if name == TYPE_HINT_NICHE {
            visitor.visit_u8(TAG_ID)
        } else {
            Err(TypeHintDeserializerError)
        }
    }
}

#[derive(Debug)]
pub struct TypeHintDeserializerError;

impl Display for TypeHintDeserializerError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Error for TypeHintDeserializerError {}

impl de::Error for TypeHintDeserializerError {
    fn custom<T>(_msg: T) -> Self
    where T: Display {
        TypeHintDeserializerError
    }
}

// A trait to implement specialization - sort of.
pub unsafe trait BufferSpecialization<'buffer> {
    const BUFFERED: bool;

    unsafe fn read_bytes<'de, R>(_reader: &mut R, _len: usize) -> Result<&'de [u8], io::Error>
    where 'buffer: 'de {
        panic!("read_bytes called on a non-buffered reader")
    }
}

pub struct Unbuffered;

unsafe impl BufferSpecialization<'static> for Unbuffered {
    const BUFFERED: bool = false;
}

pub struct BufferedCursor<'buffer> {
    // We are essentially a function which takes a slice and returns a sub-slice, so we need to
    // act like that
    _phantom: PhantomData<fn(&'buffer [u8])>,
}

unsafe impl<'buffer> BufferSpecialization<'buffer> for BufferedCursor<'buffer> {
    const BUFFERED: bool = true;

    /// Extracts a reference to a slice of bytes out of the given reader.
    ///
    /// # Safety
    ///
    /// The caller must assert that `R` is `Cursor<&'buffer [u8]>`, otherwise unconscionable
    /// amounts of UB will ensue.
    unsafe fn read_bytes<'de, R>(reader: &mut R, len: usize) -> Result<&'de [u8], io::Error>
    where 'buffer: 'de {
        let cursor: &mut Cursor<&'buffer [u8]> = &mut *(reader as *mut R as *mut _);
        read_bytes_from_cursor(cursor, len)
    }
}

fn read_bytes_from_cursor<'de, 'a: 'de>(
    cursor: &mut Cursor<&'a [u8]>,
    len: usize,
) -> Result<&'de [u8], io::Error> {
    let position = cursor.position() as usize;
    let total_len = cursor.get_ref().len();
    let remaining = total_len.checked_sub(position).unwrap_or(0);

    if len > remaining {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            format!(
                "Read of {} bytes requested but only {} remain",
                len, remaining
            ),
        ));
    }

    cursor.set_position(u64::try_from(position + len).expect("Cursor position overflowed"));

    let inner: &'a [u8] = cursor.get_ref();
    Ok(&inner[position .. position + len])
}
