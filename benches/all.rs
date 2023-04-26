#![cfg(feature = "serde")]
//! This file, data.rs.in, and all files in benches/assets are from hematite_nbt:
//! https://github.com/PistonDevelopers/hematite_nbt.

extern crate criterion;
extern crate nbt;
extern crate quartz_nbt;
extern crate serde;

use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use fastnbt::{from_bytes, stream::Parser, ByteArray, LongArray};
use flate2::read::GzDecoder;
use nbt::{de::from_gzip_reader, from_reader, ser::to_writer};
use quartz_nbt::{
    io::{read_nbt, write_nbt, Flavor},
    serde::{deserialize_from, deserialize_from_buffer, serialize_into_unchecked},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::File,
    io::{self, Cursor, Read, Seek, SeekFrom},
    time::Duration,
};

fn inflate(buf: &[u8]) -> Vec<u8> {
    let mut decoder = GzDecoder::new(buf);
    let mut dest = Vec::new();
    decoder.read_to_end(&mut dest).unwrap();
    dest
}

fn hematite_bench<T>(filename: &str, c: &mut Criterion)
where T: DeserializeOwned + Serialize {
    let mut file = File::open(filename).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let mut src = Cursor::new(&contents[..]);
    file.seek(SeekFrom::Start(0)).unwrap();
    let nbt_struct: T = from_gzip_reader(&mut file).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let nbt_blob = nbt::Blob::from_gzip_reader(&mut file).unwrap();
    let uncompressed = inflate(&contents);
    let mut uncompressed_src = Cursor::new(&uncompressed[..]);

    let mut group = c.benchmark_group(filename);
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Bytes(contents.len() as u64));
    group.bench_function("Hematite: Deserialize As Struct (Compressed)", |b| {
        b.iter(|| {
            src.seek(SeekFrom::Start(0)).unwrap();
            let _: T = from_gzip_reader(&mut src).unwrap();
        })
    });
    group.bench_function("Hematite: Deserialize As Struct (Uncompressed)", |b| {
        b.iter(|| {
            uncompressed_src.seek(SeekFrom::Start(0)).unwrap();
            let _: T = from_reader(&mut uncompressed_src).unwrap();
        })
    });
    group.bench_function("Hematite: Deserialize As Blob (Compressed)", |b| {
        b.iter(|| {
            src.seek(SeekFrom::Start(0)).unwrap();
            nbt::Blob::from_gzip_reader(&mut src).unwrap();
        })
    });
    group.bench_function("Hematite: Deserialize As Blob (Uncompressed)", |b| {
        b.iter(|| {
            uncompressed_src.seek(SeekFrom::Start(0)).unwrap();
            nbt::Blob::from_reader(&mut uncompressed_src).unwrap();
        })
    });
    group.bench_function("Hematite: Serialize As Struct", |b| {
        b.iter(|| {
            to_writer(&mut io::sink(), &nbt_struct, None).unwrap();
        })
    });
    group.bench_function("Hematite: Serialize As Blob", |b| {
        b.iter(|| {
            nbt_blob.to_writer(&mut io::sink()).unwrap();
        })
    });
    group.finish();
}

fn fastnbt_bench<T>(filename: &str, c: &mut Criterion)
where T: DeserializeOwned + Serialize {
    let mut file = File::open(filename).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let uncompressed = inflate(&contents);

    let mut group = c.benchmark_group(filename);
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Bytes(contents.len() as u64));
    group.bench_function("Fastnbt: Deserialize As Struct (Uncompressed)", |b| {
        b.iter(|| {
            black_box(from_bytes::<T>(&uncompressed).unwrap());
        })
    });
    group.bench_function("Fastnbt: Deserialize As Compound (Uncompressed)", |b| {
        b.iter(|| {
            black_box(Parser::new(Cursor::new(&uncompressed[..])).next().unwrap());
        })
    });
    group.finish();
}

