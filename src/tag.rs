use crate::{
    raw,
    snbt::{self, SnbtError},
    NbtRepr,
    NbtReprError,
    NbtStructureError,
};
use std::{
    borrow::Borrow,
    collections::HashMap,
    convert::{AsMut, AsRef, TryFrom},
    fmt::{self, Debug, Display, Formatter},
    hash::Hash,
    ops::{Index, IndexMut},
    str::FromStr,
};

/// The generic NBT tag type, containing all supported tag variants which wrap around a corresponding
/// rust type.
///
/// This type will implement both `Serialize` and `Deserialize` when the serde feature is enabled,
/// however this type should still be read and written with the utilities in the [`io`] module when
/// possible if speed is the main priority. When linking into the serde ecosystem, we ensured that all
/// tag types would have their data inlined into the resulting NBT output of our Serializer. Because of
/// this, NBT tags are only compatible with self-describing formats, and also have slower deserialization
/// implementations due to this restriction.
///
/// [`io`]: crate::io
#[derive(Clone, PartialEq)]
pub enum NbtTag {
    /// A signed, one-byte integer.
    Byte(i8),
    /// A signed, two-byte integer.
    Short(i16),
    /// A signed, four-byte integer.
    Int(i32),
    /// A signed, eight-byte integer.
    Long(i64),
    /// A 32-bit floating point value.
    Float(f32),
    /// A 64-bit floating point value.
    Double(f64),
    /// An array (vec) of one-byte integers. Minecraft treats this as an array of signed bytes.
    ByteArray(Vec<i8>),
    /// A UTF-8 string.
    String(String),
    /// An NBT tag list.
    List(NbtList),
    /// An NBT tag compound.
    Compound(NbtCompound),
    /// An array (vec) of signed, four-byte integers.
    IntArray(Vec<i32>),
    /// An array (vec) of signed, eight-byte integers.
    LongArray(Vec<i64>),
}

impl NbtTag {
    /// Returns the single character denoting this tag's type, or an empty string if this tag type has
    /// no type specifier.
    ///
    /// # Examples
    ///
    /// ```
    /// # use quartz_nbt::NbtTag;
    /// assert_eq!(NbtTag::Long(10).type_specifier(), "L");
    /// assert_eq!(NbtTag::String(String::new()).type_specifier(), "");
    ///
    /// // Note that while integers do not require a type specifier, this method will still return "I"
    /// assert_eq!(NbtTag::Int(-10).type_specifier(), "I");
    /// ```
    pub fn type_specifier(&self) -> &str {
        match self {
            NbtTag::Byte(_) => "B",
            NbtTag::Short(_) => "S",
            NbtTag::Int(_) => "I",
            NbtTag::Long(_) => "L",
            NbtTag::Float(_) => "F",
            NbtTag::Double(_) => "D",
            NbtTag::ByteArray(_) => "B",
            NbtTag::IntArray(_) => "I",
            NbtTag::LongArray(_) => "L",
            _ => "",
        }
    }

