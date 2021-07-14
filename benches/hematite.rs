#![cfg(feature = "serde")]
//! This file, data.rs.in, and all files in benches/assets are from hematite_nbt:
//! https://github.com/PistonDevelopers/hematite_nbt.

extern crate criterion;
extern crate nbt;
extern crate quartz_nbt;
extern crate serde;

use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use nbt::{de::from_gzip_reader, ser::to_writer};
use quartz_nbt::{
    io::{read_nbt, write_nbt, Flavor},
    serde::{deserialize_from, serialize_into_unchecked},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    time::Duration,
};

mod data {
    use serde::{Deserialize, Serialize};
    include!("data.rs.in");
}

fn hematite_bench<T>(filename: &str, c: &mut Criterion)
where T: DeserializeOwned + Serialize {
    let mut file = File::open(filename).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let mut src = std::io::Cursor::new(&contents[..]);
    file.seek(SeekFrom::Start(0)).unwrap();
    let nbt_struct: T = from_gzip_reader(&mut file).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let nbt_blob = nbt::Blob::from_gzip_reader(&mut file).unwrap();

    let mut group = c.benchmark_group(filename);
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Bytes(contents.len() as u64));
    group.bench_function("Hematite: Deserialize As Struct", |b| {
        b.iter(|| {
            src.seek(SeekFrom::Start(0)).unwrap();
            let _: T = from_gzip_reader(&mut src).unwrap();
        })
    });
    group.bench_function("Hematite: Deserialize As Blob", |b| {
        b.iter(|| {
            src.seek(SeekFrom::Start(0)).unwrap();
            nbt::Blob::from_gzip_reader(&mut src).unwrap();
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

    let mut group = c.benchmark_group(filename);
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Bytes(contents.len() as u64));
    group.bench_function("Quartz: Deserialize As Struct", |b| {
        b.iter(|| {
            src.seek(SeekFrom::Start(0)).unwrap();
            black_box(
                deserialize_from::<_, T>(&mut src, Flavor::GzCompressed)
                    .unwrap()
                    .0,
            );
        })
    });
    group.bench_function("Quartz: Deserialize As Compound", |b| {
        b.iter(|| {
            src.seek(SeekFrom::Start(0)).unwrap();
            black_box(read_nbt(&mut src, Flavor::GzCompressed).unwrap().0);
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
    quartz_bench::<data::Big1>("benches/assets/big1.nbt", c);
    hematite_bench::<data::PlayerData>("benches/assets/simple_player.dat", c);
    quartz_bench::<data::PlayerData>("benches/assets/simple_player.dat", c);
    hematite_bench::<data::PlayerData>("benches/assets/complex_player.dat", c);
    quartz_bench::<data::PlayerData>("benches/assets/complex_player.dat", c);
    hematite_bench::<data::Level>("benches/assets/level.dat", c);
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
