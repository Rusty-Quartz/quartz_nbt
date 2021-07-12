mod assets;
use assets::*;
use quartz_nbt::{
    io::{self, Flavor},
    serde::{deserialize, serialize},
    NbtCompound,
    NbtList,
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