    pub(crate) fn tag_name(&self) -> &'static str {
        match self {
            NbtTag::Byte(_) => "Byte",
            NbtTag::Short(_) => "Short",
            NbtTag::Int(_) => "Int",
            NbtTag::Long(_) => "Long",
            NbtTag::Float(_) => "Float",
            NbtTag::Double(_) => "Double",
            NbtTag::String(_) => "String",
            NbtTag::ByteArray(_) => "ByteArray",
            NbtTag::IntArray(_) => "IntArray",
            NbtTag::LongArray(_) => "LongArray",
            NbtTag::Compound(_) => "Compound",
            NbtTag::List(_) => "List",
        }
    }

    /// Converts this NBT tag into a valid, parsable SNBT string with no extraneous spacing. This method should
    /// not be used to generate user-facing text, rather `to_component` should be used instead.
    ///
    /// # Examples
    ///
    /// Simple primitive conversion:
    ///
    /// ```
    /// # use quartz_nbt::NbtTag;
    /// assert_eq!(NbtTag::Byte(5).to_snbt(), "5B");
    /// assert_eq!(NbtTag::String("\"Quoted text\"".to_owned()).to_snbt(), "'\"Quoted text\"'");
    /// ```
    ///
    /// More complex tag conversion:
    ///
    /// ```
    /// # use quartz_nbt::*;
    /// let mut compound = NbtCompound::new();
    /// compound.insert("foo".to_owned(), vec![-1_i64, -3_i64, -5_i64]);
    /// assert_eq!(NbtTag::Compound(compound).to_snbt(), "{foo:[L;-1,-3,-5]}");
    /// ```
    pub fn to_snbt(&self) -> String {
        macro_rules! list_to_string {
            ($list:expr) => {
                format!(
                    "[{};{}]",
                    self.type_specifier(),
                    $list
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(",")
                )
            };
        }

        match self {
            NbtTag::Byte(value) => format!("{}{}", value, self.type_specifier()),
            NbtTag::Short(value) => format!("{}{}", value, self.type_specifier()),
            NbtTag::Int(value) => format!("{}", value),
            NbtTag::Long(value) => format!("{}{}", value, self.type_specifier()),
            NbtTag::Float(value) => format!("{}{}", value, self.type_specifier()),
            NbtTag::Double(value) => format!("{}{}", value, self.type_specifier()),
            NbtTag::ByteArray(value) => list_to_string!(value),
            NbtTag::String(value) => Self::string_to_snbt(value),
            NbtTag::List(value) => value.to_snbt(),
            NbtTag::Compound(value) => value.to_snbt(),
            NbtTag::IntArray(value) => list_to_string!(value),
            NbtTag::LongArray(value) => list_to_string!(value),
        }
    }

    /// Returns whether or not the given string needs to be quoted due to non-alphanumeric or otherwise
    /// non-standard characters.
    pub fn should_quote(string: &str) -> bool {
        for ch in string.chars() {
            if ch == ':'
                || ch == ','
                || ch == '"'
                || ch == '\''
                || ch == '{'
                || ch == '}'
                || ch == '['
                || ch == ']'
            {
                return true;
            }
        }

        false
    }

    /// Wraps the given string in quotes and escapes any quotes contained in the original string.
    pub fn string_to_snbt(string: &str) -> String {
        // Determine the best option for the surrounding quotes to minimize escape sequences
        let surrounding: char;
        if string.contains("\"") {
            surrounding = '\'';
        } else {
            surrounding = '"';
        }

        let mut snbt_string = String::with_capacity(2 + string.len());
        snbt_string.push(surrounding);

        // Construct the string accounting for escape sequences
        for ch in string.chars() {
            if ch == surrounding || ch == '\\' {
                snbt_string.push('\\');
            }
            snbt_string.push(ch);
        }

        snbt_string.push(surrounding);
        snbt_string
    }
}

impl Display for NbtTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.to_snbt(), f)
    }
}

impl Debug for NbtTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.to_snbt(), f)
    }
}

// Implement the from trait for all the tag's internal types
macro_rules! tag_from {
    ($($type:ty, $tag:ident);*) => {
        $(
            impl From<$type> for NbtTag {
                fn from(value: $type) -> NbtTag {
                    NbtTag::$tag(value)
                }
            }
        )*
    };
}

tag_from!(
    i8, Byte;
    i16, Short;
    i32, Int;
    i64, Long;
    f32, Float;
    f64, Double;
    Vec<i8>, ByteArray;
    String, String;
    NbtList, List;
    NbtCompound, Compound;
    Vec<i32>, IntArray;
    Vec<i64>, LongArray
);

impl From<&str> for NbtTag {
    fn from(value: &str) -> NbtTag {
        NbtTag::String(value.to_owned())
    }
}

impl From<&String> for NbtTag {
    fn from(value: &String) -> NbtTag {
        NbtTag::String(value.clone())
    }
}

impl From<bool> for NbtTag {
    fn from(value: bool) -> NbtTag {
        NbtTag::Byte(if value { 1 } else { 0 })
    }
}

impl From<u8> for NbtTag {
    fn from(value: u8) -> Self {
        NbtTag::Byte(value as i8)
    }
}

impl From<Vec<u8>> for NbtTag {
    fn from(value: Vec<u8>) -> Self {
        NbtTag::ByteArray(raw::cast_byte_buf_to_signed(value))
    }
}

impl<T: NbtRepr> From<T> for NbtTag {
    fn from(x: T) -> Self {
        NbtTag::Compound(x.to_nbt())
    }
}

macro_rules! prim_from_tag {
    ($($type:ty, $tag:ident);*) => {
        $(
            impl TryFrom<&NbtTag> for $type {
                type Error = NbtStructureError;

                fn try_from(tag: &NbtTag) -> Result<Self, Self::Error> {
                    if let NbtTag::$tag(value) = tag {
                        Ok(*value)
                    } else {
                        Err(NbtStructureError::type_mismatch(stringify!($tag), tag.tag_name()))
                    }
                }
            }
        )*
    };
}

