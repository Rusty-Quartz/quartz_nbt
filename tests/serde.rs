#![cfg(feature = "serde")]

mod assets;
use assets::*;
use quartz_nbt::{
    io::{self, Flavor},
    serde::{deserialize, serialize, Array},
    NbtCompound,
    NbtList,
    NbtTag,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Cursor};

#[derive(Serialize, Deserialize, PartialEq)]
struct Level {
    #[serde(rename = "Data")]
    data: LevelData,
}

#[derive(Serialize, Deserialize, PartialEq)]
struct LevelData {
    #[serde(rename = "allowCommands")]
    allow_commands: bool,
    #[serde(rename = "BorderCenterX")]
    border_center_x: f64,
    #[serde(rename = "BorderCenterZ")]
    border_center_z: f64,
    #[serde(rename = "BorderDamagePerBlock")]
    border_damage_per_block: f64,
    #[serde(rename = "BorderSafeZone")]
    border_safe_zone: f64,
    #[serde(rename = "BorderSize")]
    border_size: f64,
    #[serde(rename = "BorderSizeLerpTarget")]
    border_size_lerp_target: f64,
    #[serde(rename = "BorderSizeLerpTime")]
    border_size_lerp_time: i64,
    #[serde(rename = "BorderWarningBlocks")]
    border_warning_blocks: f64,
    #[serde(rename = "BorderWarningTime")]
    border_warning_time: f64,
    #[serde(rename = "Bukkit.Version")]
    bukkit_version: String,
    #[serde(rename = "clearWeatherTime")]
    clear_weather_time: i32,
    #[serde(rename = "CustomBossEvents")]
    custom_boss_events: NbtCompound,
    #[serde(rename = "DataPacks")]
    data_packs: DataPacks,
    #[serde(rename = "DataVersion")]
    data_version: i32,
    #[serde(rename = "DayTime")]
    day_time: i64,
    #[serde(rename = "Difficulty")]
    difficulty: Difficulty,
    #[serde(rename = "DifficultyLocked")]
    difficulty_locked: bool,
    #[serde(rename = "DragonFight")]
    dragon_fight: DragonFight,
    #[serde(rename = "GameRules")]
    game_rules: HashMap<String, String>,
    #[serde(rename = "GameType")]
    game_type: i32,
    hardcore: bool,
    initialized: bool,
    #[serde(rename = "LastPlayed")]
    last_played: i64,
    #[serde(rename = "LevelName")]
    level_name: String,
    raining: bool,
    #[serde(rename = "rainTime")]
    rain_time: i32,
    #[serde(rename = "ScheduledEvents")]
    scheduled_events: NbtList,
    #[serde(rename = "ServerBrands")]
    server_brands: Vec<String>,
    #[serde(rename = "SpawnAngle")]
    spawn_angle: f32,
    #[serde(rename = "SpawnX")]
    spawn_x: i32,
    #[serde(rename = "SpawnY")]
    spawn_y: i32,
    #[serde(rename = "SpawnZ")]
    spawn_z: i32,
    #[serde(rename = "Time")]
    time: i64,
    thundering: bool,
    #[serde(rename = "thunderTime")]
    thunder_time: i32,
    #[serde(rename = "Version")]
    verbose_version: Version,
    version: i32,
    #[serde(rename = "WanderingTraderSpawnDelay")]
    wandering_trader_spawn_delay: i32,
    #[serde(rename = "WanderingTraderSpawnChance")]
    wandering_trader_spawn_chance: i32,
    #[serde(rename = "WasModded")]
    was_modded: bool,
    #[serde(rename = "WorldGenSettings")]
    world_gen_settings: NbtCompound,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct DataPacks {
    #[serde(rename = "Enabled")]
    enabled: Vec<String>,
    #[serde(rename = "Disabled")]
    disabled: Vec<String>,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[repr(i8)]
enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Serialize for Difficulty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_i8(*self as i8)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct DragonFight {
    #[serde(rename = "DragonKilled")]
    dragon_killed: bool,
    #[serde(rename = "Gateways")]
    gateways: Vec<i32>,
    #[serde(rename = "PreviouslyKilled")]
    previously_killed: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct Version {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Snapshot")]
    snapshot: bool,
    #[serde(rename = "Id")]
    id: i32,
}

#[test]
fn level_dat() {
    let level: Level = deserialize(LEVEL_DAT, Flavor::GzCompressed).unwrap().0;
    let serialized = serialize(&level, None, Flavor::Uncompressed).unwrap();
    let test_nbt = io::read_nbt(&mut Cursor::new(serialized), Flavor::Uncompressed)
        .unwrap()
        .0;
    let validate_nbt = io::read_nbt(&mut Cursor::new(LEVEL_DAT), Flavor::GzCompressed)
        .unwrap()
        .0;
    assert_eq!(test_nbt, validate_nbt)
}

macro_rules! compound {
    ($($key: ident: $value: expr),*) => {
        {
            let mut compound = NbtCompound::new();
            $(compound.insert(stringify!($key), $value);)*
            compound
        }
    };
}

macro_rules! list {
    ($($value: expr),*) => {
        NbtList::from(vec![$($value),*]);
    };
}

#[test]
fn basic_datatypes_serde() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        boolean: bool,
        unsigned_byte: u8,
        signed_byte: i8,
        signed_short: i16,
        signed_int: i32,
        signed_long: i64,
        float: f32,
        double: f64,
        string: String,
    }

    let test_struct = Foo {
        boolean: true,
        unsigned_byte: 174,
        signed_byte: -12,
        signed_short: 1212,
        signed_int: 42,
        signed_long: 69420,
        float: 3.14159265,
        double: 100.001,
        string: "Mario".to_owned(),
    };

    let serialized_struct = serialize(&test_struct, None, Flavor::Uncompressed).unwrap();

    let struct_nbt = io::read_nbt(
        &mut Cursor::new(serialized_struct.clone()),
        Flavor::Uncompressed,
    )
    .unwrap()
    .0;

    let validation_nbt = compound!(
        boolean: NbtTag::Byte(1),
        unsigned_byte: NbtTag::Byte(-82),
        signed_byte: NbtTag::Byte(-12),
        signed_short: NbtTag::Short(1212),
        signed_int: NbtTag::Int(42),
        signed_long: NbtTag::Long(69420),
        float: NbtTag::Float(3.14159265),
        double: NbtTag::Double(100.001),
        string: NbtTag::String("Mario".to_owned())
    );

    assert_eq!(struct_nbt, validation_nbt);

    let deserialized_struct: Foo = deserialize(&serialized_struct, Flavor::Uncompressed)
        .unwrap()
        .0;
    assert_eq!(deserialized_struct, test_struct);
}

#[test]
fn complex_structs_serde() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        id: u8,
        bar: Bar,
        a: A,
        b: B,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar {
        a: i32,
        b: i32,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct A(i64);
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct B(u8, u8, u8);

    let test_struct = Foo {
        id: 12,
        bar: Bar { a: 13, b: 1990 },
        a: A(128),
        b: B(12, 12, 12),
    };

    let serialized_struct = serialize(&test_struct, None, Flavor::Uncompressed).unwrap();

    let struct_nbt = io::read_nbt(
        &mut Cursor::new(serialized_struct.clone()),
        Flavor::Uncompressed,
    )
    .unwrap()
    .0;

    let validation_nbt = compound!(
        id: NbtTag::Byte(12),
        bar: compound!(
            a: NbtTag::Int(13),
            b: NbtTag::Int(1990)
        ),
        a: NbtTag::Long(128),
        b: list!(NbtTag::Byte(12), NbtTag::Byte(12), NbtTag::Byte(12))
    );

    assert_eq!(struct_nbt, validation_nbt);

    let deserialized_struct: Foo = deserialize(&serialized_struct, Flavor::Uncompressed)
        .unwrap()
        .0;
    assert_eq!(deserialized_struct, test_struct);
}

#[test]
fn enum_serde() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        newtype: A,
        tuple: A,
        compound: A,
        unit: A,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum A {
        B(i16),
        C(u8, u8, u8),
        D { e: String, f: i32 },
        E,
        F,
        G,
    }

