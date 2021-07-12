mod assets;
use assets::*;
use quartz_nbt::{snbt, NbtCompound};

#[test]
fn edge_cases() {
    let nbt = snbt::parse(SNBT_EDGE_CASES).unwrap();
    assert_eq!(&nbt, &*SNBT_EDGE_CASES_VALIDATE);
}

#[test]
fn big_test() {
    let result = snbt::parse(BIG_SNBT);
    assert!(result.is_ok());
    let result = result.unwrap();
    let inner = &result
        .get::<_, &NbtCompound>("Riding")
        .unwrap()
        .get::<_, &NbtCompound>("Riding")
        .unwrap()
        .get::<_, &NbtCompound>("Riding")
        .unwrap()
        .get::<_, &NbtCompound>("TileEntityData")
        .unwrap()
        .get::<_, &str>("Command")
        .unwrap()[32 ..];
    assert!(snbt::parse(inner).is_ok());
}