prim_from_tag!(
    i8, Byte;
    i16, Short;
    i32, Int;
    i64, Long;
    f32, Float;
    f64, Double
);

impl TryFrom<&NbtTag> for bool {
    type Error = NbtStructureError;

    fn try_from(tag: &NbtTag) -> Result<Self, Self::Error> {
        match tag {
            &NbtTag::Byte(value) => Ok(value != 0),
            &NbtTag::Short(value) => Ok(value != 0),
            &NbtTag::Int(value) => Ok(value != 0),
            &NbtTag::Long(value) => Ok(value != 0),
            _ => Err(NbtStructureError::type_mismatch(
                "Byte, Short, Int, or Long",
                tag.tag_name(),
            )),
        }
    }
}

impl TryFrom<&NbtTag> for u8 {
    type Error = NbtStructureError;

    fn try_from(tag: &NbtTag) -> Result<Self, Self::Error> {
        match tag {
            &NbtTag::Byte(value) => Ok(value as u8),
            _ => Err(NbtStructureError::type_mismatch("Byte", tag.tag_name())),
        }
    }
}

macro_rules! ref_from_tag {
    ($($type:ty, $tag:ident);*) => {
        $(
            impl<'a> TryFrom<&'a NbtTag> for &'a $type {
                type Error = NbtStructureError;

                fn try_from(tag: &'a NbtTag) -> Result<Self, Self::Error> {
                    if let NbtTag::$tag(value) = tag {
                        Ok(value)
                    } else {
                        Err(NbtStructureError::type_mismatch(stringify!($tag), tag.tag_name()))
                    }
                }
            }

            impl<'a> TryFrom<&'a mut NbtTag> for &'a mut $type {
                type Error = NbtStructureError;

                fn try_from(tag: &'a mut NbtTag) -> Result<Self, Self::Error> {
                    if let NbtTag::$tag(value) = tag {
                        Ok(value)
                    } else {
                        Err(NbtStructureError::type_mismatch(stringify!($tag), tag.tag_name()))
                    }
                }
            }
        )*
    };
}

ref_from_tag!(
    i8, Byte;
    i16, Short;
    i32, Int;
    i64, Long;
    f32, Float;
    f64, Double;
    Vec<i8>, ByteArray;
    [i8], ByteArray;
    String, String;
    str, String;
    NbtList, List;
    NbtCompound, Compound;
    Vec<i32>, IntArray;
    [i32], IntArray;
    Vec<i64>, LongArray;
    [i64], LongArray
);

impl<'a> TryFrom<&'a NbtTag> for &'a u8 {
    type Error = NbtStructureError;

    fn try_from(tag: &'a NbtTag) -> Result<Self, Self::Error> {
        if let NbtTag::Byte(value) = tag {
            Ok(unsafe { &*(value as *const i8 as *const u8) })
        } else {
            Err(NbtStructureError::type_mismatch("Byte", tag.tag_name()))
        }
    }
}

impl<'a> TryFrom<&'a NbtTag> for &'a [u8] {
    type Error = NbtStructureError;

    fn try_from(tag: &'a NbtTag) -> Result<Self, Self::Error> {
        if let NbtTag::ByteArray(value) = tag {
            Ok(raw::cast_bytes_to_unsigned(value.as_slice()))
        } else {
            Err(NbtStructureError::type_mismatch(
                "ByteArray",
                tag.tag_name(),
            ))
        }
    }
}

macro_rules! from_tag {
    ($($type:ty, $tag:ident);*) => {
        $(
            impl TryFrom<NbtTag> for $type {
                type Error = NbtStructureError;

                fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
                    if let NbtTag::$tag(value) = tag {
                        Ok(value)
                    } else {
                        Err(NbtStructureError::type_mismatch(stringify!($tag), tag.tag_name()))
                    }
                }
            }
        )*
    };
}

from_tag!(
    i8, Byte;
    i16, Short;
    i32, Int;
    i64, Long;
    f32, Float;
    f64, Double;
    Vec<i8>, ByteArray;
    String, String;
    NbtList, List;
    NbtCompound, Compound;
    Vec<i32>, IntArray;
    Vec<i64>, LongArray
);

impl TryFrom<NbtTag> for Vec<u8> {
    type Error = NbtStructureError;

    fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
        if let NbtTag::ByteArray(value) = tag {
            Ok(raw::cast_byte_buf_to_unsigned(value))
        } else {
            Err(NbtStructureError::type_mismatch(
                "ByteArray",
                tag.tag_name(),
            ))
        }
    }
}