fn quartz_bench<T>(filename: &str, c: &mut Criterion)
where T: DeserializeOwned + Serialize {
    let mut file = File::open(filename).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let mut src = std::io::Cursor::new(&contents[..]);
    file.seek(SeekFrom::Start(0)).unwrap();
    let nbt_struct: T = deserialize_from(&mut file, Flavor::GzCompressed).unwrap().0;
    file.seek(SeekFrom::Start(0)).unwrap();
    let nbt_compound = read_nbt(&mut file, Flavor::GzCompressed).unwrap().0;
    let uncompressed = inflate(&contents);
    let mut uncompressed_src = Cursor::new(&uncompressed[..]);

    let mut group = c.benchmark_group(filename);
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Bytes(contents.len() as u64));
    group.bench_function("Quartz: Deserialize As Struct (Compressed)", |b| {
        b.iter(|| {
            src.seek(SeekFrom::Start(0)).unwrap();
            black_box(
                deserialize_from::<_, T>(&mut src, Flavor::GzCompressed)
                    .unwrap()
                    .0,
            );
        })
    });
    group.bench_function("Quartz: Deserialize As Struct (Uncompressed)", |b| {
        b.iter(|| {
            black_box(deserialize_from_buffer::<T>(&uncompressed).unwrap().0);
        })
    });
    group.bench_function("Quartz: Deserialize As Compound (Compressed)", |b| {
        b.iter(|| {
            src.seek(SeekFrom::Start(0)).unwrap();
            black_box(read_nbt(&mut src, Flavor::GzCompressed).unwrap().0);
        })
    });
    group.bench_function("Quartz: Deserialize As Compound (Uncompressed)", |b| {
        b.iter(|| {
            uncompressed_src.seek(SeekFrom::Start(0)).unwrap();
            black_box(
                read_nbt(&mut uncompressed_src, Flavor::Uncompressed)
                    .unwrap()
                    .0,
            );
        })
    });
    group.bench_function("Quartz: Serialize As Struct", |b| {
        b.iter(|| {
            serialize_into_unchecked(&mut io::sink(), &nbt_struct, None, Flavor::Uncompressed)
                .unwrap();
        })
    });
    group.bench_function("Quartz: Serialize As Compound", |b| {
        b.iter(|| {
            write_nbt(&mut io::sink(), None, &nbt_compound, Flavor::Uncompressed).unwrap();
        })
    });
    group.finish();
}

fn bench(c: &mut Criterion) {
    hematite_bench::<data::Big1>("benches/assets/big1.nbt", c);
    fastnbt_bench::<data::Big1<ByteArray, LongArray>>("benches/assets/big1.nbt", c);
    quartz_bench::<data::Big1>("benches/assets/big1.nbt", c);
    hematite_bench::<data::PlayerData>("benches/assets/simple_player.dat", c);
    fastnbt_bench::<data::PlayerData>("benches/assets/simple_player.dat", c);
    quartz_bench::<data::PlayerData>("benches/assets/simple_player.dat", c);
    hematite_bench::<data::PlayerData>("benches/assets/complex_player.dat", c);
    fastnbt_bench::<data::PlayerData>("benches/assets/complex_player.dat", c);
    quartz_bench::<data::PlayerData>("benches/assets/complex_player.dat", c);
    hematite_bench::<data::Level>("benches/assets/level.dat", c);
    fastnbt_bench::<data::Level>("benches/assets/level.dat", c);
    quartz_bench::<data::Level>("benches/assets/level.dat", c);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(500)
        .warm_up_time(Duration::from_secs(1));
    targets = bench
}
criterion_main!(benches);

