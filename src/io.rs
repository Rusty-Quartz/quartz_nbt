use crate::{raw, NbtCompound, NbtList, NbtTag};
use flate2::{
    read::{GzDecoder, ZlibDecoder},
    write::{GzEncoder, ZlibEncoder},
    Compression,
};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io::{self, Read, Write},
};

/// Describes the flavors of NBT data: uncompressed, Zlib compressed and Gz compressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    /// Uncompressed NBT data.
    Uncompressed,
    /// Zlib compressed NBT data. When writing, the default compression level will be used.
    ZlibCompressed,
    /// Zlib compressed NBT data with the given compression level.
    ZlibCompressedWith(Compression),
    /// Gz compressed NBT data. When writing, the default compression level will be used.
    GzCompressed,
    /// Gz compressed NBT data with the given compression level.
    GzCompressedWith(Compression),
}

/// Reads the given flavor of NBT data from the given reader, returning the resulting NBT
/// compound and associated root name.
pub fn read_nbt<R: Read>(
    reader: &mut R,
    flavor: Flavor,
) -> Result<(NbtCompound, String), NbtIoError> {
    match flavor {
        Flavor::Uncompressed => read_nbt_uncompressed(reader),
        Flavor::ZlibCompressed | Flavor::ZlibCompressedWith(_) =>
            read_nbt_uncompressed(&mut ZlibDecoder::new(reader)),
        Flavor::GzCompressed | Flavor::GzCompressedWith(_) =>
            read_nbt_uncompressed(&mut GzDecoder::new(reader)),
    }
}

fn read_nbt_uncompressed<R: Read>(reader: &mut R) -> Result<(NbtCompound, String), NbtIoError> {
    let root_id = raw::read_u8(reader)?;
    if root_id != 0xA {
        return Err(NbtIoError::TagTypeMismatch(0xA, root_id));
    }

    let root_name = raw::read_string(reader)?;
    match read_tag_body(reader, 0xA) {
        Ok(NbtTag::Compound(compound)) => Ok((compound, root_name)),
        Err(e) => Err(e),
        _ => unreachable!(),
    }
}

fn read_tag_body<R: Read>(reader: &mut R, id: u8) -> Result<NbtTag, NbtIoError> {
    let tag = match id {
        0x1 => NbtTag::Byte(raw::read_i8(reader)?),
        0x2 => NbtTag::Short(raw::read_i16(reader)?),
        0x3 => NbtTag::Int(raw::read_i32(reader)?),
        0x4 => NbtTag::Long(raw::read_i64(reader)?),
        0x5 => NbtTag::Float(raw::read_f32(reader)?),
        0x6 => NbtTag::Double(raw::read_f64(reader)?),
        0x7 => {
            let len = raw::read_i32(reader)? as usize;
            // TODO: consider using some unsafe to avoid initialization
            let mut array = vec![0u8; len];

            reader.read_exact(&mut array)?;

            NbtTag::ByteArray(raw::cast_byte_buf_to_signed(array))
        }
        0x8 => NbtTag::String(raw::read_string(reader)?),
        0x9 => {
            let type_id = raw::read_u8(reader)?;
            let len = raw::read_i32(reader)? as usize;

            // Make sure we don't have a list of TAG_End unless it's empty or an invalid type
            if type_id > 0xC || (type_id == 0 && len > 0) {
                return Err(NbtIoError::InvalidTagId(type_id));
            }

            if len == 0 {
                return Ok(NbtTag::List(NbtList::new()));
            }

            let mut list = NbtList::with_capacity(len);
            for _ in 0 .. len {
                list.push(read_tag_body(reader, type_id)?);
            }

            NbtTag::List(list)
        }
        0xA => {
            let mut compound = NbtCompound::new();
            let mut tag_id = raw::read_u8(reader)?;

            // Read until TAG_End
            while tag_id != 0x0 {
                let name = raw::read_string(reader)?;
                let tag = read_tag_body(reader, tag_id)?;
                compound.insert(name, tag);
                tag_id = raw::read_u8(reader)?;
            }

            NbtTag::Compound(compound)
        }
        0xB => {
            let len = raw::read_i32(reader)? as usize;
            let mut array = Vec::with_capacity(len);

            for _ in 0 .. len {
                array.push(raw::read_i32(reader)?);
            }

            NbtTag::IntArray(array)
        }
        0xC => {
            let len = raw::read_i32(reader)? as usize;
            let mut array = Vec::with_capacity(len);

            for _ in 0 .. len {
                array.push(raw::read_i64(reader)?);
            }

            NbtTag::LongArray(array)
        }
        _ => return Err(NbtIoError::InvalidTagId(id)),
    };

    Ok(tag)
}

