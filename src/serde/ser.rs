use super::util::{DefaultSerializer, Ser};
use crate::{io::NbtIoError, raw, serde::ARRAY_NEWTYPE_NAME_NICHE};
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
};

/// The serializer type for writing binary NBT data.
pub type Serializer<'a, W> = Ser<SerializerImpl<'a, W>>;

impl<'a, W: Write> Serializer<'a, W> {
    /// Constructs a new serializer with the given writer and root name. If no root name is specified,
    /// then an empty string is written to the header.
    pub fn new(writer: &'a mut W, root_name: Option<&'a str>) -> Self {
        SerializerImpl::new(writer, BorrowedPrefix::new(root_name.unwrap_or(""))).into_serializer()
    }
}

pub struct SerializerImpl<'a, W> {
    writer: &'a mut W,
    root_name: BorrowedPrefix<&'a str>,
}

impl<'a, W: Write> SerializerImpl<'a, W> {
    fn new(writer: &'a mut W, root_name: BorrowedPrefix<&'a str>) -> Self {
        SerializerImpl { writer, root_name }
    }
}

impl<'a, W: Write> DefaultSerializer for SerializerImpl<'a, W> {
    type Error = NbtIoError;
    type Ok = ();
    type SerializeMap = SerializeCompound<'a, W>;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = SerializeCompound<'a, W>;
    type SerializeStructVariant = SerializeCompound<'a, W>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant =
        SerializeList<'a, W, BorrowedPrefix<&'static str>, Unchecked, Homogenous, false>;

    fn unimplemented(self, _ty: &'static str) -> Self::Error {
        NbtIoError::MissingRootTag
    }

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

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.root_name.write(self.writer, 0xA)?;
        Ok(SerializeCompound::new(self.writer))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        <Self as DefaultSerializer>::serialize_map(self, Some(len))
    }

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

struct SerializeArray<'a, W, P, C> {
    writer: &'a mut W,
    outer_prefix: P,
    outer_type_checker: &'a C,
}

impl<'a, W, P, C> SerializeArray<'a, W, P, C>
where
    W: Write,
    P: Prefix,
    C: TypeChecker,
{
    fn new(writer: &'a mut W, outer_prefix: P, outer_type_checker: &'a C) -> Self {
        SerializeArray {
            writer,
            outer_prefix: outer_prefix,
            outer_type_checker,
        }
    }
}

impl<'a, W, P, C> DefaultSerializer for SerializeArray<'a, W, P, C>
where
    W: Write,
    P: Prefix,
    C: TypeChecker,
{
    type Error = NbtIoError;
    type Ok = ();
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    // Since we currently only support wrapping Vecs in Array, we can get away with unchecked
    // sequential serialization
    type SerializeSeq = SerializeList<'a, W, P, C, Unchecked, true>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

    #[cold]
    fn unimplemented(self, _ty: &'static str) -> Self::Error {
        panic!("Array<T> wrapper incorrectly called on non-sequential type")
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.outer_type_checker.verify(0x7)?;
        self.outer_prefix.write(self.writer, 0x7)?;
        raw::write_i32(self.writer, value.len() as i32)?;
        self.writer.write_all(value)?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.ok_or(NbtIoError::MissingLength)?;
        Ok(SerializeList::new(
            self.writer,
            self.outer_prefix,
            self.outer_type_checker,
            len as i32,
        ))
    }
}

pub struct SerializeList<'a, W, P, Co, Ci, const AUTO_RESOLVE_TYPE: bool> {
    writer: &'a mut W,
    outer_prefix: Option<P>,
    length: i32,
    outer_type_checker: &'a Co,
    inner_type_checker: Ci,
}

impl<'a, W, P, Co, Ci, const AUTO_RESOLVE_TYPE: bool> SerializeList<'a, W, P, Co, Ci, AUTO_RESOLVE_TYPE>
where
    W: Write,
    P: Prefix,
    Co: TypeChecker,
    Ci: TypeChecker,
{
    fn new(writer: &'a mut W, outer_prefix: P, outer_type_checker: &'a Co, length: i32) -> Self {
        SerializeList {
            writer,
            outer_prefix: Some(outer_prefix),
            length,
            outer_type_checker,
            inner_type_checker: Ci::new(),
        }
    }
}

