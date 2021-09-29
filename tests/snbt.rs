mod assets;
use assets::*;
use quartz_nbt::{snbt, NbtCompound};

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
        snbt::parse_and_size(&inner).unwrap().0,
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
