#![cfg(feature = "serde")]

mod assets;
use assets::*;
use quartz_nbt::{
    compound,
    io::{self, Flavor},
    serde::{deserialize, deserialize_from, deserialize_from_buffer, serialize, Array},
    NbtCompound,
    NbtList,
    NbtTag,
};
use serde::{de::Visitor, ser::SerializeStruct, Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::TryInto,
    io::{Cursor, Seek, SeekFrom},
};

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
    assert_compound_eq!(test_nbt, validate_nbt)
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

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
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

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    let validation_nbt = compound! {
        "boolean": 1i8,
        "unsigned_byte": -82i8,
        "signed_byte": -12i8,
        "signed_short": 1212i16,
        "signed_int": 42i32,
        "signed_long": 69420i64,
        "float": 3.14159265f32,
        "double": 100.001f64,
        "string": "Mario"
    };

    assert_compound_eq!(struct_nbt, validation_nbt);

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

    let validation_nbt = compound! {
        "id": 12i8,
        "bar": {
            "a": 13i32,
            "b": 1990i32
        },
        "a": 128i64,
        "b": [12i8, 12i8, 12i8]
    };

    assert_compound_eq!(struct_nbt, validation_nbt);

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

    let validation_nbt = compound! {
        "newtype": {
            "B": -37i16
        },
        "tuple": {
            "C": [-128i8, 99i8, 5i8]
        },
        "compound": {
            "D": {
                "e": "string",
                "f": 999i32
            }
        },
        "unit": 4i32
    };

    assert_compound_eq!(struct_nbt, validation_nbt);

    let deserialized_struct: Foo = deserialize(&serialized_struct, Flavor::Uncompressed)
        .unwrap()
        .0;
    assert_eq!(deserialized_struct, test_struct);

    #[derive(PartialEq, Deserialize, Debug)]
    struct DeserializeByName {
        enums: Vec<A>,
    }

    let deserialize_by_name = DeserializeByName {
        enums: vec![A::G, A::E, A::F],
    };

    let deserialize_by_name_nbt = compound! {
        "enums": ["G", "E", "F"]
    };

    let mut serialized_struct = Cursor::new(Vec::<u8>::new());
    io::write_nbt(
        &mut serialized_struct,
        None,
        &deserialize_by_name_nbt,
        Flavor::Uncompressed,
    )
    .unwrap();
    serialized_struct.seek(SeekFrom::Start(0)).unwrap();

    let deserialized_struct: DeserializeByName =
        deserialize_from(&mut serialized_struct, Flavor::Uncompressed)
            .unwrap()
            .0;

    assert_eq!(deserialize_by_name, deserialized_struct);
}