impl<'a, W, P, Co, Ci, const AUTO_RESOLVE_TYPE: bool> SerializeSeq for SerializeList<'a, W, P, Co, Ci, AUTO_RESOLVE_TYPE>
where
    W: Write,
    P: Prefix,
    Co: TypeChecker,
    Ci: TypeChecker,
{
    type Error = NbtIoError;
    type Ok = ();

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        match self.outer_prefix.take() {
            Some(outer_prefix) => value.serialize(
                SerializeListElement::<_, _, _, _, _, AUTO_RESOLVE_TYPE>::new(
                    self.writer,
                    outer_prefix,
                    LengthPrefix::new(self.length),
                    self.outer_type_checker,
                    &self.inner_type_checker,
                )
                .into_serializer(),
            ),
            None => value.serialize(
                SerializeListElement::<_, _, _, _, _, AUTO_RESOLVE_TYPE>::new(
                    self.writer,
                    NoPrefix,
                    NoPrefix,
                    &Unchecked,
                    &self.inner_type_checker,
                )
                .into_serializer(),
            ),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Empty list
        if let Some(prefix) = self.outer_prefix {
            self.outer_type_checker.verify(0x9)?;
            prefix.write(self.writer, 0x9)?;
            // We coerce empty tag lists to any list type when deserializing
            self.writer.write_all(&[0, 0, 0, 0, 0])?;
        }

        Ok(())
    }
}

impl<'a, W, P, Co, Ci, const AUTO_RESOLVE_TYPE: bool> SerializeTuple for SerializeList<'a, W, P, Co, Ci, AUTO_RESOLVE_TYPE>
where
    W: Write,
    P: Prefix,
    Co: TypeChecker,
    Ci: TypeChecker,
{
    type Error = NbtIoError;
    type Ok = ();

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        <Self as SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeSeq>::end(self)
    }
}

impl<'a, W, P, Co, Ci, const AUTO_RESOLVE_TYPE: bool> SerializeTupleStruct for SerializeList<'a, W, P, Co, Ci, AUTO_RESOLVE_TYPE>
where
    W: Write,
    P: Prefix,
    Co: TypeChecker,
    Ci: TypeChecker,
{
    type Error = NbtIoError;
    type Ok = ();

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        <Self as SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeSeq>::end(self)
    }
}

impl<'a, W, P, Co, Ci, const AUTO_RESOLVE_TYPE: bool> SerializeTupleVariant for SerializeList<'a, W, P, Co, Ci, AUTO_RESOLVE_TYPE>
where
    W: Write,
    P: Prefix,
    Co: TypeChecker,
    Ci: TypeChecker,
{
    type Error = NbtIoError;
    type Ok = ();

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        <Self as SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Empty list
        if let Some(prefix) = self.outer_prefix {
            self.outer_type_checker.verify(0x9)?;
            prefix.write(self.writer, 0x9)?;
            // We coerce empty tag lists to any list type when deserializing
            self.writer.write_all(&[0, 0, 0, 0, 0])?;
        }

        // Add a TAG_End because tuple variants are serialized as { name: [data...] }
        raw::write_u8(self.writer, raw::id_for_tag(None))?;
        Ok(())
    }
}

struct SerializeListElement<'a, W, Po, Pi, Co, Ci, const AUTO_RESOLVE_TYPE: bool> {
    writer: &'a mut W,
    outer_prefix: Po,
    inner_prefix: Pi,
    outer_type_checker: &'a Co,
    inner_type_checker: &'a Ci,
}

impl<'a, W, Po, Pi, Co, Ci, const AUTO_RESOLVE_TYPE: bool> SerializeListElement<'a, W, Po, Pi, Co, Ci, AUTO_RESOLVE_TYPE>
where
    W: Write,
    Po: Prefix,
    Pi: Prefix,
    Co: TypeChecker,
    Ci: TypeChecker,
{
    fn new(
        writer: &'a mut W,
        outer_prefix: Po,
        inner_prefix: Pi,
        outer_type_checker: &'a Co,
        inner_type_checker: &'a Ci,
    ) -> Self {
        SerializeListElement {
            writer,
            outer_prefix,
            inner_prefix,
            outer_type_checker,
            inner_type_checker,
        }
    }
}

