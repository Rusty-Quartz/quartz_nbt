#![allow(dead_code)]

use once_cell::sync::Lazy;
use quartz_nbt::{compound, io::Flavor, NbtCompound};

pub const BIG_TEST: &[u8] = include_bytes!("bigtest.nbt");
pub const BIG_TEST_FLAVOR: Flavor = Flavor::GzCompressed;
pub static BIG_TEST_VALIDATE: Lazy<NbtCompound> = Lazy::new(|| {
    let mut bytes = Vec::new();
    for n in 0 .. 1000 {
        bytes.push(((n * n * 255 + n * 7) % 100) as i8);
    }

    compound! {
        "nested compound test": {
            "egg": {
                "name": "Eggbert",
                "value": 0.5f32
            },
            "ham": {
                "name": "Hampus",
                "value": 0.75f32
            }
        },
        "intTest": 2147483647i32,
        "byteTest": 127i8,
        "stringTest": "HELLO WORLD THIS IS A TEST STRING \u{C5}\u{C4}\u{D6}!",
        "listTest (long)": [11i64, 12, 13, 14, 15],
        "doubleTest": 0.49312871321823148f64,
        "floatTest": 0.49823147058486938f32,
        "longTest": 9223372036854775807i64,
        "listTest (compound)": [
            {
                "created-on": 1264099775885i64,
                "name": "Compound tag #0"
            },
            {
                "created-on": 1264099775885i64,
                "name": "Compound tag #1"
            }
        ],
        "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, \
            16, 8, ...))": bytes,
        "shortTest": 32767i16
    }
});

pub const PLAYER_NAN_VALUE: &[u8] = include_bytes!("Player-nan-value.dat");
pub const PLAYER_NAN_VALUE_FLAVOR: Flavor = Flavor::GzCompressed;

pub const SNBT_EDGE_CASES: &str = include_str!("snbt_edge_cases.snbt");
pub static SNBT_EDGE_CASES_VALIDATE: Lazy<NbtCompound> = Lazy::new(|| {
    compound! {
        "byte_min": i8::MIN,
        "byte_max": i8::MAX,
        "short_min": i16::MIN,
        "short_max": i16::MAX,
        "int_min": i32::MIN,
        "int_max": i32::MAX,
        "long_min": i64::MIN,
        "long_max": i64::MAX,
        "f32_0": 0.0f32,
        "f32_10": 10f32,
        "f32_dec": 0.653f32,
        "f32_neg": -1.23453f32,
        "f64_0": 0.0f64,
        "f64_n10": -10f64,
        "f64_dec": 0.987f64,
        "f64_neg": -128375.1f64,
        "f64_suffixed": 123.4f64,
        "f64_alt_suffixed": 123.5f64,
        "this is a ;.# v3ry $trange keë": "with a weirder { value? [.*; \"\\\"\\\'\'\"] }",
        "unicode test": "aé日\u{10401}",
        "empty_byte_array": [B;],
        "empty_int_array": [I;],
        "empty_long_array": [L;],
        "empty_tag_array": [],
        "chaotic_array": [
            [[I;]],
            [[L; 10]],
            [[B;], [B; 1, 2, 3]],
            [{"foo": {"bar": ["baz", "buz"]}, ".{}": {}}],
            [0f64, 0f64]
        ],
        "nested_compounds": {
            "c1": {
                "c2": {
                    "c3": {
                        "c4": {
                            "c5": {
                                "this is a key":
                                r#"and [ this }{] '}' is { \'heh\' a \"lol"}"\"}'"'}"'" value"#
                            }
                        }
                    },
                    "a": "b"
                }
            }
        },
        "quoted \"key\"": "quoted 'value'",
        "redundant": "quotes",
        "more_redundant": "quotes",
        "escape sequences": "\'\\\r\n\t\u{00A7}\u{0F63}",
    }
});

pub const BIG_SNBT: &str = include_str!("big_snbt.snbt");

pub const LEVEL_DAT: &[u8] = include_bytes!("level.dat");