#[test]
fn array_serde() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        byte_array: Array<Vec<u8>>,
        byte_array2: Array<Vec<i8>>,
        int_array: Array<Vec<i32>>,
        long_array: Array<Vec<i64>>,
        prim_byte_array: Array<[u8; 3]>,
        prim_byte_array2: Array<[i8; 3]>,
        prim_int_array: Array<[i32; 3]>,
        prim_long_array: Array<[i64; 3]>,
    }

    let test_struct = Foo {
        byte_array: vec![12, 13, 14].into(),
        byte_array2: vec![51, 32, 99].into(),
        int_array: vec![120, 99999, 12].into(),
        long_array: vec![2122, 121212, 6666666].into(),
        prim_byte_array: [0u8, 1, 2].into(),
        prim_byte_array2: [-1i8, -10, -128].into(),
        prim_int_array: [5i32, -10, 20].into(),
        prim_long_array: [i64::MIN, i64::MAX, 0].into(),
    };

    let serialized_struct = serialize(&test_struct, None, Flavor::Uncompressed).unwrap();

    let struct_nbt = io::read_nbt(
        &mut Cursor::new(serialized_struct.clone()),
        Flavor::Uncompressed,
    )
    .unwrap()
    .0;

    let validation_nbt = compound! {
        "byte_array": [B; 12, 13, 14],
        "byte_array2": [B; 51, 32, 99],
        "int_array": [I; 120, 99999, 12],
        "long_array": [L; 2122, 121212, 6666666],
        "prim_byte_array": [B; 0, 1, 2],
        "prim_byte_array2": [B; -1, -10, -128],
        "prim_int_array": [I; 5, -10, 20],
        "prim_long_array": [L; i64::MIN, i64::MAX, 0]
    };

    assert_compound_eq!(struct_nbt, validation_nbt);

    let deserialized_struct: Foo = deserialize(&serialized_struct, Flavor::Uncompressed)
        .unwrap()
        .0;
    assert_eq!(deserialized_struct, test_struct);

    // Wrapper to ensure we test the `serialize_bytes` method
    struct Bytes<'a>(&'a [u8]);

    impl<'a> Serialize for Bytes<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
            serializer.serialize_bytes(self.0)
        }
    }

    // Wrapper to ensure that we test the `deserialize_byte_buf` method
    struct ByteBuf(Vec<u8>);

    impl<'de> Deserialize<'de> for ByteBuf {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
            struct ByteBufVisitor;

            impl<'de> Visitor<'de> for ByteBufVisitor {
                type Value = Vec<u8>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "A byte buf")
                }

                fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
                where E: serde::de::Error {
                    Ok(v)
                }
            }

            Ok(ByteBuf(deserializer.deserialize_byte_buf(ByteBufVisitor)?))
        }
    }

    // Large arrays aren't incorporated into the serde data model by default, so we need to handle
    // them manually
    #[derive(PartialEq, Eq, Debug)]
    struct SerAsSlice {
        large_byte_array: Box<[u8; 1024]>,
        large_int_array: Box<[i32; 256]>,
    }

    impl Serialize for SerAsSlice {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
            let mut serialize_struct = serializer.serialize_struct("SerAsSlice", 2)?;
            let bytes: &[u8] = &*self.large_byte_array;
            // Ensure that we call the `serialize_bytes` method
            serialize_struct.serialize_field("large_byte_array", &Bytes(bytes))?;
            let slice: &[i32] = &*self.large_int_array;
            let arr: Array<&[i32]> = Array::from(slice);
            serialize_struct.serialize_field("large_int_array", &arr)?;
            serialize_struct.end()
        }
    }

    // Quick and dirty implementation since we test structs elsewhere
    impl<'de> Deserialize<'de> for SerAsSlice {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
            struct SerAsSliceVisitor;

            impl<'de> Visitor<'de> for SerAsSliceVisitor {
                type Value = SerAsSlice;

                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "Things")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where A: serde::de::MapAccess<'de> {
                    let mut large_byte_array: Option<Box<[u8; 1024]>> = None;
                    let mut large_int_array: Option<Box<[i32; 256]>> = None;

                    // Quick and dirty implementation since we test structs elsewhere
                    while let Some(key) = map.next_key::<String>()? {
                        match key.as_str() {
                            "large_byte_array" => {
                                assert!(
                                    large_byte_array.is_none(),
                                    "large_byte_array deserialized twice"
                                );
                                let bytes: ByteBuf = map.next_value()?;
                                large_byte_array = Some(Box::new(
                                    bytes
                                        .0
                                        .try_into()
                                        .expect("large_byte_array has incorrect length"),
                                ));
                            }
                            "large_int_array" => {
                                assert!(
                                    large_int_array.is_none(),
                                    "large_int_array deserialized twice"
                                );
                                let arr: Vec<i32> = map.next_value()?;
                                large_int_array = Some(Box::new(
                                    arr.try_into()
                                        .expect("large_int_array has incorrect length"),
                                ));
                            }
                            _ => panic!("Invalid key: {}", key),
                        }
                    }

                    Ok(SerAsSlice {
                        large_byte_array: large_byte_array.expect("large_byte_array missing"),
                        large_int_array: large_int_array.expect("large_int_array missing"),
                    })
                }
            }

            let ser_as_slice = deserializer.deserialize_struct(
                "SerAsSlice",
                &["large_byte_array", "large_int_array"],
                SerAsSliceVisitor,
            )?;
            Ok(ser_as_slice)
        }
    }

    let large_byte_array: Vec<_> = (0 .. 1024).map(|i| ((i * 7 + 13) % 255) as u8).collect();
    let large_int_array: Vec<_> = (0 .. 256).collect();

    let ser_as_slice = SerAsSlice {
        large_byte_array: Box::new(large_byte_array.clone().try_into().unwrap()),
        large_int_array: Box::new(large_int_array.clone().try_into().unwrap()),
    };

    let serialized_struct = serialize(&ser_as_slice, None, Flavor::Uncompressed).unwrap();

    let struct_nbt = io::read_nbt(
        &mut Cursor::new(serialized_struct.clone()),
        Flavor::Uncompressed,
    )
    .unwrap()
    .0;

    let validation_nbt = compound! {
        "large_byte_array": large_byte_array,
        "large_int_array": large_int_array
    };

    // Ensure the validation nbt is valid
    assert!(matches!(
        validation_nbt.get::<_, &NbtTag>("large_byte_array"),
        Ok(NbtTag::ByteArray(_))
    ));
    assert!(matches!(
        validation_nbt.get::<_, &NbtTag>("large_int_array"),
        Ok(NbtTag::IntArray(_))
    ));

    assert_compound_eq!(struct_nbt, validation_nbt);

    let deserialized_struct: SerAsSlice = deserialize(&serialized_struct, Flavor::Uncompressed)
        .unwrap()
        .0;
    assert_eq!(deserialized_struct, ser_as_slice);
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
        nested_arr: Vec<Vec<Array<Vec<i8>>>>,
        enum_of_vec: Enumeration,
        empty_byte_array: Array<Vec<i8>>,
        empty_int_array: Array<Vec<i32>>,
        empty_long_array: Array<Vec<i64>>,
        empty_tag_list: Vec<()>,
        seq_empty_byte_array: Vec<Array<Vec<i8>>>,
        seq_empty_int_array: Vec<Array<Vec<i32>>>,
        seq_empty_long_array: Vec<Array<Vec<i64>>>,
        seq_empty_tag_list: Vec<Vec<()>>,
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
        baz: vec![Baz(99), Baz(42), Baz(88)],
        bar: vec![Bar { a: 32 }, Bar { a: 99 }],
        tuple: vec![Tuple(343, 89, 102), Tuple(33, 897, 457)],
        strings: vec!["test".to_owned(), "test test test".to_owned()],
        enumeration: vec![Enumeration::A, Enumeration::B, Enumeration::E],
        mixed_enumeration: vec![Enumeration::D(12), Enumeration::F { a: 14 }],
        nested_arr: vec![vec![vec![1, 20, 9].into()], vec![
            vec![3, 5, 10].into(),
            vec![99, 10, 32].into(),
        ]],
        enum_of_vec: Enumeration::G(vec![vec![Bar { a: 13 }, Bar { a: 9 }], vec![Bar { a: 14 }]]),
        empty_byte_array: Vec::new().into(),
        empty_int_array: Vec::new().into(),
        empty_long_array: Vec::new().into(),
        empty_tag_list: Vec::new(),
        seq_empty_byte_array: vec![Vec::new().into(), Vec::new().into()],
        seq_empty_int_array: vec![Vec::new().into(), Vec::new().into()],
        seq_empty_long_array: vec![Vec::new().into(), Vec::new().into()],
        seq_empty_tag_list: vec![Vec::new(), Vec::new()],
    };

    let serialized_struct = serialize(&test_struct, None, Flavor::Uncompressed).unwrap();
    let nbt_struct = io::read_nbt(
        &mut Cursor::new(serialized_struct.clone()),
        Flavor::Uncompressed,
    )
    .unwrap()
    .0;

    let validation_nbt = compound! {
        "bar": [
            {
                "a": 32i32
            },
            {
                "a": 99i32
            }
        ],
        "strings": ["test", "test test test"],
        "baz": [99i8, 42i8, 88i8],
        "tuple": [
            [343i16, 89i16, 102i16],
            [33i16, 897i16, 457i16]
        ],
        "enumeration": [0i32, 1i32, 4i32],
        "mixed_enumeration": [
            {
                "D": 12i8
            },
            {
                "F": {
                    "a": 14i16
                }
            }
        ],
        "nested_arr": [
            [
                [B; 1, 20, 9]
            ],
            [
                [B; 3, 5, 10],
                [B; 99, 10, 32]
            ]
        ],
        "enum_of_vec": {
            "G": [
                [
                    {
                        "a": 13i32
                    },
                    {
                        "a": 9i32
                    }
                ],
                [
                    {
                        "a": 14
                    }
                ]
            ]
        },
        "empty_byte_array": [B;],
        "empty_int_array": [I;],
        "empty_long_array": [L;],
        "empty_tag_list": [],
        "seq_empty_byte_array": [[B;], [B;]],
        "seq_empty_int_array": [[I;], [I;]],
        "seq_empty_long_array": [[L;], [L;]],
        "seq_empty_tag_list": [[], []],
    };

    assert_compound_eq!(nbt_struct, validation_nbt);

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

    let validation_nbt = compound! {
        "a": 0i8,
        "b": "option",
        "d": [B; 21, 42, 15],
        "e": {
            "a": 12i8
        },
        "f": {
            "A": 13i16
        },
        "g": [21i8, 98i8]
    };

    assert_compound_eq!(struct_nbt, validation_nbt);

    let deserialized_struct: Foo = deserialize(&serialized_struct, Flavor::Uncompressed)
        .unwrap()
        .0;
    assert_eq!(deserialized_struct, test_struct);
}