/// The NBT tag list type which is essentially just a wrapper for a vec of NBT tags.
///
/// This type will implement both `Serialize` and `Deserialize` when the serde feature is enabled,
/// however this type should still be read and written with the utilities in the [`io`] module when
/// possible if speed is the main priority. See [`NbtTag`] for more details.
///
/// [`io`]: crate::io
/// [`NbtTag`]: crate::NbtTag
#[repr(transparent)]
#[derive(Clone, PartialEq)]
pub struct NbtList(pub(crate) Vec<NbtTag>);

impl NbtList {
    /// Returns a new NBT tag list with an empty internal vec.
    pub const fn new() -> Self {
        NbtList(Vec::new())
    }

    /// Returns a mutable reference to the internal vector of this NBT list.
    pub fn inner_mut(&mut self) -> &mut Vec<NbtTag> {
        &mut self.0
    }

    /// Returns the internal vector of this NBT list.
    pub fn into_inner(self) -> Vec<NbtTag> {
        self.0
    }

    /// Returns a new NBT tag list with the given initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        NbtList(Vec::with_capacity(capacity))
    }

    /// Clones the data in the given list and converts it into an [`NbtList`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use quartz_nbt::NbtList;
    /// let list: Vec<i32> = vec![1, 2, 3];
    /// let nbt_list = NbtList::clone_from(&list);
    /// assert_eq!(nbt_list.iter_map::<i32>().flatten().collect::<Vec<i32>>(), list);
    /// ```
    ///
    /// [`NbtList`]: crate::tag::NbtList
    pub fn clone_from<'a, T, L>(list: L) -> Self
    where
        T: Clone + Into<NbtTag> + 'a,
        L: IntoIterator<Item = &'a T>,
    {
        NbtList(list.into_iter().map(|x| x.clone().into()).collect())
    }

    /// Creates an [`NbtList`] of [`NbtCompound`]s by mapping each element in the given list to its
    /// NBT representation.
    ///
    /// [`NbtCompound`]: crate::tag::NbtCompound
    /// [`NbtList`]: crate::tag::NbtList
    pub fn clone_repr_from<'a, T, L>(list: L) -> Self
    where
        T: NbtRepr + 'a,
        L: IntoIterator<Item = &'a T>,
    {
        NbtList(list.into_iter().map(|x| x.to_nbt().into()).collect())
    }

    /// Iterates over this tag list, converting each tag reference into the specified type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use quartz_nbt::{NbtList, NbtStructureError};
    /// let mut list = NbtList::new();
    /// list.push(0i32);
    /// list.push(1i32);
    /// list.push(2.0f64);
    ///
    /// let mut iter = list.iter_map::<i32>();
    /// assert!(matches!(iter.next(), Some(Ok(0i32))));
    /// assert!(matches!(iter.next(), Some(Ok(1i32))));
    /// assert!(matches!(iter.next(), Some(Err(..)))); // Type mismatch
    /// assert!(iter.next().is_none());
    /// ```
    pub fn iter_map<'a, T: TryFrom<&'a NbtTag>>(
        &'a self,
    ) -> impl Iterator<Item = Result<T, <T as TryFrom<&'a NbtTag>>::Error>> + 'a {
        self.0.iter().map(|tag| T::try_from(tag))
    }

    /// Iterates over mutable references to the tags in this list, converting each tag reference into
    /// the specified type. See [`iter_map`](crate::tag::NbtList::iter_map) for usage details.
    pub fn iter_mut_map<'a, T: TryFrom<&'a mut NbtTag>>(
        &'a mut self,
    ) -> impl Iterator<Item = Result<T, <T as TryFrom<&'a mut NbtTag>>::Error>> + 'a {
        self.0.iter_mut().map(|tag| T::try_from(tag))
    }

    /// Converts this tag list to a valid SNBT string.
    pub fn to_snbt(&self) -> String {
        let mut snbt_list = String::with_capacity(2);
        snbt_list.push('[');
        snbt_list.push_str(
            &self
                .as_ref()
                .iter()
                .map(|tag| tag.to_snbt())
                .collect::<Vec<String>>()
                .join(","),
        );
        snbt_list.push(']');
        snbt_list
    }

    /// Returns the length of this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if this tag list has a length of zero, false otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the value of the tag at the given index, or an error if the index is out of bounds or the
    /// the tag type does not match the type specified. This method should be used for obtaining primitives
    /// and shared references to lists and compounds.
    ///
    /// ```
    /// # use quartz_nbt::*;
    /// let mut list = NbtList::clone_from(&vec![1i32, 2, 3]);
    ///
    /// assert!(matches!(list.get::<i32>(0), Ok(1)));
    /// assert!(list.get::<f64>(0).is_err()); // Type mismatch
    /// assert!(list.get::<i32>(10).is_err()); // Invalid index
    /// ```
    pub fn get<'a, T>(&'a self, index: usize) -> Result<T, NbtReprError>
    where
        T: TryFrom<&'a NbtTag>,
        T::Error: Into<anyhow::Error>,
    {
        T::try_from(
            self.0
                .get(index)
                .ok_or(NbtStructureError::invalid_index(index, self.len()))?,
        )
        .map_err(NbtReprError::from_any)
    }

    /// Returns a mutable reference to the tag at the given index, or an error if the index is out of bounds or
    /// tag type does not match the type specified. This method should be used for obtaining mutable references
    /// to elements.
    ///
    /// ```
    /// # use quartz_nbt::*;
    /// let mut list = NbtList::clone_from(&vec![1i32, 2, 3]);
    ///
    /// *list.get_mut::<&mut i32>(0).unwrap() += 1;
    ///
    /// assert!(matches!(list.get::<i32>(0), Ok(2)));
    /// assert!(list.get::<f64>(0).is_err()); // Type mismatch
    /// assert!(list.get::<i32>(10).is_err()); // Invalid index
    /// ```
    pub fn get_mut<'a, T>(&'a mut self, index: usize) -> Result<T, NbtReprError>
    where
        T: TryFrom<&'a mut NbtTag>,
        T::Error: Into<anyhow::Error>,
    {
        let len = self.len();
        T::try_from(
            self.0
                .get_mut(index)
                .ok_or(NbtStructureError::invalid_index(index, len))?,
        )
        .map_err(NbtReprError::from_any)
    }

    /// Pushes the given value to the back of the list after wrapping it in an `NbtTag`.
    ///
    /// ```
    /// # use quartz_nbt::*;
    /// let mut list = NbtList::new();
    ///
    /// list.push(10i32);
    ///
    /// assert!(matches!(list.get::<i32>(0), Ok(10)));
    /// assert!(list.get::<f64>(0).is_err()); // Type mismatch
    /// assert!(list.get::<i32>(10).is_err()); // Invalid index
    /// ```
    pub fn push<T: Into<NbtTag>>(&mut self, value: T) {
        self.0.push(value.into());
    }
}

