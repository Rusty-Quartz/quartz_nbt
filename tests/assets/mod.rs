#![allow(dead_code)]

use once_cell::sync::Lazy;
use quartz_nbt::{io::Flavor, NbtCompound, NbtList, NbtTag};

pub const BIG_TEST: &[u8] = include_bytes!("bigtest.nbt");
pub const BIG_TEST_FLAVOR: Flavor = Flavor::GzCompressed;
pub static BIG_TEST_VALIDATE: Lazy<NbtCompound> = Lazy::new(|| {
    let mut level = NbtCompound::new();

    let mut nested_test = NbtCompound::new();
    let mut egg = NbtCompound::new();
    egg.insert("name", "Eggbert");
    egg.insert("value", 0.5f32);
    let mut ham = NbtCompound::new();
    ham.insert("name", "Hampus");
    ham.insert("value", 0.75f32);
    nested_test.insert("egg", egg);
    nested_test.insert("ham", ham);
    level.insert("nested compound test", nested_test);

    level.insert("intTest", 2147483647i32);
    level.insert("byteTest", 127i8);
    level.insert(
        "stringTest",
        "HELLO WORLD THIS IS A TEST STRING \u{C5}\u{C4}\u{D6}!",
    );
    level.insert(
        "listTest (long)",
        NbtList::from(vec![11i64, 12, 13, 14, 15]),
    );
    level.insert("doubleTest", 0.49312871321823148f64);
    level.insert("floatTest", 0.49823147058486938f32);
    level.insert("longTest", 9223372036854775807i64);

    let mut list = NbtList::new();
    let mut compound0 = NbtCompound::new();
    compound0.insert("created-on", 1264099775885i64);
    compound0.insert("name", "Compound tag #0");
    list.push(compound0);
    let mut compound1 = NbtCompound::new();
    compound1.insert("created-on", 1264099775885i64);
    compound1.insert("name", "Compound tag #1");
    list.push(compound1);
    level.insert("listTest (compound)", list);

    let mut bytes = Vec::new();
    for n in 0 .. 1000 {
        bytes.push(((n * n * 255 + n * 7) % 100) as i8);
    }
    level.insert(
        "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, \
         16, 8, ...))",
        bytes,
    );

    level.insert("shortTest", 32767i16);

    level
});

pub const PLAYER_NAN_VALUE: &[u8] = include_bytes!("Player-nan-value.dat");
pub const PLAYER_NAN_VALUE_FLAVOR: Flavor = Flavor::GzCompressed;

pub const SNBT_EDGE_CASES: &str = include_str!("snbt_edge_cases.snbt");
pub static SNBT_EDGE_CASES_VALIDATE: Lazy<NbtCompound> = Lazy::new(|| {
    let mut compound = NbtCompound::new();
    compound.insert("byte_min", i8::MIN);
    compound.insert("byte_max", i8::MAX);
    compound.insert("short_min", i16::MIN);
    compound.insert("short_max", i16::MAX);
    compound.insert("int_min", i32::MIN);
    compound.insert("int_max", i32::MAX);
    compound.insert("long_min", i64::MIN);
    compound.insert("long_max", i64::MAX);
    compound.insert("f32_0", 0.0f32);
    compound.insert("f32_10", 10f32);
    compound.insert("f32_dec", 0.653f32);
    compound.insert("f32_neg", -1.23453f32);
    compound.insert("f64_0", 0.0f64);
    compound.insert("f64_n10", -10f64);
    compound.insert("f64_dec", 0.987f64);
    compound.insert("f64_neg", -128375.1f64);
    compound.insert("f64_suffixed", 123.4f64);
    compound.insert("f64_alt_suffixed", 123.5f64);
    compound.insert(
        "this is a ;.# v3ry $trange keë",
        "with a weirder { value? [.*; \"\\\"\\\'\'\"] }",
    );
    compound.insert("unicode test", "aé日\u{10401}");
    compound.insert("empty_byte_array", Vec::<i8>::new());
    compound.insert("empty_int_array", Vec::<i32>::new());
    compound.insert("empty_long_array", Vec::<i64>::new());
    compound.insert("empty_tag_array", NbtList::new());
    let mut chaotic_array = NbtList::new();
    chaotic_array.push(NbtList::from(vec![NbtTag::IntArray(Vec::new())]));
    chaotic_array.push(NbtList::from(vec![NbtTag::LongArray(vec![10])]));
    chaotic_array.push(NbtList::from(vec![
        NbtTag::ByteArray(Vec::new()),
        NbtTag::ByteArray(vec![1, 2, 3]),
    ]));
    let mut c0 = NbtCompound::new();
    let mut foo = NbtCompound::new();
    foo.insert("bar", NbtList::from(vec!["baz", "buz"]));
    c0.insert("foo", foo);
    c0.insert(".{}", NbtCompound::new());
    chaotic_array.push(NbtList::from(vec![c0]));
    chaotic_array.push(NbtList::from(vec![0.0f64, 0.0f64]));
    compound.insert("chaotic_array", chaotic_array);
    let mut nested_compounds = NbtCompound::new();
    let mut c1 = NbtCompound::new();
    let mut c2 = NbtCompound::new();
    let mut c3 = NbtCompound::new();
    let mut c4 = NbtCompound::new();
    let mut c5 = NbtCompound::new();
    c5.insert(
        "this is a key",
        r#"and [ this }{] '}' is { \'heh\' a \"lol"}"\"}'"'}"'" value"#,
    );
    c4.insert("c5", c5);
    c3.insert("c4", c4);
    c2.insert("c3", c3);
    c2.insert("a", "b");
    c1.insert("c2", c2);
    nested_compounds.insert("c1", c1);
    compound.insert("nested_compounds", nested_compounds);
    compound
});

pub const BIG_SNBT: &str = include_str!("big_snbt.snbt");

pub const LEVEL_DAT: &[u8] = include_bytes!("level.dat");