    let test_struct = Foo {
        newtype: A::B(-37),
        tuple: A::C(128, 99, 5),
        compound: A::D {
            e: "string".to_owned(),
            f: 999,
        },
        unit: A::F,
    };

    let serialized_struct = serialize(&test_struct, None, Flavor::Uncompressed).unwrap();

    let struct_nbt = io::read_nbt(
        &mut Cursor::new(serialized_struct.clone()),
        Flavor::Uncompressed,
    )
    .unwrap()
    .0;

    let validation_nbt = compound!(
        newtype: compound!(B: NbtTag::Short(-37)),
        tuple: compound!(C: list!(NbtTag::Byte(-128), NbtTag::Byte(99), NbtTag::Byte(5))),
        compound:
            compound!(
                D: compound!(
                    e: NbtTag::String("string".to_owned()),
                    f: NbtTag::Int(999)
                )
            ),
        unit: NbtTag::Int(4)
    );

    assert_eq!(struct_nbt, validation_nbt);

    let deserialized_struct: Foo = deserialize(&serialized_struct, Flavor::Uncompressed)
        .unwrap()
        .0;
    assert_eq!(deserialized_struct, test_struct);
}

#[test]
fn array_serde() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        byte_array: Array<Vec<u8>>,
        byte_array2: Array<Vec<i8>>,
        int_array: Array<Vec<i32>>,
        long_array: Array<Vec<i64>>,
    }

    let test_struct = Foo {
        byte_array: vec![12, 13, 14].into(),
        byte_array2: vec![51, 32, 99].into(),
        int_array: vec![120, 99999, 12].into(),
        long_array: vec![2122, 121212, 6666666].into(),
    };

    let serialized_struct = serialize(&test_struct, None, Flavor::Uncompressed).unwrap();

    let struct_nbt = io::read_nbt(
        &mut Cursor::new(serialized_struct.clone()),
        Flavor::Uncompressed,
    )
    .unwrap()
    .0;

    let validation_nbt = compound!(
        byte_array: NbtTag::ByteArray(vec![12, 13, 14]),
        byte_array2: NbtTag::ByteArray(vec![51, 32, 99]),
        int_array: NbtTag::IntArray(vec![120, 99999, 12]),
        long_array: NbtTag::LongArray(vec![2122, 121212, 6666666])
    );

    assert_eq!(struct_nbt, validation_nbt);

    let deserialized_struct: Foo = deserialize(&serialized_struct, Flavor::Uncompressed)
        .unwrap()
        .0;
    assert_eq!(deserialized_struct, test_struct);
}