impl<'a, W, Po, Pi, Co, Ci, const AUTO_RESOLVE_TYPE: bool> DefaultSerializer for SerializeListElement<'a, W, Po, Pi, Co, Ci, AUTO_RESOLVE_TYPE>
where
    W: Write,
    Po: Prefix,
    Pi: Prefix,
    Co: TypeChecker,
    Ci: TypeChecker,
{
    type Error = NbtIoError;
    type Ok = ();
    type SerializeMap = SerializeCompound<'a, W>;
    type SerializeSeq = SerializeList<'a, W, Pi, Ci, Homogenous, false>;
    type SerializeStruct = SerializeCompound<'a, W>;
    type SerializeStructVariant = SerializeCompound<'a, W>;
    type SerializeTuple = SerializeList<'a, W, Pi, Ci, Homogenous, false>;
    type SerializeTupleStruct = SerializeList<'a, W, Pi, Ci, Homogenous, false>;
    type SerializeTupleVariant =
        SerializeList<'a, W, BorrowedPrefix<&'static str>, Unchecked, Homogenous, false>;

    fn unimplemented(self, ty: &'static str) -> Self::Error {
        NbtIoError::UnsupportedType(ty)
    }

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        if AUTO_RESOLVE_TYPE {
            self.outer_type_checker.verify(0x7)?;
            self.outer_prefix.write(self.writer, 0x7)?;
            self.inner_type_checker.verify(0x1)?;
            self.inner_prefix.write_raw(self.writer)?;
        } else {
            self.outer_type_checker.verify(0x9)?;
            self.outer_prefix.write(self.writer, 0x9)?;
            self.inner_type_checker.verify(0x1)?;
            self.inner_prefix.write(self.writer, 0x1)?;
        }
        raw::write_bool(self.writer, value)?;
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        if AUTO_RESOLVE_TYPE {
            self.outer_type_checker.verify(0x7)?;
            self.outer_prefix.write(self.writer, 0x7)?;
            self.inner_type_checker.verify(0x1)?;
            self.inner_prefix.write_raw(self.writer)?;
        } else {
            self.outer_type_checker.verify(0x9)?;
            self.outer_prefix.write(self.writer, 0x9)?;
            self.inner_type_checker.verify(0x1)?;
            self.inner_prefix.write(self.writer, 0x1)?;
        }
        raw::write_i8(self.writer, value)?;
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i8(value as i8)
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.outer_type_checker.verify(0x9)?;
        self.outer_prefix.write(self.writer, 0x9)?;
        self.inner_type_checker.verify(0x2)?;
        self.inner_prefix.write(self.writer, 0x2)?;
        raw::write_i16(self.writer, value)?;
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        if AUTO_RESOLVE_TYPE {
            self.outer_type_checker.verify(0xB)?;
            self.outer_prefix.write(self.writer, 0xB)?;
            self.inner_type_checker.verify(0x3)?;
            self.inner_prefix.write_raw(self.writer)?;
        } else {
            self.outer_type_checker.verify(0x9)?;
            self.outer_prefix.write(self.writer, 0x9)?;
            self.inner_type_checker.verify(0x3)?;
            self.inner_prefix.write(self.writer, 0x3)?;
        }
        raw::write_i32(self.writer, value)?;
        Ok(())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        if AUTO_RESOLVE_TYPE {
            self.outer_type_checker.verify(0xC)?;
            self.outer_prefix.write(self.writer, 0xC)?;
            self.inner_type_checker.verify(0x4)?;
            self.inner_prefix.write_raw(self.writer)?;
        } else {
            self.outer_type_checker.verify(0x9)?;
            self.outer_prefix.write(self.writer, 0x9)?;
            self.inner_type_checker.verify(0x4)?;
            self.inner_prefix.write(self.writer, 0x4)?;
        }
        raw::write_i64(self.writer, value)?;
        Ok(())
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.outer_type_checker.verify(0x9)?;
        self.outer_prefix.write(self.writer, 0x9)?;
        self.inner_type_checker.verify(0x5)?;
        self.inner_prefix.write(self.writer, 0x5)?;
        raw::write_f32(self.writer, value)?;
        Ok(())
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.outer_type_checker.verify(0x9)?;
        self.outer_prefix.write(self.writer, 0x9)?;
        self.inner_type_checker.verify(0x6)?;
        self.inner_prefix.write(self.writer, 0x6)?;
        raw::write_f64(self.writer, value)?;
        Ok(())
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.outer_type_checker.verify(0x9)?;
        self.outer_prefix.write(self.writer, 0x9)?;
        self.inner_type_checker.verify(0x8)?;
        self.inner_prefix.write(self.writer, 0x8)?;
        raw::write_string(self.writer, value)?;
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.outer_type_checker.verify(0x9)?;
        self.outer_prefix.write(self.writer, 0x9)?;
        self.inner_type_checker.verify(0x7)?;
        self.inner_prefix.write(self.writer, 0x7)?;
        raw::write_i32(self.writer, value.len() as i32)?;
        self.writer.write_all(value)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(NbtIoError::OptionInList)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where T: Serialize {
        Err(NbtIoError::OptionInList)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        if AUTO_RESOLVE_TYPE {
            self.outer_type_checker.verify(0xB)?;
            self.outer_prefix.write(self.writer, 0xB)?;
            self.inner_type_checker.verify(0x3)?;
            self.inner_prefix.write_raw(self.writer)?;
        } else {
            self.outer_type_checker.verify(0x9)?;
            self.outer_prefix.write(self.writer, 0x9)?;
            self.inner_type_checker.verify(0x3)?;
            self.inner_prefix.write(self.writer, 0x3)?;
        }
        raw::write_i32(self.writer, variant_index as i32)?;
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        if name == ARRAY_NEWTYPE_NAME_NICHE {
            value.serialize(
                SerializeArray::new(
                    self.writer,
                    self.outer_prefix,
                    self.outer_type_checker
                ).into_serializer()
            )
        } else {
            value.serialize(self.into_serializer())
        }
    }

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
        self.outer_type_checker.verify(0x9)?;
        self.outer_prefix.write(self.writer, 0x9)?;
        self.inner_type_checker.verify(0xA)?;
        self.inner_prefix.write(self.writer, 0xA)?;
        value.serialize(
            SerializeCompoundEntry::new(self.writer, BorrowedPrefix::new(variant))
                .into_serializer(),
        )?;
        raw::write_u8(self.writer, raw::id_for_tag(None))?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.outer_type_checker.verify(0x9)?;
        self.outer_prefix.write(self.writer, 0x9)?;
        let len = len.ok_or(NbtIoError::MissingLength)?;
        Ok(SerializeList::new(
            self.writer,
            self.inner_prefix,
            self.inner_type_checker,
            len as i32,
        ))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        // [{name: []}]

        // Check that we're allowed to have a tag list in a tag list
        self.outer_type_checker.verify(0x9)?;
        self.outer_prefix.write(self.writer, 0x9)?;
        // Check that we're allowed to have compounds in this list
        self.inner_type_checker.verify(0xA)?;
        self.inner_prefix.write(self.writer, 0xA)?;

        // Write the compound
        let prefix = BorrowedPrefix::new(variant);
        SerializeCompoundEntry::new(self.writer, prefix).serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.outer_type_checker.verify(0x9)?;
        self.outer_prefix.write(self.writer, 0x9)?;
        self.inner_type_checker.verify(0xA)?;
        self.inner_prefix.write(self.writer, 0xA)?;
        Ok(SerializeCompound::new(self.writer))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.outer_type_checker.verify(0x9)?;
        self.outer_prefix.write(self.writer, 0x9)?;
        self.inner_type_checker.verify(0xA)?;
        self.inner_prefix.write(self.writer, 0xA)?;
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

pub struct SerializeCompound<'a, W> {
    writer: &'a mut W,
    key: Option<Box<[u8]>>,
}

