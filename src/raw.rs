use crate::{io::NbtIoError, NbtTag};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{Read, Result, Write},
    mem::ManuallyDrop,
    ptr,
    result::Result as StdResult,
    slice,
};

#[inline]
pub const fn id_for_tag(tag: Option<&NbtTag>) -> u8 {
    match tag {
        None => 0x0, // TAG_End
        Some(NbtTag::Byte(..)) => 0x1,
        Some(NbtTag::Short(..)) => 0x2,
        Some(NbtTag::Int(..)) => 0x3,
        Some(NbtTag::Long(..)) => 0x4,
        Some(NbtTag::Float(..)) => 0x5,
        Some(NbtTag::Double(..)) => 0x6,
        Some(NbtTag::ByteArray(..)) => 0x7,
        Some(NbtTag::String(..)) => 0x8,
        Some(NbtTag::List(..)) => 0x9,
        Some(NbtTag::Compound(..)) => 0xA,
        Some(NbtTag::IntArray(..)) => 0xB,
        Some(NbtTag::LongArray(..)) => 0xC,
    }
}

#[cfg(feature = "serde")]
#[inline]
pub fn read_bool<R: Read>(reader: &mut R) -> Result<bool> {
    Ok(read_u8(reader)? != 0)
}

#[inline]
pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8> {
    reader.read_u8()
}

#[inline]
pub fn read_i8<R: Read>(reader: &mut R) -> Result<i8> {
    reader.read_i8()
}

#[inline]
pub fn read_i16<R: Read>(reader: &mut R) -> Result<i16> {
    reader.read_i16::<BigEndian>()
}

#[inline]
pub fn read_u16<R: Read>(reader: &mut R) -> Result<u16> {
    reader.read_u16::<BigEndian>()
}

#[inline]
pub fn read_i32<R: Read>(reader: &mut R) -> Result<i32> {
    reader.read_i32::<BigEndian>()
}

#[inline]
pub fn read_i64<R: Read>(reader: &mut R) -> Result<i64> {
    reader.read_i64::<BigEndian>()
}

#[inline]
pub fn read_f32<R: Read>(reader: &mut R) -> Result<f32> {
    reader.read_f32::<BigEndian>()
}

#[inline]
pub fn read_f64<R: Read>(reader: &mut R) -> Result<f64> {
    reader.read_f64::<BigEndian>()
}

pub fn read_string<R: Read>(reader: &mut R) -> StdResult<String, NbtIoError> {
    let len = read_u16(reader)? as usize;
    let mut bytes = vec![0; len];
    reader.read_exact(&mut bytes)?;

    let java_decoded = match cesu8::from_java_cesu8(&bytes) {
        Ok(string) => string,
        Err(_) => return Err(NbtIoError::InvalidCesu8String),
    };

    Ok(java_decoded.into_owned())
}

#[cfg(feature = "serde")]
pub fn read_string_into<'a, R: Read>(
    reader: &mut R,
    dest: &'a mut Vec<u8>,
) -> StdResult<std::borrow::Cow<'a, str>, NbtIoError> {
    let len = read_u16(reader)? as usize;
    dest.resize(len, 0);
    reader.read_exact(dest)?;
    match cesu8::from_java_cesu8(dest) {
        Ok(string) => Ok(string),
        Err(_) => Err(NbtIoError::InvalidCesu8String),
    }
}

#[cfg(feature = "serde")]
#[inline]
pub fn write_bool<W: Write>(writer: &mut W, value: bool) -> Result<()> {
    write_u8(writer, if value { 1 } else { 0 })
}

#[inline]
pub fn write_u8<W: Write>(writer: &mut W, value: u8) -> Result<()> {
    writer.write_u8(value)
}

#[inline]
pub fn write_i8<W: Write>(writer: &mut W, value: i8) -> Result<()> {
    writer.write_i8(value)
}

#[inline]
pub fn write_i16<W: Write>(writer: &mut W, value: i16) -> Result<()> {
    writer.write_i16::<BigEndian>(value)
}

#[inline]
pub fn write_u16<W: Write>(writer: &mut W, value: u16) -> Result<()> {
    writer.write_u16::<BigEndian>(value)
}

#[inline]
pub fn write_i32<W: Write>(writer: &mut W, value: i32) -> Result<()> {
    writer.write_i32::<BigEndian>(value)
}

#[inline]
pub fn write_i64<W: Write>(writer: &mut W, value: i64) -> Result<()> {
    writer.write_i64::<BigEndian>(value)
}

#[inline]
pub fn write_f32<W: Write>(writer: &mut W, value: f32) -> Result<()> {
    writer.write_f32::<BigEndian>(value)
}

#[inline]
pub fn write_f64<W: Write>(writer: &mut W, value: f64) -> Result<()> {
    writer.write_f64::<BigEndian>(value)
}

pub fn write_string<W: Write>(writer: &mut W, string: &str) -> Result<()> {
    let mod_utf8 = cesu8::to_java_cesu8(string);
    write_u16(writer, mod_utf8.len() as u16)?;
    writer.write_all(&mod_utf8)
}

