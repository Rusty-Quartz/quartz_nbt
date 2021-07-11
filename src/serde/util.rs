use serde::{
    ser::{
        self,
        SerializeMap,
        SerializeSeq,
        SerializeStruct,
        SerializeStructVariant,
        SerializeTuple,
        SerializeTupleStruct,
        SerializeTupleVariant,
    },
    serde_if_integer128,
    Serialize,
    Serializer,
};

/// This struct serves as a transparent wrapper around other sealed types to allow for blanket
/// implementations of `Serialize`.
pub struct Ser<T>(T);

pub trait DefaultSerializer: Sized {
    type Ok;
    type Error: ser::Error;
    type SerializeSeq: SerializeSeq<Ok = Self::Ok, Error = Self::Error>;
    type SerializeTuple: SerializeTuple<Ok = Self::Ok, Error = Self::Error>;
    type SerializeTupleStruct: SerializeTupleStruct<Ok = Self::Ok, Error = Self::Error>;
    type SerializeTupleVariant: SerializeTupleVariant<Ok = Self::Ok, Error = Self::Error>;
    type SerializeMap: SerializeMap<Ok = Self::Ok, Error = Self::Error>;
    type SerializeStruct: SerializeStruct<Ok = Self::Ok, Error = Self::Error>;
    type SerializeStructVariant: SerializeStructVariant<Ok = Self::Ok, Error = Self::Error>;

    fn unimplemented(self, ty: &'static str) -> Self::Error;

    fn into_serializer(self) -> Ser<Self> {
        Ser(self)
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("bool"))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("i8"))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("i16"))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("i32"))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("i64"))
    }

    serde_if_integer128! {
        fn serialize_i128(self, _v: i128) -> Result<Self::Ok, Self::Error> {
            Err(self.unimplemented("i128"))
        }
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("u8"))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("u16"))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("u32"))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("u64"))
    }

    serde_if_integer128! {
        fn serialize_u128(self, _v: u128) -> Result<Self::Ok, Self::Error> {
            Err(self.unimplemented("u128"))
        }
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("f64"))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("char"))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("str"))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("&[u8]"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("Option"))
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where T: Serialize {
        Err(self.unimplemented("Option"))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("unit"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("unit struct"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(self.unimplemented("unit variant"))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(self.unimplemented("newtype struct"))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(self.unimplemented("newtype variant"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(self.unimplemented("sequential"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(self.unimplemented("tuple"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(self.unimplemented("tuple struct"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(self.unimplemented("tuple variant"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(self.unimplemented("map"))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(self.unimplemented("struct"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(self.unimplemented("struct variant"))
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        true
    }
}

impl<S: DefaultSerializer> Serializer for Ser<S> {
    type Error = <S as DefaultSerializer>::Error;
    type Ok = <S as DefaultSerializer>::Ok;
    type SerializeMap = <S as DefaultSerializer>::SerializeMap;
    type SerializeSeq = <S as DefaultSerializer>::SerializeSeq;
    type SerializeStruct = <S as DefaultSerializer>::SerializeStruct;
    type SerializeStructVariant = <S as DefaultSerializer>::SerializeStructVariant;
    type SerializeTuple = <S as DefaultSerializer>::SerializeTuple;
    type SerializeTupleStruct = <S as DefaultSerializer>::SerializeTupleStruct;
    type SerializeTupleVariant = <S as DefaultSerializer>::SerializeTupleVariant;

    serde_if_integer128! {
        #[inline(always)]
        fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
            self.0.serialize_i128(v)
        }
    }

    serde_if_integer128! {
        #[inline(always)]
        fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
            self.0.serialize_u128(v)
        }
    }

    #[inline(always)]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_bool(v)
    }

    #[inline(always)]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_i8(v)
    }

    #[inline(always)]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_i16(v)
    }

    #[inline(always)]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_i32(v)
    }

    #[inline(always)]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_i64(v)
    }

    #[inline(always)]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_u8(v)
    }

    #[inline(always)]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_u16(v)
    }

    #[inline(always)]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_u32(v)
    }

    #[inline(always)]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_u64(v)
    }

    #[inline(always)]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_f32(v)
    }

    #[inline(always)]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_f64(v)
    }

    #[inline(always)]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_char(v)
    }

    #[inline(always)]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_str(v)
    }

    #[inline(always)]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_bytes(v)
    }

    #[inline(always)]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_none()
    }

    #[inline(always)]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where T: Serialize {
        self.0.serialize_some(value)
    }

    #[inline(always)]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_unit()
    }

    #[inline(always)]
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_unit_struct(name)
    }

    #[inline(always)]
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_unit_variant(name, variant_index, variant)
    }

    #[inline(always)]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.0.serialize_newtype_struct(name, value)
    }

    #[inline(always)]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.0
            .serialize_newtype_variant(name, variant_index, variant, value)
    }

    #[inline(always)]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.0.serialize_seq(len)
    }

    #[inline(always)]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.0.serialize_tuple(len)
    }

    #[inline(always)]
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.0.serialize_tuple_struct(name, len)
    }

    #[inline(always)]
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.0
            .serialize_tuple_variant(name, variant_index, variant, len)
    }

    #[inline(always)]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.0.serialize_map(len)
    }

    #[inline(always)]
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.0.serialize_struct(name, len)
    }

    #[inline(always)]
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.0
            .serialize_struct_variant(name, variant_index, variant, len)
    }

    #[inline(always)]
    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }
}
