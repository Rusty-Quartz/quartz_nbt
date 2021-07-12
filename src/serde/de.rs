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
use std::{borrow::Cow, io::Read};

/// The deserializer type for reading binary NBT data.
pub struct Deserializer<'a, R> {
    reader: &'a mut R,
}

impl<'a, R: Read> Deserializer<'a, R> {
    /// Attempts to construct a new deserializer with the given reader. If the data in the reader
    /// does not start with a valid compound tag, an error is returned. Otherwise, the root name
    /// is returned along with the deserializer.
    pub fn new(reader: &'a mut R) -> Result<(Self, String), NbtIoError> {
        if raw::read_u8(reader)? != 0xA {
            return Err(NbtIoError::MissingRootTag);
        }

        let root_name = raw::read_string(reader)?;
        Ok((Deserializer { reader }, root_name))
    }
}

impl<'de: 'a, 'a, R: Read> de::Deserializer<'de> for Deserializer<'a, R> {
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
        DeserializeTag::<_, 0xA>::new(self.reader).deserialize_map(visitor)
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
        DeserializeTag::<_, 0xA>::new(self.reader).deserialize_enum(name, variants, visitor)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

#[inline]
fn drive_visitor_seq_const<'de, R, V, const TAG_ID: u8>(
    reader: &mut R,
    visitor: V,
) -> Result<V::Value, NbtIoError>
where
    R: Read,
    V: Visitor<'de>,
{
    match TAG_ID {
        0x7 => {
            let len = raw::read_i32(reader)? as usize;
            visitor.visit_seq(DeserializeSeq::new(
                DeserializeTag::<_, 0x1>::new(reader),
                len,
            ))
        }
        0x9 => drive_visitor_seq_tag(reader, visitor),
        0xB => {
            let len = raw::read_i32(reader)? as usize;
            visitor.visit_seq(DeserializeSeq::new(
                DeserializeTag::<_, 0x3>::new(reader),
                len,
            ))
        }
        0xC => {
            let len = raw::read_i32(reader)? as usize;
            visitor.visit_seq(DeserializeSeq::new(
                DeserializeTag::<_, 0x4>::new(reader),
                len,
            ))
        }
        _ => Err(NbtIoError::ExpectedSeq),
    }
}

fn drive_visitor_seq_tag<'de, R, V>(reader: &mut R, visitor: V) -> Result<V::Value, NbtIoError>
where
    R: Read,
    V: Visitor<'de>,
{
    let id = raw::read_u8(reader)?;
    let len = raw::read_i32(reader)? as usize;

    macro_rules! drive_visitor {
        ($($id:literal)*) => {
            match id {
                0x0 => {
                    if len == 0 {
                        visitor.visit_seq(DeserializeSeq::new(DeserializeTag::<_, 0x0>::new(reader), len))
                    } else {
                        Err(NbtIoError::InvalidTagId(0))
                    }
                }
                $( $id => visitor.visit_seq(DeserializeSeq::new(DeserializeTag::<_, $id>::new(reader), len)), )*
                _ => Err(NbtIoError::InvalidTagId(id))
            }
        };
    }

    drive_visitor!(0x1 0x2 0x3 0x4 0x5 0x6 0x7 0x8 0x9 0xA 0xB 0xC)
}

pub struct DeserializeTag<'a, R, const TAG_ID: u8> {
    reader: &'a mut R,
}

