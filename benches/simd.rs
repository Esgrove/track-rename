//! Prototype comparison of scalar kernels against explicit `fearless_simd` versions.
//!
//! The scalar functions replicate the current private implementations in `src/serato`.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fearless_simd::prelude::*;
use fearless_simd::{Level, dispatch, u8x16};
use fearless_simd_macros::simd;

/// Number of frequency blocks in a Serato overview tag.
const OVERVIEW_BLOCK_COUNT: usize = 240;

/// Size of one Serato overview frequency block.
const OVERVIEW_BLOCK_SIZE: usize = 16;

/// Current `decode_utf16be` implementation from `serato_crate.rs`.
fn decode_utf16be_scalar(data: &[u8]) -> Option<String> {
    let units: Vec<u16> = data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect();
    String::from_utf16(&units).ok()
}

/// Scalar decode with an ASCII fast path that skips the intermediate `u16` buffer.
fn decode_utf16be_ascii_scalar(data: &[u8]) -> Option<String> {
    let (pairs, _) = data.as_chunks::<2>();
    if pairs.iter().all(|pair| pair[0] == 0 && pair[1] < 0x80) {
        let bytes: Vec<u8> = pairs.iter().map(|pair| pair[1]).collect();
        return String::from_utf8(bytes).ok();
    }
    decode_utf16be_scalar(data)
}

/// SIMD decode with an ASCII fast path using 16-byte lanes.
#[simd]
fn decode_utf16be_ascii_simd<S: Simd>(simd: S, data: &[u8]) -> Option<String> {
    let mut bytes = Vec::with_capacity(data.len() / 2);
    let (chunks, remainder) = data.as_chunks::<32>();
    for chunk in chunks {
        let first = u8x16::from_slice(simd, &chunk[..16]);
        let second = u8x16::from_slice(simd, &chunk[16..]);
        let high = first.unzip_low(second);
        let low = first.unzip_high(second);
        let non_ascii = high.simd_gt(0) | low.simd_ge(0x80);
        if non_ascii.any_true() {
            return decode_utf16be_scalar(data);
        }
        bytes.extend_from_slice(low.as_slice());
    }
    for pair in remainder.as_chunks::<2>().0 {
        if pair[0] != 0 || pair[1] >= 0x80 {
            return decode_utf16be_scalar(data);
        }
        bytes.push(pair[1]);
    }
    String::from_utf8(bytes).ok()
}

/// Current newline stripping from `markers.rs`.
fn strip_newlines_scalar(data: &[u8]) -> Vec<u8> {
    let mut cleaned = Vec::with_capacity(data.len());
    cleaned.extend(data.iter().filter(|&&byte| byte != b'\n'));
    cleaned
}

/// Scalar newline stripping that copies whole runs between line feeds.
fn strip_newlines_split(data: &[u8]) -> Vec<u8> {
    let mut cleaned = Vec::with_capacity(data.len());
    for part in data.split(|&byte| byte == b'\n') {
        cleaned.extend_from_slice(part);
    }
    cleaned
}

/// SIMD newline stripping that bulk-copies 16-byte blocks without line feeds.
#[simd]
fn strip_newlines_simd<S: Simd>(simd: S, data: &[u8]) -> Vec<u8> {
    let newline = u8x16::splat(simd, b'\n');
    let mut cleaned = Vec::with_capacity(data.len());
    let (chunks, remainder) = data.as_chunks::<16>();
    for chunk in chunks {
        let vector = u8x16::from_slice(simd, chunk);
        if vector.simd_eq(newline).any_true() {
            cleaned.extend(chunk.iter().filter(|&&byte| byte != b'\n'));
        } else {
            cleaned.extend_from_slice(chunk);
        }
    }
    cleaned.extend(remainder.iter().filter(|&&byte| byte != b'\n'));
    cleaned
}

/// Current overview averaging from `overview.rs`, reducing 16 bands to 8.
fn overview_average_scalar(blocks: &[[u8; OVERVIEW_BLOCK_SIZE]]) -> (Vec<[u8; 8]>, u8) {
    let averaged: Vec<[u8; 8]> = blocks
        .iter()
        .map(|block| {
            std::array::from_fn(|index| {
                u16::midpoint(u16::from(block[2 * index]), u16::from(block[2 * index + 1])) as u8
            })
        })
        .collect();
    let max_value = averaged.iter().flatten().copied().max().unwrap_or(1);
    (averaged, max_value)
}

/// SIMD overview averaging using even/odd lane deinterleaving.
#[simd]
fn overview_average_simd<S: Simd>(simd: S, blocks: &[[u8; OVERVIEW_BLOCK_SIZE]]) -> (Vec<[u8; 8]>, u8) {
    let mut averaged = Vec::with_capacity(blocks.len());
    let mut maximum = u8x16::splat(simd, 0);
    for pair in blocks.chunks(2) {
        let first = u8x16::from_slice(simd, &pair[0]);
        let second = pair.get(1).map_or(first, |block| u8x16::from_slice(simd, block));
        let even = first.unzip_low(second);
        let odd = first.unzip_high(second);
        let (even_low, even_high) = even.widen();
        let (odd_low, odd_high) = odd.widen();
        let low = (even_low + odd_low) >> 1;
        let high = (even_high + odd_high) >> 1;
        let mut output = [0u8; 16];
        for (index, value) in low.as_slice().iter().chain(high.as_slice()).enumerate() {
            output[index] = *value as u8;
        }
        let output_vector = u8x16::from_slice(simd, &output);
        maximum = maximum.max(output_vector);
        averaged.push(output[..8].try_into().expect("Slice has eight elements"));
        if pair.len() == 2 {
            averaged.push(output[8..].try_into().expect("Slice has eight elements"));
        }
    }
    let max_value = maximum.reduce_max();
    (averaged, if blocks.is_empty() { 1 } else { max_value })
}

