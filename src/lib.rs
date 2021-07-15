#![deny(rust_2018_idioms)]
#![warn(missing_debug_implementations, missing_docs)]

/*!
Provides support for encoding and decoding Minecraft's NBT format. This crate supports both
zlib and gz compression, and also provides tools for converting NBT data to stringified NBT
(SNBT) and vice versa.

# Basic Usage

The basic unit of NBT data is the [`NbtTag`]. Larger data structures are
represented through a tree of compounds (hash maps) and lists (vecs) of NBT tags.

## Creating NBT Data

```
# use quartz_nbt::*;
let mut compound = NbtCompound::new();
compound.insert("foo", 123);
compound.insert("bar", -3.6f32);

let mut list = NbtList::with_capacity(3);
(1i64..=3).for_each(|x| list.push(x));
compound.insert("list", list);

*compound.get_mut::<_, &mut i32>("foo").unwrap() += 1;

assert!(matches!(compound.get::<_, i32>("foo"), Ok(124)));
assert!(compound.get::<_, f64>("bar").is_err());
assert!(compound.get::<_, &NbtTag>("list").is_ok());
```

## Reading and Writing NBT

```
# use quartz_nbt::*;
use quartz_nbt::io::{self, Flavor};
use std::io::Cursor;

let mut compound = NbtCompound::new();
compound.insert("foo", 123);
compound.insert("bar", -3.6f32);

let mut binary: Vec<u8> = Vec::new();
io::write_nbt(&mut binary, Some("root-tag"), &compound, Flavor::Uncompressed);

let read_compound = io::read_nbt(&mut Cursor::new(binary), Flavor::Uncompressed).unwrap();
assert_eq!(read_compound.1, "root-tag"); // The root tag's name is generally unused
assert_eq!(read_compound.0, compound);
```

# Querying Tags

Generics are used to make the tag querying process as seamless as possible, however this
allows for two types of errors to occur: missing tags (invalid key or index), and tag type
mismatches. Thus, methods that would normally return an [`Option`](Option) in `std` collection
equivalents return a [`Result`](Result) in this crate.

An error converting NBT tags directly into unwrapped values via [`TryFrom`](std::convert::TryFrom)
and [`TryInto`](std::convert::TryInto) is represented by an [`NbtStructureError`](crate::NbtStructureError).
An error querying an [`NbtCompound`] or [`NbtList`] is represented by an [`NbtReprError`](crate::NbtReprError),
which is short for "NBT representation error." See the error's documentation for details.

```
# use quartz_nbt::*;
use std::convert::TryFrom;

let tag1: NbtTag = vec![1i8, 2, 3].into();
let tag2: NbtTag = "abcde".into();

assert_eq!(Vec::<i8>::try_from(tag1).unwrap(), vec![1i8, 2, 3]);
assert!(i16::try_from(tag2).is_err()); // Type mismatch
```

```
# use quartz_nbt::*;
let mut compound = NbtCompound::new();
compound.insert("foo", 123);
compound.insert("bar", -3.6f32);

assert!(compound.get::<_, i32>("fooz").is_err()); // Missing tag
assert!(compound.get::<_, i32>("bar").is_err()); // Type mismatch
```

# Collection Types and Iteration

The [`NbtCompound`] and [`NbtList`] types are wrappers around [`HashMap`](std::collections::HashMap)s
and [`Vec`](Vec)s respectively. Because [`NbtTag`]s obscure the type of data actually stored,
these wrappers provide utilities for unpacking tags into concrete types. If greater functionality
is required, then the internal collection managed by these wrappers can be accessed through
calls to [`as_ref`](std::convert::AsRef::as_ref) and [`as_mut`](std::convert::AsMut::as_mut).

## Lists

Minecraft's NBT specification currently has special tags for arrays (or [`Vec`](Vec)s in rust)
of `i8`, `i32`, and `i64`. Thus, vecs of these types can be directly converted into [`NbtTag`]s.
All other NBT-compatible types must be stored in an [`NbtList`].

Obtaining the aforementioned special list types can be done through a regular query.
```
# use quartz_nbt::*;
let mut compound = NbtCompound::new();
compound.insert("list", vec![10i32, 20, 30]);

compound.get_mut::<_, &mut [i32]>("list")
    .unwrap()
    .iter_mut()
    .for_each(|x| *x /= 10);

let list = compound.get::<_, &[i32]>("list");
assert!(list.is_ok());
assert_eq!(list.unwrap(), [1i32, 2, 3].as_ref());
```

Utility methods are provided for NBT lists to iterate over unpacked values. See
[`iter_map`](crate::NbtList::iter_map) and [`iter_mut_map`](crate::NbtList::iter_mut_map).
```
# use quartz_nbt::*;
let mut list = NbtList::new();
list.push("abc");
list.push("ijk");
list.push("xyz");

list.iter_mut_map::<&mut String>()
    .for_each(|s| s.unwrap().push('!'));

let mut iter = list.iter_map::<&str>();
assert!(matches!(iter.next(), Some(Ok("abc!"))));
assert!(matches!(iter.next(), Some(Ok("ijk!"))));
assert!(matches!(iter.next(), Some(Ok("xyz!"))));
assert!(matches!(iter.next(), None));
```

NBT lists can be created by cloning data from an iterator (or something which can be
converted into an iterator) via [`clone_from`](crate::NbtList::clone_from).
```
# use quartz_nbt::*;
let mut list1 = NbtList::new();
list1.push("abc");
list1.push("ijk");
list1.push("xyz");

let list2 = NbtList::clone_from(&["abc", "ijk", "xyz"]);

assert_eq!(list1, list2);
```

## Compounds

[`NbtCompound`]s have the same set of utility functions as [`NbtList`]s, except for the
obvious fact that compounds use string keys instead of indices. Similar to lists, compounds
have [`iter_map`](crate::NbtCompound::iter_map) and [`iter_mut_map`](crate::NbtCompound::iter_mut_map)
utility functions, as well as a [`clone_from`](crate::NbtCompound::clone_from) constructor.
See the documentation for more details.

# Stringified NBT (SNBT)

Minecraft also contains a string encoding of NBT data called SNBT. This encoding is basically an
extension of JSON with stricter types and looser rules regarding string quotation. See the
[`snbt`](crate::snbt) module documentation for more details.

```
# use quartz_nbt::*;
use quartz_nbt::snbt;

let tag: NbtTag = vec![10i8, 15, 20].into();
assert_eq!(tag.to_snbt(), "[B;10,15,20]");

let mut compound = NbtCompound::new();
compound.insert("short", -10i16);
compound.insert("string", "fizzbuzz");
compound.insert("array", vec![1i64, 1, 2, 3, 5]);

const SNBT: &str = "{short: -10s, string: fizzbuzz, array: [L; 1, 1, 2, 3, 5]}";

assert_eq!(compound, snbt::parse(SNBT).unwrap());
```

# NBT Representation

The [`NbtRepr`] trait allows for custom types to be convertible into [`NbtTag`]s by defining
methods for writing and reading to and from an [`NbtCompound`].

```
# use quartz_nbt::*;
#[derive(Debug, PartialEq, Eq)]
struct Example {
    name: String,
    value: i32
}

impl NbtRepr for Example {
    fn read_nbt(&mut self, nbt: &NbtCompound) -> Result<(), NbtReprError> {
        self.name = nbt.get::<_, &str>("name")?.to_owned();
        self.value = nbt.get("value")?;
        Ok(())
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.insert("name", &self.name);
        nbt.insert("value", self.value);
    }
}

let ex1 = Example {
    name: "foo".to_owned(),
    value: 10
};

let mut nbt = NbtCompound::new();
nbt.insert("name", "foo");
nbt.insert("value", 10);

let mut ex2 = Example {
    name: "".to_owned(),
    value: 0
};
ex2.read_nbt(&nbt);

assert_eq!(ex1.to_nbt(), nbt);
assert_eq!(ex1, ex2);
```

Currently, implementing this trait only allows for basic conversion into [`NbtTag`]s and construction
of compounds and lists via the `clone_from_repr` methods in each. In the future, we plan to create
a derive macro for this trait as well as to more thoroughly integrate its use into this crate.

[`NbtCompound`]: crate::NbtCompound
[`NbtList`]: crate::NbtList
[`NbtRepr`]: crate::NbtRepr
[`NbtTag`]: crate::NbtTag
*/