impl<T: Into<NbtTag>> From<Vec<T>> for NbtList {
    fn from(list: Vec<T>) -> Self {
        NbtList(list.into_iter().map(|x| x.into()).collect())
    }
}

impl AsRef<[NbtTag]> for NbtList {
    fn as_ref(&self) -> &[NbtTag] {
        &self.0
    }
}

impl AsMut<[NbtTag]> for NbtList {
    fn as_mut(&mut self) -> &mut [NbtTag] {
        &mut self.0
    }
}

impl Display for NbtList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.to_snbt(), f)
    }
}

impl Debug for NbtList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.to_snbt(), f)
    }
}

impl Index<usize> for NbtList {
    type Output = NbtTag;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for NbtList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

/// The NBT tag compound type which is essentially just a wrapper for a hash map of string keys
/// to tag values.
///
/// This type will implement both `Serialize` and `Deserialize` when the serde feature is enabled,
/// however this type should still be read and written with the utilities in the [`io`] module when
/// possible if speed is the main priority. See [`NbtTag`] for more details.
///
/// [`NbtTag`]: crate::NbtTag
/// [`io`]: crate::io
#[repr(transparent)]
#[derive(Clone, PartialEq)]
pub struct NbtCompound(pub(crate) HashMap<String, NbtTag>);

impl NbtCompound {
    /// Returns a new NBT tag compound with an empty internal hash map.
    pub fn new() -> Self {
        NbtCompound(HashMap::new())
    }

    /// Returns a reference to the internal hash map of this compound.
    pub fn inner(&self) -> &HashMap<String, NbtTag> {
        &self.0
    }

    /// Returns a mutable reference to the internal hash map of this compound.
    pub fn inner_mut(&mut self) -> &mut HashMap<String, NbtTag> {
        &mut self.0
    }

    /// Returns the internal hash map of this NBT compound.
    pub fn into_inner(self) -> HashMap<String, NbtTag> {
        self.0
    }