/// Writes the given flavor of NBT data to the given writer. If no root name is provided, and empty
/// string is used.
pub fn write_nbt<W: Write>(
    writer: &mut W,
    root_name: Option<&str>,
    root: &NbtCompound,
    flavor: Flavor,
) -> Result<(), NbtIoError> {
    let (mode, compression) = match flavor {
        Flavor::Uncompressed => {
            return write_nbt_uncompressed(writer, root_name, root);
        }
        Flavor::ZlibCompressed => (2, Compression::default()),
        Flavor::ZlibCompressedWith(compression) => (2, compression),
        Flavor::GzCompressed => (1, Compression::default()),
        Flavor::GzCompressedWith(compression) => (1, compression),
    };

    if mode == 1 {
        write_nbt_uncompressed(&mut GzEncoder::new(writer, compression), root_name, root)
    } else {
        write_nbt_uncompressed(&mut ZlibEncoder::new(writer, compression), root_name, root)
    }
}

/// Writes the given tag compound with the given name to the provided writer, writing only the raw
/// NBT data without any compression.
fn write_nbt_uncompressed<W>(
    writer: &mut W,
    root_name: Option<&str>,
    root: &NbtCompound,
) -> Result<(), NbtIoError>
where
    W: Write,
{
    // Compound ID
    raw::write_u8(writer, 0xA)?;
    raw::write_string(writer, root_name.unwrap_or(""))?;
    for (name, tag) in root.inner() {
        raw::write_u8(writer, raw::id_for_tag(Some(tag)))?;
        raw::write_string(writer, name)?;
        write_tag_body(writer, tag)?;
    }
    raw::write_u8(writer, raw::id_for_tag(None))?;
    Ok(())
}

fn write_tag_body<W: Write>(writer: &mut W, tag: &NbtTag) -> Result<(), NbtIoError> {
    match tag {
        &NbtTag::Byte(value) => raw::write_i8(writer, value)?,
        &NbtTag::Short(value) => raw::write_i16(writer, value)?,
        &NbtTag::Int(value) => raw::write_i32(writer, value)?,
        &NbtTag::Long(value) => raw::write_i64(writer, value)?,
        &NbtTag::Float(value) => raw::write_f32(writer, value)?,
        &NbtTag::Double(value) => raw::write_f64(writer, value)?,
        NbtTag::ByteArray(value) => {
            writer.write_all(raw::cast_bytes_to_unsigned(value.as_slice()))?;
        }
        NbtTag::String(value) => raw::write_string(writer, value)?,
        NbtTag::List(value) =>
            if value.is_empty() {
                writer.write_all(&[raw::id_for_tag(None), 0, 0, 0, 0])?;
            } else {
                let type_id = raw::id_for_tag(Some(&value[0]));
                raw::write_u8(writer, type_id)?;
                raw::write_i32(writer, value.len() as i32)?;

                for sub_tag in value.as_ref() {
                    if raw::id_for_tag(Some(sub_tag)) != type_id {
                        return Err(NbtIoError::NonHomogenousList);
                    }

                    write_tag_body(writer, sub_tag)?;
                }
            },
        NbtTag::Compound(value) => {
            for (name, tag) in value.inner() {
                raw::write_u8(writer, raw::id_for_tag(Some(tag)))?;
                raw::write_string(writer, name)?;
                write_tag_body(writer, tag)?;
            }

            // TAG_End
            raw::write_u8(writer, raw::id_for_tag(None))?;
        }
        NbtTag::IntArray(value) => {
            raw::write_i32(writer, value.len() as i32)?;

            for &int in value.iter() {
                raw::write_i32(writer, int)?;
            }
        }
        NbtTag::LongArray(value) => {
            raw::write_i32(writer, value.len() as i32)?;

            for &long in value.iter() {
                raw::write_i64(writer, long)?;
            }
        }
    }

    Ok(())
}