impl<'a, R: Read, const TAG_ID: u8> DeserializeTag<'a, R, TAG_ID> {
    fn new(reader: &'a mut R) -> DeserializeTag<'a, R, TAG_ID> {
        DeserializeTag { reader }
    }
}

impl<'de, 'a, 'b, R: Read, const TAG_ID: u8> de::Deserializer<'de>
    for &'b mut DeserializeTag<'a, R, TAG_ID>
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
                visitor.visit_seq(DeserializeSeq::new(
                    DeserializeTag::<_, 0x1>::new(self.reader),
                    len,
                ))
            }
            0x8 => visitor.visit_string(raw::read_string(self.reader)?),
            0x9 => drive_visitor_seq_tag(self.reader, visitor),
            0xA => visitor.visit_map(DeserializeMap::new(self.reader)),
            0xB => {
                let len = raw::read_i32(self.reader)? as usize;
                visitor.visit_seq(DeserializeSeq::new(
                    DeserializeTag::<_, 0x3>::new(self.reader),
                    len,
                ))
            }
            0xC => {
                let len = raw::read_i32(self.reader)? as usize;
                visitor.visit_seq(DeserializeSeq::new(
                    DeserializeTag::<_, 0x4>::new(self.reader),
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
            let mut array = vec![0u8; len];
            self.reader.read_exact(&mut array)?;
            visitor.visit_bytes(&array)
        } else {
            self.deserialize_any(visitor)
        }
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        if TAG_ID == 0x8 {
            let mut dest = Vec::new();
            match raw::read_string_into(self.reader, &mut dest)? {
                Cow::Borrowed(string) => visitor.visit_str(string),
                Cow::Owned(string) => visitor.visit_string(string),
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
        drive_visitor_seq_const::<_, _, TAG_ID>(self.reader, visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        visitor.visit_map(DeserializeMap::new(self.reader))
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
            // Newtype, tuple, and struct variants
            0xA => {
                let id = raw::read_u8(self.reader)?;
                let mut buf = Vec::new();
                let variant = raw::read_string_into(self.reader, &mut buf)?;

                macro_rules! drive_visitor {
                    ($($id:literal)*) => {
                        match id {
                            $( $id => visitor.visit_enum(DeserializeEnum::<_, $id>::new(self.reader, variant)), )*
                            _ => Err(NbtIoError::InvalidTagId(id))
                        }
                    };
                }

                let result = drive_visitor!(0x1 0x2 0x3 0x4 0x5 0x6 0x7 0x8 0x9 0xA 0xB 0xC)?;

                let end = raw::read_u8(self.reader)?;
                if end != 0x0 {
                    return Err(NbtIoError::TagTypeMismatch(0x0, end));
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

struct DeserializeEnum<'a, R, const TAG_ID: u8> {
    reader: &'a mut R,
    variant: Cow<'a, str>,
}

impl<'a, R, const TAG_ID: u8> DeserializeEnum<'a, R, TAG_ID> {
    #[inline]
    fn new(reader: &'a mut R, variant: Cow<'a, str>) -> Self {
        DeserializeEnum { reader, variant }
    }
}

impl<'de, 'a, R: Read, const TAG_ID: u8> EnumAccess<'de> for DeserializeEnum<'a, R, TAG_ID> {
    type Error = NbtIoError;
    type Variant = DeserializeVariant<'a, R, TAG_ID>;

    #[inline]
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where V: DeserializeSeed<'de> {
        let de: CowStrDeserializer<'a, Self::Error> = self.variant.into_deserializer();
        Ok((seed.deserialize(de)?, DeserializeVariant::new(self.reader)))
    }
}

struct DeserializeVariant<'a, R, const TAG_ID: u8> {
    reader: &'a mut R,
}

impl<'a, R, const TAG_ID: u8> DeserializeVariant<'a, R, TAG_ID> {
    #[inline]
    fn new(reader: &'a mut R) -> Self {
        DeserializeVariant { reader }
    }
}

impl<'de, 'a, R: Read, const TAG_ID: u8> VariantAccess<'de> for DeserializeVariant<'a, R, TAG_ID> {
    type Error = NbtIoError;

    #[cold]
    fn unit_variant(self) -> Result<(), Self::Error> {
        unimplemented!("Unit variant should have been handled by deserialize_enum")
    }

    #[inline]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where T: DeserializeSeed<'de> {
        seed.deserialize(&mut DeserializeTag::<_, TAG_ID>::new(self.reader))
    }

    #[inline]
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        drive_visitor_seq_const::<_, _, TAG_ID>(self.reader, visitor)
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
            visitor.visit_map(DeserializeMap::new(self.reader))
        } else {
            Err(NbtIoError::TagTypeMismatch(0xA, TAG_ID))
        }
    }
}

struct DeserializeSeq<'a, R, const TAG_ID: u8> {
    inner: DeserializeTag<'a, R, TAG_ID>,
    remaining: usize,
}

impl<'a, R: Read, const TAG_ID: u8> DeserializeSeq<'a, R, TAG_ID> {
    #[inline]
    fn new(inner: DeserializeTag<'a, R, TAG_ID>, len: usize) -> Self {
        DeserializeSeq {
            inner,
            remaining: len,
        }
    }
}

impl<'de, 'a, R: Read, const TAG_ID: u8> SeqAccess<'de> for DeserializeSeq<'a, R, TAG_ID> {
    type Error = NbtIoError;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where T: de::DeserializeSeed<'de> {
        if TAG_ID == 0 || self.remaining == 0 {
            return Ok(None);
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

struct DeserializeMap<'a, R> {
    reader: &'a mut R,
    tag_id: u8,
}

impl<'a, R: Read> DeserializeMap<'a, R> {
    #[inline]
    fn new(reader: &'a mut R) -> Self {
        DeserializeMap { reader, tag_id: 0 }
    }

    fn drive_value_visitor<'de, V>(
        &mut self,
        tag_id: u8,
        seed: V,
    ) -> Result<V::Value, <Self as MapAccess<'de>>::Error>
    where
        V: DeserializeSeed<'de>,
    {
        macro_rules! drive_visitor {
            ($($id:literal)*) => {
                match tag_id {
                    $( $id => seed.deserialize(&mut DeserializeTag::<_, $id>::new(self.reader)), )*
                    _ => Err(NbtIoError::InvalidTagId(tag_id))
                }
            };
        }

        drive_visitor!(0x1 0x2 0x3 0x4 0x5 0x6 0x7 0x8 0x9 0xA 0xB 0xC)
    }
}

impl<'de, 'a, R: Read> MapAccess<'de> for DeserializeMap<'a, R> {
    type Error = NbtIoError;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where K: de::DeserializeSeed<'de> {
        self.tag_id = raw::read_u8(self.reader)?;

        if self.tag_id == 0 {
            return Ok(None);
        }

        let mut buf = Vec::new();
        let de: CowStrDeserializer<'_, Self::Error> =
            raw::read_string_into(self.reader, &mut buf)?.into_deserializer();
        seed.deserialize(de).map(Some)
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where V: de::DeserializeSeed<'de> {
        self.drive_value_visitor(self.tag_id, seed)
    }

    #[inline]
    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
        V: de::DeserializeSeed<'de>,
    {
        let tag_id = raw::read_u8(self.reader)?;

        if tag_id == 0 {
            return Ok(None);
        }

        let mut buf = Vec::new();
        let de: CowStrDeserializer<'_, Self::Error> =
            raw::read_string_into(self.reader, &mut buf)?.into_deserializer();
        let key = kseed.deserialize(de)?;
        Ok(Some((key, self.drive_value_visitor(tag_id, vseed)?)))
    }
}
