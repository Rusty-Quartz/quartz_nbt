#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![feature(or_patterns)]

//! Provides support for encoding and decoding Minecraft's NBT format. This crate supports both
//! zlib and gz compression, and also provides tools for converting NBT data to stringified NBT
//! (SNBT) data and vice versa.

mod repr;
mod tag;

/// Contains utilities for reading NBT data.
pub mod read;
/// Provides support for SNBT parsing.
pub mod snbt;
/// Contains utilities for writing NBT data.
pub mod write;

pub use repr::*;
pub use tag::*;
