mod assets;
use assets::*;
use quartz_nbt::{
    io::{self, read_nbt, write_nbt, Flavor},
    NbtCompound,
    NbtList,
};
use std::io::Cursor;

#[test]
fn big_test() {
    let (nbt, root_name) = io::read_nbt(&mut Cursor::new(BIG_TEST), BIG_TEST_FLAVOR).unwrap();

    assert_eq!(root_name, "Level");
    assert_eq!(&nbt, &*BIG_TEST_VALIDATE);
}

#[test]
fn player_nan_value() {
    let (nbt, _) =
        io::read_nbt(&mut Cursor::new(PLAYER_NAN_VALUE), PLAYER_NAN_VALUE_FLAVOR).unwrap();
    let pos = nbt.get::<_, &NbtList>("Pos").unwrap();

    assert_eq!(pos.get::<f64>(0).unwrap(), 0.0);
    assert_eq!(pos.get::<f64>(2).unwrap(), 0.0);
    assert!(pos.get::<f64>(1).unwrap().is_nan());
}

#[test]
fn writing_nbt() {
    let mut nbt = NbtCompound::new();
    nbt.insert("byte", 12_i8);
    nbt.insert("short", 32_i16);
    nbt.insert("int", 512_i32);
    nbt.insert("long", 1024_i64);
    nbt.insert("float", 12.99_f32);
    nbt.insert("double", 1212.0101_f64);
    nbt.insert("string", "test");
    nbt.insert("list", NbtList::from(vec!["a", "b", "c"]));
    nbt.insert(
        "compound_list",
        NbtList::from(vec![NbtCompound::new(), NbtCompound::new()]),
    );
    nbt.insert("byte_array", vec![1_i8, 2, 3, 4]);
    nbt.insert("int_array", vec![1_i32, 3, 5, 7]);
    nbt.insert("long_array", vec![1_i64, 9, 81]);
    let mut test_tag = NbtCompound::new();
    test_tag.insert("test", 12_i8);
    nbt.insert("compound", test_tag);

    let mut bytes = Vec::new();
    write_nbt(&mut bytes, None, &nbt, Flavor::Uncompressed).unwrap();
    println!("{:02X?}", bytes);

    let read_nbt = read_nbt(&mut Cursor::new(bytes), Flavor::Uncompressed)
        .unwrap()
        .0;

    assert_eq!(nbt, read_nbt);
}