    /// Returns a new NBT tag compound with the given initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        NbtCompound(HashMap::with_capacity(capacity))
    }

    /// Clones the data in the given map and converts it into an [`NbtCompound`](crate::tag::NbtCompound).
    ///
    /// # Examples
    ///
    /// ```
    /// # use quartz_nbt::NbtCompound;
    /// # use std::collections::HashMap;
    /// let mut map = HashMap::new();
    /// map.insert("foo", 10i32);
    /// map.insert("bar", -5i32);
    ///
    /// let compound = NbtCompound::clone_from(&map);
    /// assert_eq!(
    ///     compound.get::<_, i32>("foo").unwrap() + compound.get::<_, i32>("bar").unwrap(),
    ///     5i32
    /// );
    /// ```
    pub fn clone_from<'a, K, V, M>(map: &'a M) -> Self
    where
        K: Clone + Into<String> + 'a,
        V: Clone + Into<NbtTag> + 'a,
        &'a M: IntoIterator<Item = (&'a K, &'a V)>,
    {
        NbtCompound(
            map.into_iter()
                .map(|(key, value)| (key.clone().into(), value.clone().into()))
                .collect(),
        )
    }

    /// Creates an [`NbtCompound`] of [`NbtCompound`]s by mapping each element in the given map to its
    /// NBT representation.
    ///
    /// [`NbtCompound`]: crate::tag::NbtCompound
    pub fn clone_repr_from<'a, K, V, M>(map: &'a M) -> Self
    where
        K: Clone + Into<String> + 'a,
        V: NbtRepr + 'a,
        &'a M: IntoIterator<Item = (&'a K, &'a V)>,
    {
        NbtCompound(
            map.into_iter()
                .map(|(key, value)| (key.clone().into(), value.to_nbt().into()))
                .collect(),
        )
    }

    /// Iterates over this tag compound, converting each tag reference into the specified type. Each key is
    /// paired with the result of the attempted conversion into the specified type. The iterator will not
    /// terminate even if some conversions fail.
    pub fn iter_map<'a, T: TryFrom<&'a NbtTag>>(
        &'a self,
    ) -> impl Iterator<Item = (&'a str, Result<T, <T as TryFrom<&'a NbtTag>>::Error>)> + 'a {
        self.0
            .iter()
            .map(|(key, tag)| (key.as_str(), T::try_from(tag)))
    }

    /// Iterates over this tag compound, converting each mutable tag reference into the specified type. See
    /// [`iter_map`](crate::tag::NbtCompound::iter_map) for details.
    pub fn iter_mut_map<'a, T: TryFrom<&'a mut NbtTag>>(
        &'a mut self,
    ) -> impl Iterator<Item = (&'a str, Result<T, <T as TryFrom<&'a mut NbtTag>>::Error>)> + 'a
    {
        self.0
            .iter_mut()
            .map(|(key, tag)| (key.as_str(), T::try_from(tag)))
    }

    /// Converts this tag compound into a valid SNBT string.
    pub fn to_snbt(&self) -> String {
        let mut snbt_compound = String::with_capacity(2);
        snbt_compound.push('{');
        snbt_compound.push_str(
            &self
                .0
                .iter()
                .map(|(key, tag)| {
                    if NbtTag::should_quote(key) {
                        format!("{}:{}", NbtTag::string_to_snbt(key), tag.to_snbt())
                    } else {
                        format!("{}:{}", key, tag.to_snbt())
                    }
                })
                .collect::<Vec<String>>()
                .join(","),
        );
        snbt_compound.push('}');
        snbt_compound
    }

    /// Returns the number of tags in this compound.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the length of this compound is zero, false otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the value of the tag with the given name, or an error if no tag exists with the given name
    /// or specified type. This method should be used to obtain primitives as well as shared references to
    /// lists and compounds.
    ///
    /// ```
    /// # use quartz_nbt::*;
    /// let mut compound = NbtCompound::new();
    /// compound.insert("test", 1.0f64);
    ///
    /// assert!((compound.get::<_, f64>("test").unwrap() - 1.0f64).abs() < 1e-5);
    /// assert!(compound.get::<_, i32>("test").is_err()); // Type mismatch
    /// assert!(compound.get::<_, f64>("foo").is_err()); // Missing tag
    /// ```
    pub fn get<'a, 'b, K, T>(&'a self, name: &'b K) -> Result<T, NbtReprError>
    where
        String: Borrow<K>,
        K: Hash + Eq + ?Sized,
        &'b K: Into<String>,
        T: TryFrom<&'a NbtTag>,
        T::Error: Into<anyhow::Error>,
    {
        T::try_from(
            self.0
                .get(name)
                .ok_or(NbtStructureError::missing_tag(name))?,
        )
        .map_err(NbtReprError::from_any)
    }

    /// Returns the value of the tag with the given name, or an error if no tag exists with the given name
    /// or specified type. This method should be used to obtain mutable references to lists and compounds.
    ///
    /// ```
    /// # use quartz_nbt::*;
    /// let mut compound = NbtCompound::new();
    /// compound.insert("test", 1.0f64);
    ///
    /// *compound.get_mut::<_, &mut f64>("test").unwrap() *= 2.0;
    ///
    /// assert!((compound.get::<_, f64>("test").unwrap() - 2.0f64).abs() < 1e-5);
    /// assert!(compound.get::<_, i32>("test").is_err()); // Type mismatch
    /// assert!(compound.get::<_, f64>("foo").is_err()); // Missing tag
    /// ```
    pub fn get_mut<'a, 'b, K, T>(&'a mut self, name: &'b K) -> Result<T, NbtReprError>
    where
        String: Borrow<K>,
        K: Hash + Eq + ?Sized,
        &'b K: Into<String>,
        T: TryFrom<&'a mut NbtTag>,
        T::Error: Into<anyhow::Error>,
    {
        T::try_from(
            self.0
                .get_mut(name)
                .ok_or(NbtStructureError::missing_tag(name))?,
        )
        .map_err(NbtReprError::from_any)
    }

    /// Returns whether or not this compound has a tag with the given name.
    ///
    /// ```
    /// # use quartz_nbt::*;
    /// let mut compound = NbtCompound::new();
    /// compound.insert("test", 1.0f64);
    ///
    /// assert!(compound.contains_key("test"));
    /// assert!(!compound.contains_key("foo"));
    /// ```
    #[inline]
    pub fn contains_key<K>(&self, key: &K) -> bool
    where
        String: Borrow<K>,
        K: Hash + Eq + ?Sized,
    {
        self.0.contains_key(key)
    }

    /// Adds the given value to this compound with the given name after wrapping that value in an `NbtTag`.
    ///
    /// ```
    /// # use quartz_nbt::*;
    /// let mut compound = NbtCompound::new();
    /// compound.insert("test", 1.0f64);
    ///
    /// assert!((compound.get::<_, f64>("test").unwrap() - 1.0f64).abs() < 1e-5);
    /// ```
    pub fn insert<K: Into<String>, T: Into<NbtTag>>(&mut self, name: K, value: T) {
        self.0.insert(name.into(), value.into());
    }

    /// Parses a nbt compound from snbt
    ///
    /// # Example
    ///
    /// ```
    /// # use quartz_nbt::NbtCompound;
    /// let tag = NbtCompound::from_snbt(r#"{string:Stuff, list:[I;1,2,3,4,5]}"#).unwrap();
    /// assert!(matches!(tag.get::<_, &str>("string"), Ok("Stuff")));
    /// assert_eq!(tag.get::<_, &[i32]>("list").unwrap(), vec![1,2,3,4,5].as_slice());
    /// ```
    pub fn from_snbt(input: &str) -> Result<Self, SnbtError> {
        snbt::parse(input)
    }
}