#[test]
fn borrowed_serde() {
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct BorrowedData<'a> {
        bytes: Array<&'a [u8]>,
        string: &'a str,
        map: HashMap<&'a str, &'a str>,
    }

    const BYTES: &[u8] = &[1, 2, 3, 4, 5];

    let mut map = HashMap::new();
    map.insert("a", "b");
    map.insert("str", "string");

    let data = BorrowedData {
        bytes: Array::from(BYTES),
        string: "is my unsafe code sound?",
        map,
    };

    let serialized_struct = serialize(&data, None, Flavor::Uncompressed).unwrap();
    let struct_nbt = io::read_nbt(
        &mut Cursor::new(serialized_struct.clone()),
        Flavor::Uncompressed,
    )
    .unwrap()
    .0;

    let validation_nbt = compound! {
        "bytes": BYTES.to_vec(),
        "string": "is my unsafe code sound?",
        "map": {
            "a": "b",
            "str": "string"
        }
    };

    assert_compound_eq!(struct_nbt, validation_nbt);

    let deserialized_struct: BorrowedData<'_> =
        deserialize_from_buffer(&serialized_struct).unwrap().0;
    assert_eq!(deserialized_struct, data);

    // Just to make it clear what the drop order is
    drop(deserialized_struct);
    drop(serialized_struct);
}

