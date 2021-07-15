use quartz_nbt::{snbt::SnbtError, NbtCompound};

fn main() -> Result<(), SnbtError> {
    // Instead of making a NbtCompound by inserting everything you can use SNBT
    // This allows NBT to be represented by a json-like format
    // some things to note are numbers have to be suffixed to be a type other than Int
    // and lists can become ByteArrays or similar by putting B; (or respective) at the start
    let nbt = NbtCompound::from_snbt(
        r#"{
        name: "stringified nbt",
        tags: 4S,
        nested_compounds: {
            "keys can have spaces": [B;12, 13, 14]
        }
    }"#,
    )?;

    // You can also convert back to SNBT by using .to_snbt on any Nbt value
    // This includes NbtTag and NbtList
    println!("{}", nbt.to_snbt());

    Ok(())
}
