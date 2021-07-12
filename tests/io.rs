mod assets;
use assets::*;
use quartz_nbt::{io, NbtList};
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