#[test]
fn inlined_nbt() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Inlined {
        byte: NbtTag,
        short: NbtTag,
        int: NbtTag,
        long: NbtTag,
        float: NbtTag,
        double: NbtTag,
        byte_array: NbtTag,
        string: NbtTag,
        tag_list: NbtTag,
        tag_compound: NbtTag,
        int_array: NbtTag,
        long_array: NbtTag,
        list: NbtList,
        compound: NbtCompound,
        normal_field: i32,
    }

    let test_compound = SNBT_EDGE_CASES_VALIDATE.clone();
    let test_list = test_compound
        .get::<_, &NbtList>("chaotic_array")
        .unwrap()
        .clone();

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    let inlined = Inlined {
        byte: NbtTag::Byte(31),
        short: NbtTag::Short(-214),
        int: NbtTag::Int(328795),
        long: NbtTag::Long(-928592323532),
        float: NbtTag::Float(2.71828),
        double: NbtTag::Double(-3.14159),
        byte_array: NbtTag::ByteArray(vec![-1, 2, -3, 4]),
        string: NbtTag::String("foobar".to_owned()),
        tag_list: NbtTag::List(test_list.clone()),
        tag_compound: NbtTag::Compound(test_compound.clone()),
        int_array: NbtTag::IntArray(vec![-1_000_000, 2_000_000]),
        long_array: NbtTag::LongArray(Vec::new()),
        list: test_list.clone(),
        compound: test_compound.clone(),
        normal_field: 42,
    };

    let serialized_struct = serialize(&inlined, None, Flavor::Uncompressed).unwrap();
    let struct_nbt = io::read_nbt(
        &mut Cursor::new(serialized_struct.clone()),
        Flavor::Uncompressed,
    )
    .unwrap()
    .0;

    #[allow(clippy::excessive_precision, clippy::approx_constant)]
    let validation_nbt = compound! {
        "byte": 31i8,
        "short": -214i16,
        "int": 328795i32,
        "long": -928592323532i64,
        "float": 2.71828f32,
        "double": -3.14159f64,
        "byte_array": [B; -1, 2, -3, 4],
        "string": "foobar",
        "tag_list": test_list.clone(),
        "tag_compound": test_compound.clone(),
        "int_array": [I; -1_000_000, 2_000_000],
        "long_array": [L;],
        "list": test_list,
        "compound": test_compound,
        "normal_field": 42i32
    };

    assert_compound_eq!(struct_nbt, validation_nbt);

    let deserialized_struct: Inlined = deserialize(&serialized_struct, Flavor::Uncompressed)
        .unwrap()
        .0;
    assert_eq!(deserialized_struct, inlined);
}
