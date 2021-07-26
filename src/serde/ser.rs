use super::{
    array::{BYTE_ARRAY_NICHE, INT_ARRAY_NICHE, LONG_ARRAY_NICHE},
    util::{DefaultSerializer, Ser},
};
use crate::{io::NbtIoError, raw};
use serde::{
    ser::{
        Impossible,
        SerializeMap,
        SerializeSeq,
        SerializeStruct,
        SerializeStructVariant,
        SerializeTuple,
        SerializeTupleStruct,
        SerializeTupleVariant,
    },
    Serialize,
};
use std::{
    cell::Cell,
    io::{Cursor, Write},
    marker::PhantomData,
};

/// The serializer type for writing binary NBT data.
pub type Serializer<'a, W> = Ser<SerializerImpl<'a, W, Homogenous>>;

/// An alternative serializer type for writing binary NBT data which elides checks for
/// sequence homogeneity. Using this type could result in bogus NBT data.
pub type UncheckedSerializer<'a, W> = Ser<SerializerImpl<'a, W, Unchecked>>;

impl<'a, W: Write> Serializer<'a, W> {
    /// Constructs a new serializer with the given writer and root name. If no root name is specified,
    /// then an empty string is written to the header.
    pub fn new(writer: &'a mut W, root_name: Option<&'a str>) -> Self {
        SerializerImpl::new(writer, BorrowedPrefix::new(root_name.unwrap_or(""))).into_serializer()
    }
}

impl<'a, W: Write> UncheckedSerializer<'a, W> {
    /// Constructs a new unchecked serializer with the given writer and root name, If no root name is
    /// specified then an empty string is written to the header.
    pub fn new(writer: &'a mut W, root_name: Option<&'a str>) -> Self {
        SerializerImpl::new(writer, BorrowedPrefix::new(root_name.unwrap_or(""))).into_serializer()
    }
}

pub struct SerializerImpl<'a, W, C> {
    writer: &'a mut W,
    root_name: BorrowedPrefix<&'a str>,
    _phantom: PhantomData<C>,
}

impl<'a, W: Write, C: TypeChecker> SerializerImpl<'a, W, C> {
    fn new(writer: &'a mut W, root_name: BorrowedPrefix<&'a str>) -> Self {
        SerializerImpl {
            writer,
            root_name,
            _phantom: PhantomData,
        }
    }
}

impl<'a, W: Write, C: TypeChecker> DefaultSerializer for SerializerImpl<'a, W, C> {
    type Error = NbtIoError;
    type Ok = ();
    type SerializeMap = SerializeCompound<'a, W, C>;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = SerializeCompound<'a, W, C>;
    type SerializeStructVariant = SerializeCompound<'a, W, C>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = SerializeList<'a, W, C>;

    #[cold]
    fn unimplemented(self, _ty: &'static str) -> Self::Error {
        NbtIoError::MissingRootTag
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let mut map = self.serialize_map(Some(1))?;
        map.serialize_entry(variant, value)?;
        SerializeMap::end(map)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.root_name.write(self.writer, 0xA)?;
        let prefix = BorrowedPrefix::new(variant);
        SerializeCompoundEntry::new(self.writer, prefix).serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.root_name.write(self.writer, 0xA)?;
        Ok(SerializeCompound::new(self.writer))
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        <Self as DefaultSerializer>::serialize_map(self, Some(len))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.root_name.write(self.writer, 0xA)?;
        raw::write_u8(self.writer, 0xA)?;
        raw::write_string(self.writer, variant)?;
        // The extra closing tag is added by the SerializeStructVariant impl
        Ok(SerializeCompound::new(self.writer))
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct SerializeArray<'a, W> {
    writer: &'a mut W,
}

impl<'a, W> SerializeArray<'a, W>
where W: Write
{
    #[inline]
    fn new(writer: &'a mut W) -> Self {
        SerializeArray { writer }
    }
}

impl<'a, W> DefaultSerializer for SerializeArray<'a, W>
where W: Write
{
    type Error = NbtIoError;
    type Ok = ();
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeSeq = Self;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeTupleStruct = Self::SerializeSeq;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

    #[cold]
    fn unimplemented(self, _ty: &'static str) -> Self::Error {
        panic!("Array<T> wrapper incorrectly used on non-sequential type")
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        raw::write_i32(self.writer, value.len() as i32)?;
        self.writer.write_all(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.ok_or(NbtIoError::MissingLength)?;
        raw::write_i32(self.writer, len as i32)?;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }
}

impl<'a, W> SerializeSeq for SerializeArray<'a, W>
where W: Write
{
    type Error = NbtIoError;
    type Ok = ();

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        value.serialize(
            SerializeListElement::new(self.writer, NoPrefix, &UNCHECKED).into_serializer(),
        )
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> SerializeTuple for SerializeArray<'a, W>
where W: Write
{
    type Error = NbtIoError;
    type Ok = ();

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        <Self as SerializeSeq>::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeSeq>::end(self)
    }
}

impl<'a, W> SerializeTupleStruct for SerializeArray<'a, W>
where W: Write
{
    type Error = NbtIoError;
    type Ok = ();

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        <Self as SerializeSeq>::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeSeq>::end(self)
    }
}

pub struct SerializeList<'a, W, C> {
    writer: &'a mut W,
    length: Option<i32>,
    type_checker: C,
}

