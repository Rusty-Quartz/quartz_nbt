mod de;
mod ser;
mod util;

pub use de::Deserializer;
pub use ser::{Serializer, UncheckedSerializer};
pub use util::Ser;

use crate::io::{Flavor, NbtIoError};
use flate2::{
    read::{GzDecoder, ZlibDecoder},
    write::{GzEncoder, ZlibEncoder},
    Compression,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::{Borrow, BorrowMut},
    convert::{AsMut, AsRef},
    io::{Cursor, Read, Write},
    marker::PhantomData,
};

/// Serializes the given value as binary NBT data, returning the resulting Vec. The value must
/// be a struct or non-unit enum variant, else the serializer will return with an error.
pub fn serialize<T: Serialize>(
    value: &T,
    root_name: Option<&str>,
    flavor: Flavor,
) -> Result<Vec<u8>, NbtIoError> {
    let mut cursor = Cursor::new(Vec::<u8>::new());
    serialize_into(&mut cursor, value, root_name, flavor)?;
    Ok(cursor.into_inner())
}

/// Similar to [`serialize`], but elides homogeneity checks on sequential types.  This
/// means that there are some `T` for which this method will return invalid NBT data.
///
/// [`serialize`]: crate::serde::serialize
pub fn serialize_unchecked<T: Serialize>(
    value: &T,
    root_name: Option<&str>,
    flavor: Flavor,
) -> Result<Vec<u8>, NbtIoError> {
    let mut cursor = Cursor::new(Vec::<u8>::new());
    serialize_into_unchecked(&mut cursor, value, root_name, flavor)?;
    Ok(cursor.into_inner())
}

/// Serializes the given value as binary NBT data, writing to the given writer. The value must
/// be a struct or non-unit enum variant, else the serializer will return with an error.
pub fn serialize_into<W: Write, T: Serialize>(
    writer: &mut W,
    value: &T,
    root_name: Option<&str>,
    flavor: Flavor,
) -> Result<(), NbtIoError> {
    let (mode, compression) = match flavor {
        Flavor::Uncompressed => {
            return value.serialize(Serializer::new(writer, root_name));
        }
        Flavor::ZlibCompressed => (2, Compression::default()),
        Flavor::ZlibCompressedWith(compression) => (2, compression),
        Flavor::GzCompressed => (1, Compression::default()),
        Flavor::GzCompressedWith(compression) => (1, compression),
    };

    if mode == 1 {
        value.serialize(Serializer::new(
            &mut GzEncoder::new(writer, compression),
            root_name,
        ))
    } else {
        value.serialize(Serializer::new(
            &mut ZlibEncoder::new(writer, compression),
            root_name,
        ))
    }
}

/// Similar to [`serialize_into`], but elides checks for homogeneity on sequential types. This
/// means that there are some `T` for which this method will write invalid NBT data to the
/// given writer.
///
/// [`serialize_into`]: crate::serde::serialize_into
pub fn serialize_into_unchecked<W: Write, T: Serialize>(
    writer: &mut W,
    value: &T,
    root_name: Option<&str>,
    flavor: Flavor,
) -> Result<(), NbtIoError> {
    let (mode, compression) = match flavor {
        Flavor::Uncompressed => {
            return value.serialize(UncheckedSerializer::new(writer, root_name));
        }
        Flavor::ZlibCompressed => (2, Compression::default()),
        Flavor::ZlibCompressedWith(compression) => (2, compression),
        Flavor::GzCompressed => (1, Compression::default()),
        Flavor::GzCompressedWith(compression) => (1, compression),
    };

    if mode == 1 {
        value.serialize(UncheckedSerializer::new(
            &mut GzEncoder::new(writer, compression),
            root_name,
        ))
    } else {
        value.serialize(UncheckedSerializer::new(
            &mut ZlibEncoder::new(writer, compression),
            root_name,
        ))
    }
}

