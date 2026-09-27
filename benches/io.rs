//! Benchmarks for file I/O heavy operations: tag reading, tag writing, file collection and state storage.

use std::hint::black_box;
use std::path::{Path, PathBuf};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rayon::prelude::*;

use track_rename::state::State;
use track_rename::tags::{self, FileTags};
use track_rename::track::{Track, TrackMetadata};
use track_rename::utils;

/// Number of subdirectories in the generated benchmark library.
const LIBRARY_DIRECTORY_COUNT: usize = 50;

/// Fixture directories with complete tags so no warnings are printed while benchmarking.
const FIXTURE_DIRECTORIES: [&str; 2] = ["basic_tags", "extended_tags"];

/// Number of entries used for state database benchmarks.
const STATE_ENTRY_COUNT: usize = 10_000;

/// Return the path to the test fixture directory.
fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files")
}

/// Return all fixture files with complete tags.
fn fixture_files() -> Vec<PathBuf> {
    FIXTURE_DIRECTORIES
        .iter()
        .flat_map(|directory| {
            std::fs::read_dir(fixtures_root().join(directory))
                .expect("Failed to read fixture directory")
                .map(|entry| entry.expect("Failed to read fixture entry").path())
        })
        .collect()
}

/// Create a library of copied fixture files in the temp directory, reusing an existing one.
fn benchmark_library() -> PathBuf {
    let library_root = std::env::temp_dir().join("track-rename-bench-library");
    let files = fixture_files();
    let expected_count = files.len() * LIBRARY_DIRECTORY_COUNT;
    if utils::collect_tracks(&library_root).len() == expected_count {
        return library_root;
    }
    if library_root.exists() {
        std::fs::remove_dir_all(&library_root).expect("Failed to remove old benchmark library");
    }
    for index in 0..LIBRARY_DIRECTORY_COUNT {
        let directory = library_root.join(format!("Directory {index:02}"));
        std::fs::create_dir_all(&directory).expect("Failed to create benchmark directory");
        for file in &files {
            let name = file.file_name().expect("Fixture file should have a name");
            std::fs::copy(file, directory.join(name)).expect("Failed to copy fixture file");
        }
    }
    library_root
}

/// Benchmark reading tags from each supported file format.
fn bench_read_tags(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("read_tags");
    for extension in ["mp3", "aif", "flac"] {
        let path = fixtures_root().join(format!("extended_tags/Extended Tags - Song - 16-44.{extension}"));
        let track = Track::try_from_path(&path).expect("Failed to create track");
        group.bench_with_input(BenchmarkId::from_parameter(extension), &track, |bencher, track| {
            bencher.iter(|| FileTags::read(black_box(track), false).expect("Failed to read tags"));
        });
    }
    group.finish();
}

/// Benchmark writing tags, including preserving binary Serato frames.
fn bench_write_tags(criterion: &mut Criterion) {
    let temp_directory = std::env::temp_dir().join("track-rename-bench-write");
    std::fs::create_dir_all(&temp_directory).expect("Failed to create temp directory");

    let mut group = criterion.benchmark_group("write_tags");
    group.sample_size(20);
    for extension in ["mp3", "aif", "flac"] {
        let source = fixtures_root().join(format!("extended_tags/Extended Tags - Song - 16-44.{extension}"));
        let target = temp_directory.join(format!("Extended Tags - Song - 16-44.{extension}"));
        group.bench_function(extension, |bencher| {
            bencher.iter_batched(
                || {
                    std::fs::copy(&source, &target).expect("Failed to copy fixture");
                    let mut track = Track::try_from_path(&target).expect("Failed to create track");
                    let file_tags = track.read_tags(false).expect("Failed to read tags");
                    track.format_tags(&file_tags);
                    track.tags.formatted_album = String::from("Benchmark Album");
                    (track, file_tags)
                },
                |(track, mut file_tags)| tags::write_tags(&track, &mut file_tags).expect("Failed to write tags"),
                BatchSize::PerIteration,
            );
        });
    }
    group.finish();
}

/// Benchmark gathering audio files and the read + format pipeline.
fn bench_library(criterion: &mut Criterion) {
    let library_root = benchmark_library();
    let tracks = utils::collect_tracks(&library_root);

    let mut group = criterion.benchmark_group("library");
    group.sample_size(20);
    group.throughput(Throughput::Elements(tracks.len() as u64));
    group.bench_function("collect_tracks", |bencher| {
        bencher.iter(|| utils::collect_tracks(black_box(&library_root)));
    });
    group.bench_function("read_and_format/sequential", |bencher| {
        bencher.iter_batched(
            || tracks.clone(),
            |mut tracks| {
                for track in &mut tracks {
                    if let Some(file_tags) = track.read_tags(false) {
                        track.format_tags(&file_tags);
                    }
                }
                tracks
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("read_and_format/parallel", |bencher| {
        bencher.iter_batched(
            || tracks.clone(),
            |mut tracks| {
                tracks.par_iter_mut().for_each(|track| {
                    if let Some(file_tags) = track.read_tags(false) {
                        track.format_tags(&file_tags);
                    }
                });
                tracks
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

/// Benchmark state database inserts and lookups.
fn bench_state(criterion: &mut Criterion) {
    let paths: Vec<PathBuf> = (0..STATE_ENTRY_COUNT)
        .map(|index| PathBuf::from(format!("/Users/user/Music/Directory {}/Track {index}.mp3", index % 100)))
        .collect();
    let metadata = TrackMetadata {
        modified: 1_700_000_000,
        version: String::from(track_rename::track::VERSION),
    };
    let entries: Vec<(&Path, &TrackMetadata)> = paths.iter().map(|path| (path.as_path(), &metadata)).collect();

    let mut group = criterion.benchmark_group("state");
    group.sample_size(20);
    group.throughput(Throughput::Elements(STATE_ENTRY_COUNT as u64));
    group.bench_function("batch_insert", |bencher| {
        bencher.iter_batched(
            || State::open_in_memory().expect("Failed to open state"),
            |mut state| {
                state.batch_insert(&entries).expect("Failed to insert");
                state
            },
            BatchSize::PerIteration,
        );
    });

    let mut state = State::open_in_memory().expect("Failed to open state");
    state.batch_insert(&entries).expect("Failed to insert");
    group.bench_function("get", |bencher| {
        bencher.iter(|| {
            for path in &paths {
                black_box(state.get(path).expect("Failed to get state"));
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench_read_tags, bench_write_tags, bench_library, bench_state);
criterion_main!(benches);
