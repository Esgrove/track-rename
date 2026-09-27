//! Benchmarks for Serato tag parsing and crate file encoding.

use std::hint::black_box;
use std::path::{Path, PathBuf};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

use track_rename::serato::{SeratoCrate, SeratoData};

/// Test file that contains all Serato GEOB frames.
fn serato_mp3_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/extended_tags/Extended Tags - Song - 16-44.mp3")
}

/// Build a crate with the given number of realistic track paths.
fn synthetic_crate(track_count: usize) -> SeratoCrate {
    let mut serato_crate = SeratoCrate::new("Benchmark");
    serato_crate.add_tracks((0..track_count).map(|index| {
        PathBuf::from(format!(
            "Users/user/Dropbox/DJ MUSIC/House/Artist Name {index} - Track Title {index} (Extended Mix).aif"
        ))
    }));
    serato_crate
}

/// Benchmark parsing Serato GEOB frames from an ID3 tag.
fn bench_serato_tags(criterion: &mut Criterion) {
    let tag = id3::Tag::read_from_path(serato_mp3_path()).expect("Failed to read Serato test file");
    criterion.bench_function("serato_data_parse", |bencher| {
        bencher.iter(|| SeratoData::parse(black_box(&tag)));
    });
}

/// Benchmark reading and writing Serato crate files.
fn bench_serato_crate(criterion: &mut Criterion) {
    let test_crate = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/TEST.crate");
    let temp_dir = std::env::temp_dir().join("track-rename-bench-crate");
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let mut group = criterion.benchmark_group("serato_crate");
    group.bench_function("from_file/test_crate", |bencher| {
        bencher.iter(|| SeratoCrate::from_file(black_box(&test_crate)).expect("Failed to parse crate"));
    });

    for track_count in [1_000, 10_000] {
        let serato_crate = synthetic_crate(track_count);
        let crate_path = temp_dir.join(format!("bench_{track_count}.crate"));
        serato_crate
            .write_to_file(&crate_path)
            .expect("Failed to write benchmark crate");

        group.throughput(Throughput::Elements(track_count as u64));
        group.bench_with_input(
            BenchmarkId::new("to_bytes", track_count),
            &serato_crate,
            |bencher, input| {
                bencher.iter(|| input.to_bytes());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("from_file", track_count),
            &crate_path,
            |bencher, path| {
                bencher.iter(|| SeratoCrate::from_file(black_box(path)).expect("Failed to parse crate"));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_serato_tags, bench_serato_crate);
criterion_main!(benches);
