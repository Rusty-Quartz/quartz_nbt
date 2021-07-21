mod array;
mod de;
mod ser;
mod util;

pub use array::Array;
pub(crate) use array::{TypeHint, TYPE_HINT_NICHE};
pub use de::Deserializer;
pub use ser::{Serializer, UncheckedSerializer};
pub use util::Ser;

use crate::io::{Flavor, NbtIoError};
use flate2::{
    read::{GzDecoder, ZlibDecoder},
    write::{GzEncoder, ZlibEncoder},
    Compression,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::{Cursor, Read, Write},
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

/// Serializes the given value as binary NBT data, writing to the given writer.
///
/// The value must be a struct or non-unit enum variant, else the serializer will return with an
/// error.
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

/// Deserializes the given type from uncompressed, binary NBT data, allowing for the type to borrow
/// from the given buffer.
///
/// The NBT data must be uncompressed, start with a compound tag, and represent the type `T`
/// correctly, else the deserializer will return with an error.
pub fn deserialize_from_buffer<'de, T: Deserialize<'de>>(
    buffer: &'de [u8],
) -> Result<(T, Cow<'de, str>), NbtIoError> {
    let mut cursor = Cursor::new(buffer);
    let (de, root_name) = Deserializer::from_cursor(&mut cursor)?;
    Ok((T::deserialize(de)?, root_name))
}

/// Deserializes the given type from binary NBT data.
///
/// The NBT data must start with a compound tag and represent the type `T` correctly, else the
/// deserializer will return with an error.
pub fn deserialize<T: DeserializeOwned>(
    bytes: &[u8],
    flavor: Flavor,
) -> Result<(T, String), NbtIoError> {
    deserialize_from(&mut Cursor::new(bytes), flavor)
}

/// Deserializes the given type from binary NBT data read from the given reader.
///
/// The NBT data must start with a compound tag and represent the type `T` correctly, else the
/// deserializer will return with an error.
pub fn deserialize_from<R: Read, T: DeserializeOwned>(
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

fn deserialize_from_raw<'de: 'a, 'a, R: Read, T: Deserialize<'de>>(
    reader: &'a mut R,
) -> Result<(T, String), NbtIoError> {
    let (de, root_name) = Deserializer::new(reader)?;
    Ok((T::deserialize(de)?, root_name))
}