impl<'a, W: Write> SerializeCompound<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        SerializeCompound { writer, key: None }
    }
}

impl<'a, W: Write> SerializeMap for SerializeCompound<'a, W> {
    type Error = NbtIoError;
    type Ok = ();

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where T: Serialize {
        let mut cursor = Cursor::new(Vec::new());
        key.serialize(SerializeKey::new(&mut cursor).into_serializer())?;
        self.key = Some(cursor.into_inner().into_boxed_slice());
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where T: Serialize {
        let key = self
            .key
            .take()
            .expect("serialize_value called before key was serialized.");
        let prefix = RawPrefix::new(key);
        value.serialize(SerializeCompoundEntry::new(self.writer, prefix).into_serializer())
    }

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
        value.serialize(SerializeCompoundEntry::new(self.writer, prefix).into_serializer())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        raw::write_u8(self.writer, raw::id_for_tag(None))?;
        Ok(())
    }
}

impl<'a, W: Write> SerializeStruct for SerializeCompound<'a, W> {
    type Error = NbtIoError;
    type Ok = ();

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let prefix = BorrowedPrefix::new(key);
        value.serialize(SerializeCompoundEntry::new(self.writer, prefix).into_serializer())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        raw::write_u8(self.writer, raw::id_for_tag(None))?;
        Ok(())
    }
}

