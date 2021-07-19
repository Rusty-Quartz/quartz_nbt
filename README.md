# quartz_nbt

Provides support for encoding and decoding Minecraft's NBT format. This crate supports both
zlib and gz compression, and also provides tools for converting NBT data to stringified NBT
(SNBT) and vice versa.

This crate is the standalone NBT crate for [Quartz](https://github.com/Rusty-Quartz/Quartz),
a Minecraft server implementation in Rust.

# Usage

Use the most recent version of this crate when adding it to your dependencies as shown below.
```toml
[dependencies]
quartz_nbt = "0.2.2"
```
View the documentation [here](https://docs.rs/quartz_nbt) for examples.