/// Provides efficient serializer and deserializer implementations for arbitrary NBT tag trees. The
/// functions in this module should be used for serializing and deserializing [`NbtCompound`]s
/// over the utilities provided by serde.
///
/// [`NbtCompound`]: crate::NbtCompound
pub mod io;
mod raw;
mod repr;
/// When the `serde` feature is enabled, this module provides `Serializer` and `Deserializer`
/// implementations to link this crate into the serde data model.
#[cfg(feature = "serde")]
#[allow(missing_debug_implementations)]
pub mod serde;
mod tag;

/// Provides support for parsing stringified NBT data.
///
/// SNBT is essentially an extension of JSON. It uses the same overarching syntax with some changes
/// to enforce stronger types.
///
/// # Numbers
///
/// Numbers in SNBT generally have a single-character suffix specifying their type (with `i32` and
/// `f64` being exceptions). If a number without a decimal point is encountered without a type
/// specifier, then the parser assumes it is an int. Likewise, if a number with a decimal point
/// but no type specifier is encountered, then it is assumed to be a double. Note that the type
/// specifier for doubles (`D` or `d`) is optional, however the integer type specifier (`I` or `i`)
/// is reserved for arrays and cannot be appended to an integer. Examples are shown below:
///  - Byte (`i8`): `2B`, `-3b`
///  - Short (`i16`): `17S`, `-1024s`
///  - Int (`i32`): `123`
///  - Long (`i64`): `43046721L`
///  - Float (`f32`): `3.141F`, `0.0f`
///  - Double (`f64`): `18932.214`, `10.2D`
///
/// Booleans are encoded as bytes, so `0b` represents `false` and `1b` (or any non-zero byte value)
/// represents `true`.
///
/// # Strings
///
/// SNBT treats any sequence of unicode characters not representing another token to be a string. For this
/// reason, strings are not required to be in quotes, however they can optionally be enclosed
/// in either single or double quotes. In other words, `foo` is equivalent to `"foo"` and `'foo'`, and
/// `"\"quoted\""` is equivalent to `'"quoted"'`. Although Minecraft's parser discourages the use of
/// whitespace in SNBT, this parser implementation is fully capable of handling it.
/// The way whitespace is ignored is by trimming strings, so if leading
/// or trailing whitespace is required, then the string must be enclosed in quotes.
///
/// SNBT strings also support several escape sequences:
///  - `\'`, `\"`, `\\`: copies the second character verbatim
///  - `\n`: new line
///  - `\r`: carriage return
///  - `\t`: tab
///  - `\uXXXX`: an unsigned hex number representing a unicode character
///
/// These escape sequences are only applied if a string is surrounded by quotes. An unquoted escape
/// sequence will be taken verbatim.
///
/// # Arrays and Lists
///
/// There are three array types supported by Minecraft's NBT formal: byte arrays, int arrays, and
/// long arrays. To differentiate an [`NbtList`](crate::NbtList) from an array, arrays start with
/// its values' type specifier followed by a semicolon. For example, an empty int array is denoted
/// by `[I;]`, and an example of a long array is `[L; -1, -2, -3]`. It is not necessary to put a
/// type specifier on the elements of an array, however the array must be homogenously typed, meaning
/// each element is of the same type.
///
/// NBT lists also use the square bracket syntax, however they do not contain a type specifier.
/// For example, a list of strings may look like `[foo, bar, baz]`. NBT lists, even though they
/// theoretically can contain multiple different types, must also be homogenous. The parser will
/// throw an error if the list is non-homogenous. Note that it is necessary to include the type specifier
/// for each element in an NBT list where applicable.
///
/// # Compounds
///
/// All valid SNBT strings have a compound as the root tag. Compounds, like JSON objects, follow the
/// syntax of `{key: value, ...}`. Compounds must have every key be a string, but its values do not have
/// to be homogenously typed. Whitespace is allowed to make compounds more readable, however one should
/// refer to the section on strings to avoid unexpected elisions.
pub mod snbt;