impl<'a, W: Write> SerializeStructVariant for SerializeCompound<'a, W> {
    type Error = NbtIoError;
    type Ok = ();

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

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Add an extra tag because struct variants are serialized as { name: { fields... } }
        self.writer.write_all(&[0, 0])?;
        Ok(())
    }
}

struct SerializeCompoundEntry<'a, W, P> {
    writer: &'a mut W,
    prefix: P,
}

impl<'a, W: Write, P: Prefix> SerializeCompoundEntry<'a, W, P> {
    fn new(writer: &'a mut W, prefix: P) -> Self {
        SerializeCompoundEntry { writer, prefix }
    }
}

impl<'a, W, P> DefaultSerializer for SerializeCompoundEntry<'a, W, P>
where
    W: Write,
    P: Prefix,
{
    type Error = NbtIoError;
    type Ok = ();
    type SerializeMap = SerializeCompound<'a, W>;
    type SerializeSeq = SerializeList<'a, W, P, Unchecked, Homogenous, false>;
    type SerializeStruct = SerializeCompound<'a, W>;
    type SerializeStructVariant = SerializeCompound<'a, W>;
    type SerializeTuple = SerializeList<'a, W, P, Unchecked, Homogenous, false>;
    type SerializeTupleStruct = SerializeList<'a, W, P, Unchecked, Homogenous, false>;
    type SerializeTupleVariant =
        SerializeList<'a, W, BorrowedPrefix<&'static str>, Unchecked, Homogenous, false>;

    fn unimplemented(self, ty: &'static str) -> Self::Error {
        NbtIoError::UnsupportedType(ty)
    }

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x1)?;
        raw::write_bool(self.writer, value)?;
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x1)?;
        raw::write_i8(self.writer, value)?;
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i8(value as i8)
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x2)?;
        raw::write_i16(self.writer, value)?;
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x3)?;
        raw::write_i32(self.writer, value)?;
        Ok(())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x4)?;
        raw::write_i64(self.writer, value)?;
        Ok(())
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x5)?;
        raw::write_f32(self.writer, value)?;
        Ok(())
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x6)?;
        raw::write_f64(self.writer, value)?;
        Ok(())
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x8)?;
        raw::write_string(self.writer, value)?;
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.prefix.write(self.writer, 0x7)?;
        raw::write_i32(self.writer, value.len() as i32)?;
        self.writer.write_all(value)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where T: Serialize {
        value.serialize(self.into_serializer())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

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

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        if name == ARRAY_NEWTYPE_NAME_NICHE {
            value.serialize(
                SerializeArray::new(
                    self.writer,
                    self.prefix,
                    &UNCHECKED
                ).into_serializer()
            )
        } else {
            value.serialize(self.into_serializer())
        }
    }

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
            SerializeCompoundEntry::new(self.writer, BorrowedPrefix::new(variant))
                .into_serializer(),
        )?;
        raw::write_u8(self.writer, raw::id_for_tag(None))?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.ok_or(NbtIoError::MissingLength)?;
        Ok(SerializeList::new(
            self.writer,
            self.prefix,
            &UNCHECKED,
            len as i32,
        ))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

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

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.prefix.write(self.writer, 0xA)?;
        Ok(SerializeCompound::new(self.writer))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

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

impl<'a, W> SerializeKey<'a, W> {
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

    #[inline]
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
    fn new() -> Self {
        Homogenous {
            id: Cell::new(None),
        }
    }

    fn verify(&self, tag_id: u8) -> Result<(), NbtIoError> {
        match self.id.get() {
            Some(id) =>
                if id == tag_id {
                    Ok(())
                } else {
                    Err(NbtIoError::NonHomogenousList)
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
