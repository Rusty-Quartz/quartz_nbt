mod assets;
use assets::*;
use quartz_nbt::{snbt, NbtCompound};
use quartz_nbt_macros::compound;

#[test]
fn edge_cases() {
    let nbt = snbt::parse(SNBT_EDGE_CASES).unwrap();
    assert_compound_eq!(&nbt, &*SNBT_EDGE_CASES_VALIDATE);
}

#[test]
fn big_test() {
    let result = snbt::parse(BIG_SNBT);
    assert!(result.is_ok());
    let result = result.unwrap();
    let inner = result
        .get::<_, &NbtCompound>("Riding")
        .unwrap()
        .get::<_, &NbtCompound>("Riding")
        .unwrap()
        .get::<_, &NbtCompound>("Riding")
        .unwrap()
        .get::<_, &NbtCompound>("TileEntityData")
        .unwrap()
        .get::<_, &str>("Command")
        .unwrap()[32 ..]
        .to_string()
        + " and some garbage";
    assert!(snbt::parse(&inner).is_ok());
    assert_eq!(
        snbt::parse_and_size(&inner).unwrap().1,
        inner.len() - " and some garbage".len()
    );
}

#[test]
fn formatting() {
    let repr = format!("{:+#.2?}", &*SNBT_EDGE_CASES_VALIDATE);

    // For manual inspection
    println!("{}", repr);

    assert_compound_eq!(
        &snbt::parse(&SNBT_EDGE_CASES_VALIDATE.to_snbt()).unwrap(),
        &*SNBT_EDGE_CASES_VALIDATE
    );
    assert_compound_eq!(
        &snbt::parse(&SNBT_EDGE_CASES_VALIDATE.to_pretty_snbt()).unwrap(),
        &*SNBT_EDGE_CASES_VALIDATE
    );
}

#[test]
fn number_like_strings() {
    fn assert_round_trip_same(tag: NbtCompound) {
        let repr = tag.to_string();

        // For manual inspection
        println!("{repr}");

        assert_eq!(quartz_nbt::snbt::parse(&repr).unwrap(), tag);
    }

    assert_round_trip_same(compound! { "str": "1" });
    assert_round_trip_same(compound! { "str": "-1" });
    assert_round_trip_same(compound! { "str": "0" });
    assert_round_trip_same(compound! { "str": "-0" });
    assert_round_trip_same(compound! { "str": "0.5" });
    assert_round_trip_same(compound! { "str": "-0.5" });
}