impl<'a, W, C> SerializeList<'a, W, C>
where
    W: Write,
    C: TypeChecker,
{
    fn new(writer: &'a mut W, length: i32) -> Result<Self, NbtIoError> {
        Ok(SerializeList {
            writer,
            length: Some(length),
            type_checker: C::new(),
        })
    }
}

impl<'a, W, C> SerializeSeq for SerializeList<'a, W, C>
where
    W: Write,
    C: TypeChecker,
{
    type Error = NbtIoError;
    type Ok = ();

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        match self.length.take() {
            None => value.serialize(
                SerializeListElement::new(self.writer, NoPrefix, &self.type_checker)
                    .into_serializer(),
            ),
            Some(length) => value.serialize(
                SerializeListElement::new(
                    self.writer,
                    LengthPrefix::new(length),
                    &self.type_checker,
                )
                .into_serializer(),
            ),
        }
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Empty list
        if let Some(..) = self.length {
            self.writer.write_all(&[0, 0, 0, 0, 0])?;
        }

        Ok(())
    }
}

impl<'a, W, C> SerializeTuple for SerializeList<'a, W, C>
where
    W: Write,
    C: TypeChecker,
{
    type Error = NbtIoError;
    type Ok = ();

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        <Self as SerializeSeq>::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeSeq>::end(self)
    }
}

impl<'a, W, C> SerializeTupleStruct for SerializeList<'a, W, C>
where
    W: Write,
    C: TypeChecker,
{
    type Error = NbtIoError;
    type Ok = ();

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        <Self as SerializeSeq>::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeSeq>::end(self)
    }
}

impl<'a, W, C> SerializeTupleVariant for SerializeList<'a, W, C>
where
    W: Write,
    C: TypeChecker,
{
    type Error = NbtIoError;
    type Ok = ();

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        <Self as SerializeSeq>::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Empty list
        if let Some(..) = self.length {
            self.writer.write_all(&[0, 0, 0, 0, 0])?;
        }

        // Add a TAG_End because tuple variants are serialized as { name: [data...] }
        raw::write_u8(self.writer, raw::id_for_tag(None))?;
        Ok(())
    }
}

struct SerializeListElement<'a, W, P, C> {
    writer: &'a mut W,
    prefix: P,
    type_checker: &'a C,
}

impl<'a, W, P, C> SerializeListElement<'a, W, P, C>
where
    W: Write,
    P: Prefix,
    C: TypeChecker,
{
    #[inline]
    fn new(writer: &'a mut W, inner_prefix: P, inner_type_checker: &'a C) -> Self {
        SerializeListElement {
            writer,
            prefix: inner_prefix,
            type_checker: inner_type_checker,
        }
    }
}

