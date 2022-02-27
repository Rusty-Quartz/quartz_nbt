#![cfg(feature = "serde")]
//! This file, data.rs.in, and all files in benches/assets are from hematite_nbt:
//! https://github.com/PistonDevelopers/hematite_nbt.

extern crate criterion;
extern crate nbt;
extern crate quartz_nbt;
extern crate serde;

use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
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

mod data {
    use serde::{Deserialize, Serialize};
    include!("data.rs.in");
}

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