/// Generate UTF-16BE encoded track paths similar to a crate file.
fn utf16_paths(count: usize, non_ascii: bool) -> Vec<Vec<u8>> {
    (0..count)
        .map(|index| {
            let artist = if non_ascii { "Beyoncé" } else { "Beyonce" };
            format!("Users/user/Dropbox/DJ MUSIC/House/{artist} {index} - Track Title {index} (Extended Mix).aif")
                .encode_utf16()
                .flat_map(u16::to_be_bytes)
                .collect()
        })
        .collect()
}

/// Generate base64 text wrapped with line feeds every 72 characters like Serato Markers2.
fn wrapped_base64(length: usize) -> Vec<u8> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut data = Vec::with_capacity(length + length / 72);
    for index in 0..length {
        if index > 0 && index % 72 == 0 {
            data.push(b'\n');
        }
        data.push(alphabet[index % alphabet.len()]);
    }
    data
}

/// Generate a deterministic overview waveform.
fn overview_blocks() -> Vec<[u8; OVERVIEW_BLOCK_SIZE]> {
    (0..OVERVIEW_BLOCK_COUNT)
        .map(|block| std::array::from_fn(|band| ((block * 7 + band * 13) % 256) as u8))
        .collect()
}

/// Verify that SIMD and scalar implementations agree before measuring.
fn verify_equivalence(level: Level) {
    for paths in [utf16_paths(64, false), utf16_paths(64, true)] {
        for path in &paths {
            let expected = decode_utf16be_scalar(path);
            assert_eq!(decode_utf16be_ascii_scalar(path), expected);
            assert_eq!(
                dispatch!(level, simd => decode_utf16be_ascii_simd(simd, path)),
                expected
            );
        }
    }
    let base64 = wrapped_base64(4096);
    let expected = strip_newlines_scalar(&base64);
    assert_eq!(strip_newlines_split(&base64), expected);
    assert_eq!(dispatch!(level, simd => strip_newlines_simd(simd, &base64)), expected);

    let blocks = overview_blocks();
    assert_eq!(
        dispatch!(level, simd => overview_average_simd(simd, &blocks)),
        overview_average_scalar(&blocks)
    );
}

/// Benchmark UTF-16BE decoding of crate track paths.
fn bench_utf16(criterion: &mut Criterion, level: Level) {
    let mut group = criterion.benchmark_group("simd/utf16be_decode");
    for (name, non_ascii) in [("ascii", false), ("non_ascii", true)] {
        let paths = utf16_paths(1_000, non_ascii);
        let total_bytes: usize = paths.iter().map(Vec::len).sum();
        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(BenchmarkId::new("scalar", name), &paths, |bencher, paths| {
            bencher.iter(|| {
                for path in paths {
                    black_box(decode_utf16be_scalar(black_box(path)));
                }
            });
        });
        group.bench_with_input(BenchmarkId::new("ascii_scalar", name), &paths, |bencher, paths| {
            bencher.iter(|| {
                for path in paths {
                    black_box(decode_utf16be_ascii_scalar(black_box(path)));
                }
            });
        });
        group.bench_with_input(BenchmarkId::new("ascii_simd", name), &paths, |bencher, paths| {
            bencher.iter(|| {
                for path in paths {
                    black_box(dispatch!(level, simd => decode_utf16be_ascii_simd(simd, black_box(path))));
                }
            });
        });
    }
    group.finish();
}

/// Benchmark stripping line feeds from Markers2 base64 payloads.
fn bench_newlines(criterion: &mut Criterion, level: Level) {
    let mut group = criterion.benchmark_group("simd/strip_newlines");
    for length in [512, 4096] {
        let data = wrapped_base64(length);
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::new("scalar_filter", length), &data, |bencher, data| {
            bencher.iter(|| strip_newlines_scalar(black_box(data)));
        });
        group.bench_with_input(BenchmarkId::new("scalar_split", length), &data, |bencher, data| {
            bencher.iter(|| strip_newlines_split(black_box(data)));
        });
        group.bench_with_input(BenchmarkId::new("simd", length), &data, |bencher, data| {
            bencher.iter(|| dispatch!(level, simd => strip_newlines_simd(simd, black_box(data))));
        });
    }
    group.finish();
}

/// Benchmark overview waveform averaging and maximum search.
fn bench_overview(criterion: &mut Criterion, level: Level) {
    let blocks = overview_blocks();
    let mut group = criterion.benchmark_group("simd/overview_average");
    group.throughput(Throughput::Bytes((blocks.len() * OVERVIEW_BLOCK_SIZE) as u64));
    group.bench_function("scalar", |bencher| {
        bencher.iter(|| overview_average_scalar(black_box(&blocks)));
    });
    group.bench_function("simd", |bencher| {
        bencher.iter(|| dispatch!(level, simd => overview_average_simd(simd, black_box(&blocks))));
    });
    group.finish();
}

/// Run all SIMD prototype benchmarks after checking correctness.
fn bench_simd(criterion: &mut Criterion) {
    let level = Level::new();
    verify_equivalence(level);
    bench_utf16(criterion, level);
    bench_newlines(criterion, level);
    bench_overview(criterion, level);
}

criterion_group!(benches, bench_simd);
criterion_main!(benches);
