mod assets;
use assets::*;
use quartz_nbt::{
    compound,
    io::{self, read_nbt, write_nbt, Flavor},
    NbtList,
};
use std::io::Cursor;

#[test]
fn big_test() {
    let (nbt, root_name) = io::read_nbt(&mut Cursor::new(BIG_TEST), BIG_TEST_FLAVOR).unwrap();

    assert_eq!(root_name, "Level");
    assert_compound_eq!(&nbt, &*BIG_TEST_VALIDATE);
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
    let nbt = compound! {
        "byte": 12i8,
        "short": 32i16,
        "int": 512i32,
        "long": 1024i64,
        "float": 12.99f32,
        "double": 1212.0101f64,
        "string": "test",
        "list": ["a", "b", "c"],
        "compound_list": [{}, {}],
        "byte_array": [B; 1, 2, 3, 4],
        "int_array": [I; 1, 3, 5, 7],
        "long_array": [L; 1, 9, 81],
        "compound": {
            "test": 12i8
        }
    };

    let mut bytes = Vec::new();
    write_nbt(&mut bytes, None, &nbt, Flavor::Uncompressed).unwrap();

    let read_nbt = read_nbt(&mut Cursor::new(bytes), Flavor::Uncompressed)
        .unwrap()
        .0;

    assert_compound_eq!(read_nbt, nbt);
}