impl<'a, W, P, C> DefaultSerializer for SerializeListElement<'a, W, P, C>
where
    W: Write,
    P: Prefix,
    C: TypeChecker,
{
    type Error = NbtIoError;
    type Ok = ();
    type SerializeMap = SerializeCompound<'a, W, C>;
    type SerializeSeq = SerializeList<'a, W, C>;
    type SerializeStruct = SerializeCompound<'a, W, C>;
    type SerializeStructVariant = SerializeCompound<'a, W, C>;
    type SerializeTuple = SerializeList<'a, W, C>;
    type SerializeTupleStruct = SerializeList<'a, W, C>;
    type SerializeTupleVariant = SerializeList<'a, W, C>;

    #[cold]
    fn unimplemented(self, ty: &'static str) -> Self::Error {
        NbtIoError::UnsupportedType(ty)
    }

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.type_checker.verify(0x1)?;
        self.prefix.write(self.writer, 0x1)?;
        raw::write_bool(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.type_checker.verify(0x1)?;
        self.prefix.write(self.writer, 0x1)?;
        raw::write_i8(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i8(value as i8)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.type_checker.verify(0x2)?;
        self.prefix.write(self.writer, 0x2)?;
        raw::write_i16(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.type_checker.verify(0x3)?;
        self.prefix.write(self.writer, 0x3)?;
        raw::write_i32(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.type_checker.verify(0x4)?;
        self.prefix.write(self.writer, 0x4)?;
        raw::write_i64(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.type_checker.verify(0x5)?;
        self.prefix.write(self.writer, 0x5)?;
        raw::write_f32(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.type_checker.verify(0x6)?;
        self.prefix.write(self.writer, 0x6)?;
        raw::write_f64(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.type_checker.verify(0x8)?;
        self.prefix.write(self.writer, 0x8)?;
        raw::write_string(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.type_checker.verify(0x7)?;
        self.prefix.write(self.writer, 0x7)?;
        raw::write_i32(self.writer, value.len() as i32)?;
        self.writer.write_all(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(NbtIoError::OptionInList)
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where T: Serialize {
        Err(NbtIoError::OptionInList)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.type_checker.verify(0x3)?;
        self.prefix.write(self.writer, 0x3)?;
        raw::write_i32(self.writer, variant_index as i32)?;
        Ok(())
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        match name {
            BYTE_ARRAY_NICHE => {
                self.type_checker.verify(0x7)?;
                self.prefix.write(self.writer, 0x7)?;
            }
            INT_ARRAY_NICHE => {
                self.type_checker.verify(0xB)?;
                self.prefix.write(self.writer, 0xB)?;
            }
            LONG_ARRAY_NICHE => {
                self.type_checker.verify(0xC)?;
                self.prefix.write(self.writer, 0xC)?;
            }
            _ => return value.serialize(self.into_serializer()),
        }
        value.serialize(SerializeArray::new(self.writer).into_serializer())
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.type_checker.verify(0xA)?;
        self.prefix.write(self.writer, 0xA)?;
        value.serialize(
            SerializeCompoundEntry::<_, C, _>::new(self.writer, BorrowedPrefix::new(variant))
                .into_serializer(),
        )?;
        raw::write_u8(self.writer, raw::id_for_tag(None))?;
        Ok(())
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.type_checker.verify(0x9)?;
        self.prefix.write(self.writer, 0x9)?;
        let len = len.ok_or(NbtIoError::MissingLength)?;

        SerializeList::new(self.writer, len as i32)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        // [{name: []}]

        // Check that we're allowed to have compounds in this list
        self.type_checker.verify(0xA)?;
        self.prefix.write(self.writer, 0xA)?;

        // Write the compound
        let prefix = BorrowedPrefix::new(variant);
        SerializeCompoundEntry::new(self.writer, prefix).serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.type_checker.verify(0xA)?;
        self.prefix.write(self.writer, 0xA)?;
        Ok(SerializeCompound::new(self.writer))
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.type_checker.verify(0xA)?;
        self.prefix.write(self.writer, 0xA)?;
        raw::write_u8(self.writer, 0xA)?;
        raw::write_string(self.writer, variant)?;
        // The extra closing tag is added by the SerializeStructVariant impl
        Ok(SerializeCompound::new(self.writer))
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

pub struct SerializeCompound<'a, W, C> {
    writer: &'a mut W,
    key: Option<Box<[u8]>>,
    _phantom: PhantomData<C>,
}

impl<'a, W: Write, C: TypeChecker> SerializeCompound<'a, W, C> {
    #[inline]
    fn new(writer: &'a mut W) -> Self {
        SerializeCompound {
            writer,
            key: None,
            _phantom: PhantomData,
        }
    }
}

impl<'a, W: Write, C: TypeChecker> SerializeMap for SerializeCompound<'a, W, C> {
    type Error = NbtIoError;
    type Ok = ();

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where T: Serialize {
        let mut cursor = Cursor::new(Vec::new());
        key.serialize(SerializeKey::new(&mut cursor).into_serializer())?;
        self.key = Some(cursor.into_inner().into_boxed_slice());
        Ok(())
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        let key = self
            .key
            .take()
            .expect("serialize_value called before key was serialized.");
        let prefix = RawPrefix::new(key);
        value.serialize(
            SerializeCompoundEntry::<_, C, _>::new(self.writer, prefix).into_serializer(),
        )
    }

    #[inline]
    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        let prefix = BorrowedPrefix::new(key);
        value.serialize(
            SerializeCompoundEntry::<_, C, _>::new(self.writer, prefix).into_serializer(),
        )
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        raw::write_u8(self.writer, raw::id_for_tag(None))?;
        Ok(())
    }
}

impl<'a, W: Write, C: TypeChecker> SerializeStruct for SerializeCompound<'a, W, C> {
    type Error = NbtIoError;
    type Ok = ();

    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let prefix = BorrowedPrefix::new(key);
        value.serialize(
            SerializeCompoundEntry::<_, C, _>::new(self.writer, prefix).into_serializer(),
        )
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        raw::write_u8(self.writer, raw::id_for_tag(None))?;
        Ok(())
    }
}

impl<'a, W: Write, C: TypeChecker> SerializeStructVariant for SerializeCompound<'a, W, C> {
    type Error = NbtIoError;
    type Ok = ();

    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        <Self as SerializeStruct>::serialize_field(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Add an extra tag because struct variants are serialized as { name: { fields... } }
        self.writer.write_all(&[0, 0])?;
        Ok(())
    }
}

struct SerializeCompoundEntry<'a, W, C, P> {
    writer: &'a mut W,
    prefix: P,
    _phantom: PhantomData<C>,
}

impl<'a, W: Write, C: TypeChecker, P: Prefix> SerializeCompoundEntry<'a, W, C, P> {
    #[inline]
    fn new(writer: &'a mut W, prefix: P) -> Self {
        SerializeCompoundEntry {
            writer,
            prefix,
            _phantom: PhantomData,
        }
    }
}

impl<'a, W, C, P> DefaultSerializer for SerializeCompoundEntry<'a, W, C, P>
where
    W: Write,
    C: TypeChecker,
    P: Prefix,
{
    type Error = NbtIoError;
    type Ok = ();
    type SerializeMap = SerializeCompound<'a, W, C>;
    type SerializeSeq = SerializeList<'a, W, C>;
    type SerializeStruct = SerializeCompound<'a, W, C>;
    type SerializeStructVariant = SerializeCompound<'a, W, C>;
    type SerializeTuple = SerializeList<'a, W, C>;
    type SerializeTupleStruct = SerializeList<'a, W, C>;
    type SerializeTupleVariant = SerializeList<'a, W, C>;

    #[cold]
    fn unimplemented(self, ty: &'static str) -> Self::Error {
        NbtIoError::UnsupportedType(ty)
    }

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x1)?;
        raw::write_bool(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x1)?;
        raw::write_i8(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i8(value as i8)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x2)?;
        raw::write_i16(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x3)?;
        raw::write_i32(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x4)?;
        raw::write_i64(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x5)?;
        raw::write_f32(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x6)?;
        raw::write_f64(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x8)?;
        raw::write_string(self.writer, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x7)?;
        raw::write_i32(self.writer, value.len() as i32)?;
        self.writer.write_all(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where T: Serialize {
        value.serialize(self.into_serializer())
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x3)?;
        raw::write_i32(self.writer, variant_index as i32)?;
        Ok(())
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        match name {
            BYTE_ARRAY_NICHE => {
                self.prefix.write(self.writer, 0x7)?;
            }
            INT_ARRAY_NICHE => {
                self.prefix.write(self.writer, 0xB)?;
            }
            LONG_ARRAY_NICHE => {
                self.prefix.write(self.writer, 0xC)?;
            }
            _ => return value.serialize(self.into_serializer()),
        }
        value.serialize(SerializeArray::new(self.writer).into_serializer())
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.prefix.write(self.writer, 0xA)?;
        value.serialize(
            SerializeCompoundEntry::<_, C, _>::new(self.writer, BorrowedPrefix::new(variant))
                .into_serializer(),
        )?;
        raw::write_u8(self.writer, raw::id_for_tag(None))?;
        Ok(())
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.prefix.write(self.writer, 0x9)?;
        let len = len.ok_or(NbtIoError::MissingLength)?;

        SerializeList::new(self.writer, len as i32)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.prefix.write(self.writer, 0xA)?;
        let prefix = BorrowedPrefix::new(variant);
        SerializeCompoundEntry::new(self.writer, prefix).serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.prefix.write(self.writer, 0xA)?;
        Ok(SerializeCompound::new(self.writer))
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.prefix.write(self.writer, 0xA)?;
        raw::write_u8(self.writer, 0xA)?;
        raw::write_string(self.writer, variant)?;
        // The extra closing tag is added by the SerializeStructVariant impl
        Ok(SerializeCompound::new(self.writer))
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct SerializeKey<'a, W> {
    writer: &'a mut W,
}

impl<'a, W: Write> SerializeKey<'a, W> {
    #[inline]
    fn new(writer: &'a mut W) -> Self {
        SerializeKey { writer }
    }
}

impl<'a, W: Write> DefaultSerializer for SerializeKey<'a, W> {
    type Error = NbtIoError;
    type Ok = ();
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

    #[cold]
    fn unimplemented(self, _ty: &'static str) -> Self::Error {
        NbtIoError::InvalidKey
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        raw::write_string(self.writer, value)?;
        Ok(())
    }
}

pub trait TypeChecker: Sized {
    fn new() -> Self;

    fn verify(&self, tag_id: u8) -> Result<(), NbtIoError>;
}

pub struct Unchecked;

const UNCHECKED: Unchecked = Unchecked;

impl TypeChecker for Unchecked {
    #[inline]
    fn new() -> Self {
        Unchecked
    }

    #[inline]
    fn verify(&self, _tag_id: u8) -> Result<(), NbtIoError> {
        Ok(())
    }
}

pub struct Homogenous {
    id: Cell<Option<u8>>,
}

impl TypeChecker for Homogenous {
    #[inline]
    fn new() -> Self {
        Homogenous {
            id: Cell::new(None),
        }
    }

    #[inline]
    fn verify(&self, tag_id: u8) -> Result<(), NbtIoError> {
        match self.id.get() {
            Some(id) =>
                if id == tag_id {
                    Ok(())
                } else {
                    Err(NbtIoError::NonHomogenousList {
                        list_type: id,
                        encountered_type: tag_id
                    })
                },
            None => {
                self.id.set(Some(tag_id));
                Ok(())
            }
        }
    }
}

pub trait Prefix: Sized {
    fn write_raw<W: Write>(self, writer: &mut W) -> Result<(), NbtIoError>;

    #[inline]
    fn write<W: Write>(self, writer: &mut W, tag_id: u8) -> Result<(), NbtIoError> {
        raw::write_u8(writer, tag_id)?;
        self.write_raw(writer)
    }
}

pub struct NoPrefix;

impl Prefix for NoPrefix {
    #[inline]
    fn write_raw<W: Write>(self, _writer: &mut W) -> Result<(), NbtIoError> {
        Ok(())
    }

    #[inline]
    fn write<W: Write>(self, _writer: &mut W, _tag_id: u8) -> Result<(), NbtIoError> {
        Ok(())
    }
}

struct LengthPrefix {
    length: i32,
}

impl LengthPrefix {
    #[inline]
    fn new(length: i32) -> Self {
        LengthPrefix { length }
    }
}

impl Prefix for LengthPrefix {
    #[inline]
    fn write_raw<W: Write>(self, writer: &mut W) -> Result<(), NbtIoError> {
        raw::write_i32(writer, self.length)?;
        Ok(())
    }
}

pub struct BorrowedPrefix<K> {
    key: K,
}

impl<K: Serialize> BorrowedPrefix<K> {
    #[inline]
    fn new(key: K) -> Self {
        BorrowedPrefix { key }
    }
}

impl<K: Serialize> Prefix for BorrowedPrefix<K> {
    #[inline]
    fn write_raw<W: Write>(self, writer: &mut W) -> Result<(), NbtIoError> {
        self.key
            .serialize(SerializeKey::new(writer).into_serializer())
    }
}

struct RawPrefix {
    raw: Box<[u8]>,
}

impl RawPrefix {
    #[inline]
    fn new(raw: Box<[u8]>) -> Self {
        RawPrefix { raw }
    }
}

impl Prefix for RawPrefix {
    #[inline]
    fn write_raw<W: Write>(self, writer: &mut W) -> Result<(), NbtIoError> {
        writer.write_all(&self.raw)?;
        Ok(())
    }
}