mod data {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct Small1 {
        name: String,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct Small2Sub {
        #[serde(rename = "1")]
        one: i8,
        #[serde(rename = "2")]
        two: i16,
        #[serde(rename = "3")]
        three: i32,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct Small2 {
        aaa: Small2Sub,
        bbb: Small2Sub,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct Small3Sub {
        ccc: i32,
        name: String,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct Small3 {
        bbb: Vec<Small3Sub>,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct Small4Sub {
        aaa: i8,
        bbb: i8,
        ccc: i8,
        ddd: i8,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct Small4 {
        c1: Small4Sub,
        c2: Small4Sub,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Big1Sub1 {
        name: String,
        #[serde(rename = "created-on")]
        created_on: i64,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Big1Sub2 {
        name: String,
        value: f32,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Big1Sub3 {
        ham: Big1Sub2,
        egg: Big1Sub2,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Big1<I = Vec<i8>, L = Vec<i64>> {
        #[serde(rename = "listTest (compound)")]
        list_test_compound: Vec<Big1Sub1>,
        #[serde(rename = "longTest")]
        long_test: i64,
        #[serde(rename = "shortTest")]
        short_test: i32,
        #[serde(rename = "byteTest")]
        byte_test: i8,
        #[serde(rename = "floatTest")]
        float_test: f64,
        #[serde(rename = "nested compound test")]
        nested_compound_test: Big1Sub3,
        #[serde(
            rename = "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with \
                      n=0 (0, 62, 34, 16, 8, ...))"
        )]
        byte_array_test: I, // [i8; 1000] does not implement PartialEq.
        #[serde(rename = "stringTest")]
        string_test: String,
        #[serde(rename = "listTest (long)")]
        list_test_long: L,
        #[serde(rename = "doubleTest")]
        double_test: f64,
        #[serde(rename = "intTest")]
        int_test: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Level {
        #[serde(rename = "Data")]
        pub data: LevelData,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LevelData {
        #[serde(rename = "RandomSeed")]
        seed: i64,
        #[serde(rename = "DayTime")]
        daytime: i64,
        #[serde(rename = "Player")]
        player: PlayerData,
        initialized: bool,
        version: i32,
        #[serde(rename = "allowCommands")]
        allow_commands: bool,
        #[serde(rename = "LastPlayed")]
        last_played: i64,
        #[serde(rename = "SpawnZ")]
        spawn_z: i32,
        #[serde(rename = "SpawnX")]
        spawn_x: i32,
        #[serde(rename = "SpawnY")]
        spawn_y: i32,
        #[serde(rename = "LevelName")]
        name: String,
        #[serde(rename = "MapFeatures")]
        map_features: bool,

        #[serde(rename = "GameType")]
        game_type: i32,
        #[serde(rename = "Difficulty")]
        difficulty: i8,
        #[serde(rename = "DifficultyLocked")]
        difficulty_locked: bool,

        #[serde(rename = "generatorName")]
        generator_name: String,
        #[serde(rename = "generatorOptions")]
        generator_options: String,
        #[serde(rename = "generatorVersion")]
        generator_version: i32,

        #[serde(rename = "Time")]
        time: i64,
        #[serde(rename = "clearWeatherTime")]
        clear_weather_time: i32,
        #[serde(rename = "thunderTime")]
        thunder_time: i32,
        #[serde(rename = "rainTime")]
        rain_time: i32,

        thundering: bool,
        raining: bool,
        hardcore: bool,

        #[serde(rename = "GameRules")]
        game_rules: GameRules,
        #[serde(rename = "SizeOnDisk")]
        size_on_disk: i64,

        #[serde(rename = "BorderCenterX")]
        border_center_x: f64,
        #[serde(rename = "BorderCenterY")]
        border_center_y: Option<f64>,
        #[serde(rename = "BorderCenterZ")]
        border_center_z: f64,
        #[serde(rename = "BorderWarningBlocks")]
        border_warning_blocks: f64,
        #[serde(rename = "BorderWarningTime")]
        border_warning_time: f64,
        #[serde(rename = "BorderSafeZone")]
        border_safe_zone: f64,
        #[serde(rename = "BorderSize")]
        border_size: f64,
        #[serde(rename = "BorderSizeLerpTarget")]
        border_size_lerp_target: f64,
        #[serde(rename = "BorderSizeLerpTime")]
        border_size_lerp_time: i64,
        #[serde(rename = "BorderDamagePerBlock")]
        border_damage_per_block: f64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PlayerData {
        #[serde(rename = "PersistentId")]
        persistant_id: Option<i32>,
        #[serde(rename = "playerGameType")]
        game_type: i32,
        abilities: PlayerAbilityData,
        #[serde(rename = "Score")]
        score: Option<i32>,

        #[serde(rename = "Dimension")]
        dimension: i32,
        #[serde(rename = "OnGround")]
        on_ground: bool,
        #[serde(rename = "FallDistance")]
        fall_distance: f32,
        #[serde(rename = "Motion")]
        motion: Vec<f64>, // [f64; 3]
        #[serde(rename = "Pos")]
        position: Vec<f64>, // [f64; 3]
        #[serde(rename = "Rotation")]
        rotation: Vec<f32>, // [f32; 2]

        #[serde(rename = "SpawnX")]
        spawn_x: i32,
        #[serde(rename = "SpawnY")]
        spawn_y: i32,
        #[serde(rename = "SpawnZ")]
        spawn_z: i32,
        #[serde(rename = "SpawnForced")]
        spawn_forced: Option<bool>,

        #[serde(rename = "PortalCooldown")]
        portal_cooldown: Option<i32>,
        #[serde(rename = "Invulnerable")]
        invulnerable: Option<bool>,

        #[serde(rename = "AttackTime")]
        attack_time: Option<i16>,
        #[serde(rename = "HurtTime")]
        hurt_time: i16,
        #[serde(rename = "HurtByTimestamp")]
        hurt_by: Option<i32>,
        #[serde(rename = "DeathTime")]
        death_time: i16,
        #[serde(rename = "Sleeping")]
        sleeping: bool,
        #[serde(rename = "SleepTimer")]
        sleep_timer: i16,

        #[serde(rename = "Health")]
        health: i16,
        #[serde(rename = "HealF")]
        heal: Option<f32>,
        #[serde(rename = "foodLevel")]
        food_level: i32,
        #[serde(rename = "foodTickTimer")]
        food_tick_timer: i32,
        #[serde(rename = "foodSaturationLevel")]
        food_saturation_level: f32,
        #[serde(rename = "foodExhaustionLevel")]
        food_exhaustion_level: f32,

        #[serde(rename = "Fire")]
        fire: i16,
        #[serde(rename = "Air")]
        air: i16,

        #[serde(rename = "XpP")]
        xp_p: f32,
        #[serde(rename = "XpLevel")]
        xp_level: i32,
        #[serde(rename = "XpTotal")]
        xp_total: i32,
        #[serde(rename = "XpSeed")]
        xp_seed: Option<i32>,

        #[serde(rename = "Inventory")]
        inventory: Vec<InventoryEntry>,
        #[serde(rename = "EnderItems")]
        ender_items: Vec<i8>,

        #[serde(rename = "SelectedItemSlot")]
        selected_item_slot: Option<i32>,
        #[serde(rename = "SelectedItem")]
        selected_item: Option<InventoryEntry>,
        #[serde(rename = "UUIDLeast")]
        uuid_least: Option<i64>,
        #[serde(rename = "UUIDMost")]
        uuid_most: Option<i64>,
        #[serde(rename = "AbsorptionAmount")]
        absorbtion_amount: Option<f32>,
        #[serde(rename = "Attributes")]
        attributes: Option<Vec<AttributeEntry>>,
        #[serde(rename = "ActiveEffects")]
        active_effects: Option<Vec<ActiveEffect>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PlayerAbilityData {
        invulnerable: bool,
        instabuild: bool,
        flying: bool,
        #[serde(rename = "flySpeed")]
        fly_speed: f32,
        #[serde(rename = "walkSpeed")]
        walk_speed: f32,
        #[serde(rename = "mayBuild")]
        may_build: bool,
        #[serde(rename = "mayfly")]
        may_fly: bool,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct InventoryEntry {
        id: String,
        #[serde(rename = "Slot")]
        slot: Option<i8>,
        #[serde(rename = "Count")]
        count: i8,
        #[serde(rename = "Damage")]
        damage: i16,
        #[serde(rename = "tag")]
        info: Option<InventoryEntryInfo>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct InventoryEntryInfo {
        display: Option<InventoryEntryDisplay>,
        #[serde(rename = "RepairCost")]
        repair_cost: Option<i32>,
        #[serde(rename = "ench")]
        enchantments: Vec<Enchantment>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct InventoryEntryDisplay {
        #[serde(rename = "Name")]
        name: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Enchantment {
        id: i16,
        #[serde(rename = "lvl")]
        level: i16,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EnderItemsEntry {
        id: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AttributeEntry {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Base")]
        base: f64,
        #[serde(rename = "Modifiers")]
        modifiers: Option<Vec<AttributeModifier>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AttributeModifier {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Amount")]
        amount: f64,
        #[serde(rename = "Operation")]
        operation: i32,
        #[serde(rename = "UUIDLeast")]
        uuid_least: i64,
        #[serde(rename = "UUIDMost")]
        uuid_most: i64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ActiveEffect {
        #[serde(rename = "Id")]
        id: i8,
        #[serde(rename = "Duration")]
        base: i32,
        #[serde(rename = "Ambient")]
        ambient: bool,
        #[serde(rename = "Amplifier")]
        amplifier: bool,
        #[serde(rename = "ShowParticles")]
        show_particles: bool,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GameRules {
        #[serde(rename = "doMobLoot")]
        mob_loot: String,
        #[serde(rename = "doTileDrops")]
        tile_drops: String,
        #[serde(rename = "doFireTick")]
        fire_tick: String,
        #[serde(rename = "mobGriefing")]
        mob_griefing: String,
        #[serde(rename = "commandBlockOutput")]
        command_block_output: String,
        #[serde(rename = "doMobSpawning")]
        mob_spawning: String,
        #[serde(rename = "keepInventory")]
        keep_inventory: String,
        #[serde(rename = "showDeathMessages")]
        show_death_messages: String,
        #[serde(rename = "doEntityDrops")]
        entity_drops: String,
        #[serde(rename = "naturalRegeneration")]
        natural_regeneration: String,
        #[serde(rename = "logAdminCommands")]
        log_admin_commands: String,
        #[serde(rename = "doDaylightCycle")]
        daylight_cycle: String,
        #[serde(rename = "sendCommandFeedback")]
        send_command_feedback: String,
        #[serde(rename = "randomTickSpeed")]
        random_tick_speed: String,
        #[serde(rename = "reducedDebugInfo")]
        reduced_debug_info: String,
    }
}