#[inline]
pub fn cast_byte_buf_to_signed(buf: Vec<u8>) -> Vec<i8> {
    let mut me = ManuallyDrop::new(buf);
    // Pointer cast is valid because i8 and u8 have the same layout
    let ptr = me.as_mut_ptr() as *mut i8;
    let length = me.len();
    let capacity = me.capacity();

    // Safety
    // * `ptr` was allocated by a Vec
    // * i8 has the same size and alignment as u8
    // * `length` and `capacity` came from a valid Vec
    unsafe { Vec::from_raw_parts(ptr, length, capacity) }
}

#[inline]
pub fn cast_byte_buf_to_unsigned(buf: Vec<i8>) -> Vec<u8> {
    let mut me = ManuallyDrop::new(buf);
    // Pointer cast is valid because i8 and u8 have the same layout
    let ptr = me.as_mut_ptr() as *mut u8;
    let length = me.len();
    let capacity = me.capacity();

    // Safety
    // * `ptr` was allocated by a Vec
    // * u8 has the same size and alignment as i8
    // * `length` and `capacity` came from a valid Vec
    unsafe { Vec::from_raw_parts(ptr, length, capacity) }
}

// Currently unused, but might be used later
#[inline]
#[allow(dead_code)]
pub fn cast_bytes_to_signed(bytes: &[u8]) -> &[i8] {
    let data = bytes.as_ptr() as *const i8;
    let len = bytes.len();

    // Safety
    // * `data` is valid for len * 1 bytes
    //     * The entire memory range of `data` is contained in a single
    //       allocated object since it came from a valid slice
    //     * `data` is non-null and aligned correctly for u8 (and thus i8)
    // * `data` points to exactly `len` consecutive bytes
    // * The constructed reference adopts the lifetime of the provided reference
    // * `len` <= isize::MAX because `len` came from a valid slice
    unsafe { slice::from_raw_parts(data, len) }
}

#[inline]
pub fn cast_bytes_to_unsigned(bytes: &[i8]) -> &[u8] {
    let data = bytes.as_ptr() as *const u8;
    let len = bytes.len();

    // Safety
    // * `data` is valid for len * 1 bytes
    //     * The entire memory range of `data` is contained in a single
    //       allocated object since it came from a valid slice
    //     * `data` is non-null and aligned correctly for i8 (and thus u8)
    // * `data` points to exactly `len` consecutive bytes
    // * The constructed reference adopts the lifetime of the provided reference
    // * `len` <= isize::MAX because `len` came from a valid slice
    unsafe { slice::from_raw_parts(data, len) }
}

#[inline]
pub fn read_i32_array<R: Read>(reader: &mut R, len: usize) -> Result<Vec<i32>> {
    let mut bytes = ManuallyDrop::new(vec![0i32; len]);

    let ptr = bytes.as_mut_ptr() as *mut u8;
    let length = bytes.len() * 4;
    let capacity = bytes.capacity() * 4;

    let mut bytes = unsafe { Vec::from_raw_parts(ptr, length, capacity) };

    reader.read_exact(&mut bytes)?;

    // Safety: the length of the vec is a multiple of 4, and the alignment is 4
    Ok(unsafe { convert_be_int_array_in_place::<i32, 4>(bytes, i32::from_be_bytes) })
}

#[inline]
pub fn read_i64_array<R: Read>(reader: &mut R, len: usize) -> Result<Vec<i64>> {
    let mut bytes = ManuallyDrop::new(vec![0i64; len]);

    let ptr = bytes.as_mut_ptr() as *mut u8;
    let length = bytes.len() * 8;
    let capacity = bytes.capacity() * 8;

    let mut bytes = unsafe { Vec::from_raw_parts(ptr, length, capacity) };

    reader.read_exact(&mut bytes)?;

    // Safety: the length of the vec is a multiple of 8, and the alignment is 8
    Ok(unsafe { convert_be_int_array_in_place::<i64, 8>(bytes, i64::from_be_bytes) })
}

#[inline]
unsafe fn convert_be_int_array_in_place<I, const SIZE: usize>(
    mut bytes: Vec<u8>,
    convert: fn([u8; SIZE]) -> I
) -> Vec<I> {
    let mut buf: [u8; SIZE];

    let mut read = bytes.as_ptr() as *const [u8; SIZE];
    let mut write = bytes.as_mut_ptr() as *mut I;
    let end = bytes.as_ptr().add(bytes.len()) as *const [u8; SIZE];

    while read != end {
        buf = ptr::read(read);
        ptr::write(write, convert(buf));
        read = read.add(1);
        write = write.add(1);
    }

    let mut me = ManuallyDrop::new(bytes);

    let ptr = me.as_mut_ptr() as *mut I;
    let length = me.len();
    let capacity = me.capacity();

    Vec::from_raw_parts(ptr, length / SIZE, capacity / SIZE)
}
