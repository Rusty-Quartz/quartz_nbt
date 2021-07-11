use serde::{
    de::{self, Visitor},
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
    Deserializer,
    Serialize,
    Serializer,
};

/// This struct serves as a transparent wrapper around other sealed types to allow for blanket
/// implementations of `Serialize` and `Deserialize`.
pub struct Serde<T>(T);

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

    fn into_serializer(self) -> Serde<Self> {
        Serde(self)
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

impl<S: DefaultSerializer> Serializer for Serde<S> {
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

pub trait DefaultDeserializer<'de>: Sized {
    type Error: de::Error;

    fn unimplemented(self, ty: &'static str) -> Self::Error;

    fn into_deserializer(self) -> Serde<Self> {
        Serde(self)
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("any"))
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("bool"))
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("i8"))
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("i16"))
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("i32"))
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("i64"))
    }

    serde_if_integer128! {
        fn deserialize_i128<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>
        {
            Err(self.unimplemented("i128"))
        }
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("u8"))
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("u16"))
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("u32"))
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("u64"))
    }

    serde_if_integer128! {
        fn deserialize_u128<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>
        {
            Err(self.unimplemented("u128"))
        }
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("f32"))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("f64"))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("char"))
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("str"))
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("string"))
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("&[u8]"))
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("Vec<u8>"))
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("Option"))
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("unit"))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.unimplemented("unit struct"))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.unimplemented("newtype struct"))
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("sequential"))
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("tuple"))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.unimplemented("tuple struct"))
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("map"))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.unimplemented("struct"))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.unimplemented("enum"))
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("identifier"))
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        Err(self.unimplemented("ignored any"))
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        true
    }
}

impl<'de, D: DefaultDeserializer<'de>> Deserializer<'de> for Serde<D> {
    type Error = <D as DefaultDeserializer<'de>>::Error;

    serde_if_integer128! {
        #[inline(always)]
        fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>
        {
            self.0.deserialize_i128(visitor)
        }
    }

    serde_if_integer128! {
        #[inline(always)]
        fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>
        {
            self.0.deserialize_u128(visitor)
        }
    }

    #[inline(always)]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_any(visitor)
    }

    #[inline(always)]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_bool(visitor)
    }

    #[inline(always)]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_i8(visitor)
    }

    #[inline(always)]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_i16(visitor)
    }

    #[inline(always)]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_i32(visitor)
    }

    #[inline(always)]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_i64(visitor)
    }

    #[inline(always)]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_u8(visitor)
    }

    #[inline(always)]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_u16(visitor)
    }

    #[inline(always)]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_u32(visitor)
    }

    #[inline(always)]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_u64(visitor)
    }

    #[inline(always)]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_f32(visitor)
    }

    #[inline(always)]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_f64(visitor)
    }

    #[inline(always)]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_char(visitor)
    }

    #[inline(always)]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_str(visitor)
    }

    #[inline(always)]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_string(visitor)
    }

    #[inline(always)]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_bytes(visitor)
    }

    #[inline(always)]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_byte_buf(visitor)
    }

    #[inline(always)]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_option(visitor)
    }

    #[inline(always)]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_unit(visitor)
    }

    #[inline(always)]
    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_unit_struct(name, visitor)
    }

    #[inline(always)]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_newtype_struct(name, visitor)
    }

    #[inline(always)]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_seq(visitor)
    }

    #[inline(always)]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_tuple(len, visitor)
    }

    #[inline(always)]
    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_tuple_struct(name, len, visitor)
    }

    #[inline(always)]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_map(visitor)
    }

    #[inline(always)]
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_struct(name, fields, visitor)
    }

    #[inline(always)]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_enum(name, variants, visitor)
    }

    #[inline(always)]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_identifier(visitor)
    }

    #[inline(always)]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de> {
        self.0.deserialize_ignored_any(visitor)
    }

    #[inline(always)]
    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }
}
