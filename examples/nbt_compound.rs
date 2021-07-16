use quartz_nbt::{compound, NbtCompound, NbtList, NbtTag};

fn main() {
    // All NbtTags are stored in NbtCompounds
    let mut nbt = NbtCompound::new();

    // You can insert values wrapped in NbtTags
    nbt.insert("int", NbtTag::Int(128));
    // or you can insert the value directly
    nbt.insert("byte", 42_u8);

    // Vecs that are not of bytes, ints, or longs have to be converted to an NbtList first
    nbt.insert("list", NbtList::from(vec!["string 1", "string 2"]));

    // NbtCompound::display will convert the compound tag to snbt
    println!("{}", nbt);

    // Alternatively, you can do the same as the above with our handy `compound!` macro
    let macro_nbt = compound! {
        "int": 128i32,
        "byte": 42u8,
        "list": ["string 1", "string 2"]
    };

    assert_eq!(macro_nbt, nbt);
}