impl FromStr for NbtCompound {
    type Err = SnbtError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_snbt(s)
    }
}

impl Display for NbtCompound {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.to_snbt(), f)
    }
}

impl Debug for NbtCompound {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.to_snbt(), f)
    }
}

#[cfg(feature = "serde")]
pub use serde_impl::*;

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use crate::serde::{Array, TypeHint};
    use serde::{
        de::{self, MapAccess, Visitor},
        Deserialize,
        Deserializer,
        Serialize,
        Serializer,
    };

    impl Serialize for NbtTag {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
            match self {
                &NbtTag::Byte(value) => serializer.serialize_i8(value),
                &NbtTag::Short(value) => serializer.serialize_i16(value),
                &NbtTag::Int(value) => serializer.serialize_i32(value),
                &NbtTag::Long(value) => serializer.serialize_i64(value),
                &NbtTag::Float(value) => serializer.serialize_f32(value),
                &NbtTag::Double(value) => serializer.serialize_f64(value),
                NbtTag::ByteArray(array) => Array::from(array).serialize(serializer),
                NbtTag::String(value) => serializer.serialize_str(value),
                NbtTag::List(list) => list.serialize(serializer),
                NbtTag::Compound(compound) => compound.serialize(serializer),
                NbtTag::IntArray(array) => Array::from(array).serialize(serializer),
                NbtTag::LongArray(array) => Array::from(array).serialize(serializer),
            }
        }
    }

    impl<'de> Deserialize<'de> for NbtTag {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> {
            deserializer.deserialize_any(NbtTagVisitor)
        }
    }

    struct NbtTagVisitor;

    impl<'de> Visitor<'de> for NbtTagVisitor {
        type Value = NbtTag;

        fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "a valid NBT type")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(NbtTag::Byte(if v { 1 } else { 0 }))
        }

        fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(NbtTag::Byte(v))
        }

        fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(NbtTag::Byte(v as i8))
        }

        fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(NbtTag::Short(v))
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(NbtTag::Int(v))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(NbtTag::Long(v))
        }

        fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(NbtTag::Float(v))
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(NbtTag::Double(v))
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where E: de::Error {
            self.visit_byte_buf(v.to_owned())
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(NbtTag::ByteArray(raw::cast_byte_buf_to_signed(v)))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(NbtTag::String(v.to_owned()))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(NbtTag::String(v))
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where A: MapAccess<'de> {
            let mut dest = match map.size_hint() {
                Some(hint) => HashMap::with_capacity(hint),
                None => HashMap::new(),
            };
            while let Some((key, tag)) = map.next_entry::<String, NbtTag>()? {
                dest.insert(key, tag);
            }
            Ok(NbtTag::Compound(NbtCompound(dest)))
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where A: de::SeqAccess<'de> {
            enum ArbitraryList {
                Byte(Vec<i8>),
                Int(Vec<i32>),
                Long(Vec<i64>),
                Tag(Vec<NbtTag>),
                Indeterminate,
            }

            let mut list = ArbitraryList::Indeterminate;

            fn init_vec<T>(element: T, size: Option<usize>) -> Vec<T> {
                match size {
                    Some(size) => {
                        // Add one because the size hint returns the remaining amount
                        let mut vec = Vec::with_capacity(1 + size);
                        vec.push(element);
                        vec
                    }
                    None => vec![element],
                }
            }

            while let Some(tag) = seq.next_element::<NbtTag>()? {
                match (tag, &mut list) {
                    (NbtTag::Byte(value), ArbitraryList::Byte(list)) => list.push(value),
                    (NbtTag::Int(value), ArbitraryList::Int(list)) => list.push(value),
                    (NbtTag::Long(value), ArbitraryList::Long(list)) => list.push(value),
                    (tag, ArbitraryList::Tag(list)) => list.push(tag),
                    (tag, list @ ArbitraryList::Indeterminate) => {
                        let size = seq.size_hint();
                        match tag {
                            NbtTag::Byte(value) =>
                                *list = ArbitraryList::Byte(init_vec(value, size)),
                            NbtTag::Int(value) => *list = ArbitraryList::Int(init_vec(value, size)),
                            NbtTag::Long(value) =>
                                *list = ArbitraryList::Long(init_vec(value, size)),
                            tag => *list = ArbitraryList::Tag(init_vec(tag, size)),
                        }
                    }
                    _ =>
                        return Err(de::Error::custom(
                            "tag type mismatch when deserializing array",
                        )),
                }
            }

            Ok(match list {
                ArbitraryList::Byte(list) => NbtTag::ByteArray(list),
                ArbitraryList::Int(list) => NbtTag::IntArray(list),
                ArbitraryList::Long(list) => NbtTag::LongArray(list),
                ArbitraryList::Tag(list) => NbtTag::List(NbtList(list)),
                // Try to acquire a type hint
                ArbitraryList::Indeterminate => match seq.next_element::<TypeHint>() {
                    Ok(Some(TypeHint { hint: Some(tag_id) })) => match tag_id {
                        0x7 => NbtTag::ByteArray(Vec::new()),
                        0xB => NbtTag::IntArray(Vec::new()),
                        0xC => NbtTag::LongArray(Vec::new()),
                        _ => NbtTag::List(NbtList::new()),
                    },
                    _ => NbtTag::List(NbtList::new()),
                },
            })
        }
    }

    impl Serialize for NbtList {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
            self.0.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for NbtList {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> {
            Ok(NbtList(Deserialize::deserialize(deserializer)?))
        }
    }

    impl Serialize for NbtCompound {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
            self.0.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for NbtCompound {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> {
            Ok(NbtCompound(Deserialize::deserialize(deserializer)?))
        }
    }
}