pub use repr::*;
pub use tag::*;

/// A utility macro for constructing `NbtCompound`s.
///
/// With exceptions for arrays and compounds, all keys and values must be well-formed rust
/// expressions. The benefit of this is that local variables can be included in the generated
/// compound.
/// ```
/// # use quartz_nbt::NbtCompound;
/// let product = 87235i32 * 932i32;
///
/// let compound = quartz_nbt::compound! {
///     "product": product,
///     "foo": "bar"
/// };
///
/// let mut manual_compound = NbtCompound::new();
/// manual_compound.insert("product", 81303020i32);
/// manual_compound.insert("foo", "bar");
///
/// assert_eq!(compound, manual_compound);
/// ```
///
/// Similar to SNBT, the specialized array types can be opted-into with a type specifier:
/// ```
/// # use quartz_nbt::NbtTag;
/// let compound = quartz_nbt::compound! {
///     "byte_array": [B; 1, 2, 3],
///     "int_array": [I; 4, 5, 6],
///     "long_array": [L; 7, 8, 9],
///     "tag_array": [10, 11, 12]
/// };
///
/// assert!(matches!(compound.get::<_, &NbtTag>("byte_array"), Ok(NbtTag::ByteArray(_))));
/// assert!(matches!(compound.get::<_, &NbtTag>("int_array"), Ok(NbtTag::IntArray(_))));
/// assert!(matches!(compound.get::<_, &NbtTag>("long_array"), Ok(NbtTag::LongArray(_))));
/// assert!(matches!(compound.get::<_, &NbtTag>("tag_array"), Ok(NbtTag::List(_))));
///
/// assert_eq!(
///     compound.get::<_, &[i64]>("long_array")
///         .unwrap()
///         .iter()
///         .copied()
///         .sum::<i64>(),
///     24
/// );
/// ```
///
/// Just like in JSON or SNBT, compounds are enclosed by braces:
/// ```
/// # use quartz_nbt::{NbtCompound, NbtList};
/// let compound = quartz_nbt::compound! {
///     "nested": {
///         "a": [I;],
///         "b": []
///     }
/// };
///
/// let mut outer = NbtCompound::new();
/// let mut nested = NbtCompound::new();
/// nested.insert("a", Vec::<i32>::new());
/// nested.insert("b", NbtList::new());
/// outer.insert("nested", nested);
///
/// assert_eq!(compound, outer);
/// ```
pub use quartz_nbt_macros::compound;