/// Deserializes the given type from binary NBT data. The NBT data must start with a compound tag
/// and represent the type `T` correctly, else the deserializer will return with an error.
pub fn deserialize<'de, T: Deserialize<'de>>(
    bytes: &[u8],
    flavor: Flavor,
) -> Result<(T, String), NbtIoError> {
    deserialize_from(&mut Cursor::new(bytes), flavor)
}

/// Deserializes the given type from binary NBT data read from the given reader.  The NBT data must
/// start with a compound tag and represent the type `T` correctly, else the deserializer will return with an error.
pub fn deserialize_from<'de, R: Read, T: Deserialize<'de>>(
    reader: &mut R,
    flavor: Flavor,
) -> Result<(T, String), NbtIoError> {
    match flavor {
        Flavor::Uncompressed => deserialize_from_raw(reader),
        Flavor::ZlibCompressed | Flavor::ZlibCompressedWith(_) =>
            deserialize_from_raw(&mut ZlibDecoder::new(reader)),
        Flavor::GzCompressed | Flavor::GzCompressedWith(_) =>
            deserialize_from_raw(&mut GzDecoder::new(reader)),
    }
}

fn deserialize_from_raw<'de, R: Read, T: Deserialize<'de>>(
    reader: &mut R,
) -> Result<(T, String), NbtIoError> {
    let (de, root_name) = Deserializer::new(reader)?;
    Ok((T::deserialize(de)?, root_name))
}

pub(crate) const ARRAY_NEWTYPE_NAME_NICHE: &str = "__quartz_nbt_array";

/// A transparent wrapper around sequential types to allow the NBT serializer to automatically
/// select an appropriate array type, favoring specialized array types like [`IntArray`] and
/// [`ByteArray`]. Currently this type can only wrap vectors, slices, and arrays, however
/// homogenous tuples may be supported in the future.
///
/// [`IntArray`]: crate::NbtTag::IntArray
/// [`ByteArray`]: crate::NbtTag::ByteArray
// TODO: consider supporting homogenous tuples
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Array<T>(pub(crate) T);

impl<T> Array<T> {
    /// Returns the inner value wrapped by this type.
    #[inline]
    pub fn into_inner(array: Self) -> T {
        array.0
    }
}

impl<T: Serialize> Serialize for Array<T> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_newtype_struct(ARRAY_NEWTYPE_NAME_NICHE, &self.0)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Array<T> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        struct Visitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
            type Value = Array<T>;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "A newtype struct type")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where D: serde::Deserializer<'de> {
                Ok(Array(Deserialize::deserialize(deserializer)?))
            }
        }

        deserializer.deserialize_newtype_struct(ARRAY_NEWTYPE_NAME_NICHE, Visitor(PhantomData))
    }
}

impl<T> AsRef<T> for Array<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Array<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Borrow<T> for Array<T> {
    #[inline]
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> BorrowMut<T> for Array<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> From<Vec<T>> for Array<Vec<T>> {
    #[inline]
    fn from(value: Vec<T>) -> Self {
        Array(value)
    }
}

impl<T> From<Array<Vec<T>>> for Vec<T> {
    #[inline]
    fn from(array: Array<Vec<T>>) -> Self {
        array.0
    }
}

impl<T, const LEN: usize> From<[T; LEN]> for Array<[T; LEN]> {
    #[inline]
    fn from(value: [T; LEN]) -> Self {
        Array(value)
    }
}

impl<T, const LEN: usize> From<Array<[T; LEN]>> for [T; LEN] {
    #[inline]
    fn from(array: Array<[T; LEN]>) -> Self {
        array.0
    }
}

impl<'a, T> From<&'a [T]> for Array<&'a [T]> {
    #[inline]
    fn from(value: &'a [T]) -> Self {
        Array(value)
    }
}

impl<'a, T> From<Array<&'a [T]>> for &'a [T] {
    #[inline]
    fn from(array: Array<&'a [T]>) -> Self {
        array.0
    }
}