/// Describes an error which occurred during the reading or writing of NBT data.
#[derive(Debug)]
pub enum NbtIoError {
    /// A native I/O error.
    StdIo(io::Error),
    /// No root tag was found. All NBT data must start with a valid compound tag.
    MissingRootTag,
    /// A sequential data structure was found to be non-homogenous. All sequential structures
    /// in NBT data are homogenous.
    NonHomogenousList,
    /// A type requested an option to be read from a list. Since options are indicated by the
    /// absence or presence of a tag, and since all sequential types are length-prefixed,
    /// options cannot exists within arrays in NBT data.
    OptionInList,
    /// A sequential type without a definite length was passed to a serializer.
    MissingLength,
    /// An invalid tag ID was encountered.
    InvalidTagId(u8),
    /// The first tag ID was expected, but the second was found.
    TagTypeMismatch(u8, u8),
    /// A sequential type was expected, but another was found.
    ExpectedSeq,
    /// An enum representation was expected, but another was found.
    ExpectedEnum,
    /// An invalid map key was encountered.
    InvalidKey,
    /// An invalid enum variant was encountered.
    InvalidEnumVariant,
    /// An unsupported type was passed to a serializer or queried from a deserializer.
    UnsupportedType(&'static str),
    /// A custom error message.
    Custom(Box<str>),
}

#[cfg(feature = "serde")]
impl serde::ser::Error for NbtIoError {
    fn custom<T>(msg: T) -> Self
    where T: Display {
        NbtIoError::Custom(msg.to_string().into_boxed_str())
    }
}

#[cfg(feature = "serde")]
impl serde::de::Error for NbtIoError {
    fn custom<T>(msg: T) -> Self
    where T: Display {
        NbtIoError::Custom(msg.to_string().into_boxed_str())
    }
}

impl From<io::Error> for NbtIoError {
    fn from(error: io::Error) -> Self {
        NbtIoError::StdIo(error)
    }
}

impl Display for NbtIoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NbtIoError::StdIo(error) => write!(f, "{}", error),
            NbtIoError::MissingRootTag =>
                write!(f, "NBT tree does not start with a valid root tag."),
            NbtIoError::NonHomogenousList =>
                write!(f, "Encountered non-homogenous list or sequential type"),
            NbtIoError::OptionInList => write!(
                f,
                "Minecraft's NBT format cannot support options in sequential data structures"
            ),
            NbtIoError::MissingLength => write!(
                f,
                "Sequential types must have an initial computable length to be serializable"
            ),
            &NbtIoError::InvalidTagId(id) => write!(
                f,
                "Encountered invalid tag ID 0x{:X} during deserialization",
                id
            ),
            &NbtIoError::TagTypeMismatch(expected, found) => write!(
                f,
                "Tag type mismatch: expected 0x{:X} but found 0x{:X}",
                expected, found
            ),
            NbtIoError::ExpectedSeq => write!(f, "Expected sequential tag type (array)"),
            NbtIoError::ExpectedEnum => write!(
                f,
                "Encountered invalid enum representation in the NBT tag tree"
            ),
            NbtIoError::InvalidKey => write!(f, "Map keys must be a valid string"),
            NbtIoError::InvalidEnumVariant =>
                write!(f, "Encountered invalid enum variant while deserializing"),
            NbtIoError::UnsupportedType(ty) =>
                write!(f, "Type {} is not supported by Minecraft's NBT format", ty),
            NbtIoError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for NbtIoError {}
