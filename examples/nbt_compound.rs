use quartz_nbt::{NbtCompound, NbtList, NbtTag};

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
}
