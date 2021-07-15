#![allow(unused_imports)]
use std::io::Cursor;

use quartz_nbt::io::{read_nbt, Flavor, NbtIoError};

#[cfg(feature = "serde")]
use quartz_nbt::serde::{deserialize, serialize};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};


#[cfg(feature = "serde")]
fn main() -> Result<(), NbtIoError> {
    use quartz_nbt::serde::Array;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Item {
        name: String,
        count: i32,
        metadata: Option<Vec<ItemMetadata>>,
        // If you want to serialize as a ByteArray (or similar) you need to wrap a vec in an Array
        extra_bytes: Array<Vec<i8>>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum ItemMetadata {
        ToolData { durability: i64, level: u8 },
        FoodData { hunger: f32 },
        BowData(i32),
        // Note: while you can serialize a Vec of enum variants, you cannot include unit variants with non-unit variants
        // this is because they are serialized different from the rest
        SpecialItem,
    }

    let item_1 = Item {
        name: "test".to_owned(),
        count: 12,
        metadata: Some(vec![ItemMetadata::ToolData {
            durability: 12,
            level: 9,
        }]),
        extra_bytes: vec![12, 125, 121].into(),
    };

    // You can serialize any struct or enum that implements Serialize into nbt
    let nbt_bytes = serialize(&item_1, None, Flavor::Uncompressed)?;

    let nbt = read_nbt(&mut Cursor::new(nbt_bytes.clone()), Flavor::Uncompressed)?.0;

    println!("nbt: {}", nbt);

    // You can also deserialze from a &[u8]
    let item_2: Item = deserialize(&nbt_bytes, Flavor::Uncompressed)?.0;

    println!("item: {:?}", item_2);
    assert_eq!(item_1, item_2);

    Ok(())
}


#[cfg(not(feature = "serde"))]
fn main() {}