#[test]
fn vec_serde() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        bar: Vec<Bar>,
        strings: Vec<String>,
        baz: Vec<Baz>,
        tuple: Vec<Tuple>,
        enumeration: Vec<Enumeration>,
        mixed_enumeration: Vec<Enumeration>,
        bar_arr: Array<Vec<Bar>>,
        strings_arr: Array<Vec<String>>,
        baz_arr: Array<Vec<Baz>>,
        tuple_arr: Array<Vec<Tuple>>,
        nested_arr: Vec<Array<Vec<Array<Vec<i8>>>>>,
        enumeration_arr: Array<Vec<Enumeration>>,
        enum_of_vec: Enumeration,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar {
        a: i32,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Baz(i8);
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Tuple(i16, i16, i16);

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum Enumeration {
        A,
        B,
        C,
        D(i8),
        E,
        F { a: i16 },
        G(Vec<Vec<Bar>>),
    }

    let test_struct = Foo {
        baz_arr: vec![Baz(52)].into(),
        baz: vec![Baz(99), Baz(42), Baz(88)],
        bar: vec![Bar { a: 32 }, Bar { a: 99 }],
        tuple_arr: vec![Tuple(12, 12, 12)].into(),
        tuple: vec![Tuple(343, 89, 102), Tuple(33, 897, 457)],
        strings: vec!["test".to_owned(), "test test test".to_owned()],
        enumeration: vec![Enumeration::A, Enumeration::B, Enumeration::E],
        mixed_enumeration: vec![Enumeration::D(12), Enumeration::F { a: 14 }],
        bar_arr: vec![Bar { a: 35 }].into(),
        strings_arr: vec!["tteesstt".to_owned()].into(),
        nested_arr: vec![
            vec![vec![1, 20, 9].into()].into(),
            vec![vec![3, 5, 10].into(), vec![99, 10, 32].into()].into(),
        ],
        enumeration_arr: vec![Enumeration::A, Enumeration::C, Enumeration::E].into(),
        enum_of_vec: Enumeration::G(vec![vec![Bar { a: 13 }, Bar { a: 9 }], vec![Bar { a: 14 }]]),
    };

    let serialized_struct = serialize(&test_struct, None, Flavor::Uncompressed).unwrap();
    let nbt_struct = io::read_nbt(
        &mut Cursor::new(serialized_struct.clone()),
        Flavor::Uncompressed,
    )
    .unwrap()
    .0;

    let validation_nbt = compound!(
        bar: list![compound!(a: NbtTag::Int(32)), compound!(a: NbtTag::Int(99))],
        strings:
            list![
                NbtTag::String("test".to_owned()),
                NbtTag::String("test test test".to_owned())
            ],
        baz: list![NbtTag::Byte(99), NbtTag::Byte(42), NbtTag::Byte(88)],
        tuple:
            list![
                list![NbtTag::Short(343), NbtTag::Short(89), NbtTag::Short(102)],
                list![NbtTag::Short(33), NbtTag::Short(897), NbtTag::Short(457)]
            ],
        bar_arr: list![compound!(a: NbtTag::Int(35))],
        strings_arr: list![NbtTag::String("tteesstt".to_owned())],
        enumeration: list![NbtTag::Int(0), NbtTag::Int(1), NbtTag::Int(4)],
        mixed_enumeration:
            list![
                compound!(D: NbtTag::Byte(12)),
                compound!(F: compound!(a: NbtTag::Short(14)))
            ],
        baz_arr: NbtTag::ByteArray(vec![52]),
        tuple_arr:
            list![list![
                NbtTag::Short(12),
                NbtTag::Short(12),
                NbtTag::Short(12)
            ]],
        nested_arr:
            list![list![NbtTag::ByteArray(vec![1, 20, 9])], list![
                NbtTag::ByteArray(vec![3, 5, 10]),
                NbtTag::ByteArray(vec![99, 10, 32])
            ]],
        enumeration_arr: NbtTag::IntArray(vec![0, 2, 4]),
        enum_of_vec:
            compound!(
                G: list![list![compound!(a: 13), compound!(a: 9)], list![
                    compound!(a: 14)
                ]]
            )
    );

    assert_eq!(nbt_struct, validation_nbt);

    let deserialized_struct: Foo = deserialize(&serialized_struct, Flavor::Uncompressed)
        .unwrap()
        .0;
    assert_eq!(deserialized_struct, test_struct);
}

#[test]
fn option_serde() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        a: Option<i8>,
        b: Option<String>,
        c: Option<Vec<i8>>,
        d: Option<Array<Vec<i8>>>,
        e: Option<Bar>,
        f: Option<Baz>,
        g: Option<Tuple>,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar {
        a: i8,
        b: Option<String>,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum Baz {
        A(i16),
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Tuple(i8, i8);

    let test_struct = Foo {
        a: Some(0),
        b: Some("option".to_owned()),
        c: None,
        d: Some(vec![21, 42, 15].into()),
        e: Some(Bar { a: 12, b: None }),
        f: Some(Baz::A(13)),
        g: Some(Tuple(21, 98)),
    };
    let serialized_struct = serialize(&test_struct, None, Flavor::Uncompressed).unwrap();

    let struct_nbt = io::read_nbt(
        &mut Cursor::new(serialized_struct.clone()),
        Flavor::Uncompressed,
    )
    .unwrap()
    .0;

    let validation_nbt = compound!(
        a: NbtTag::Byte(0),
        b: NbtTag::String("option".to_owned()),
        d: NbtTag::ByteArray(vec![21, 42, 15]),
        e: compound!(
            a: NbtTag::Byte(12)
        ),
        f: compound!(
            A: NbtTag::Short(13)
        ),
        g: list![NbtTag::Byte(21), NbtTag::Byte(98)]
    );

    assert_eq!(struct_nbt, validation_nbt);

    let deserialized_struct: Foo = deserialize(&serialized_struct, Flavor::Uncompressed)
        .unwrap()
        .0;
    assert_eq!(deserialized_struct, test_struct);
}
