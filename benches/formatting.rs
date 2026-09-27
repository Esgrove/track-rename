//! Benchmarks for artist, title, album, genre and filename formatting.

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

use track_rename::{formatting, genre, utils};

/// Representative artist and title pairs taken from the formatting test data.
const TAG_CORPUS: &[(&str, &str)] = &[
    ("Janet Jackson", "If (Kaytranada Edition (Live Set Version)"),
    ("Dynasty", "Still In Love) (Dave Lee Original Vibe Mix)"),
    (
        "Seige",
        "Holla Remix (featuring Busta Rhymes, Little Brother, Kurupt, Crooked I, and Willie B)",
    ),
    ("Fanu & Ane Brun", "Taivaita ja Tarinoita (feat. Ane Brun)"),
    ("Rihanna feat. Drake", "Whats My Name (Trayze Intro) feat. Drake"),
    ("Audiojack", "Stay Glued (Feat Kevin Knapp - Zds Remix)"),
    ("Mike Dunn & Riva Starr", "Feel The Heat feat. Mike Dunn (Extended Mix)"),
    (
        "DJ Chus & David Penn",
        "Will I (Discover Love - feat. Concha Buika - Mediterranean Club Mix)",
    ),
    (
        "Daft Punk, Pharrell Williams & Nile Rodgers",
        "Get Lucky (Drumless Edition) (feat. Pharrell Williams and Nile Rodgers)",
    ),
    ("Major Lazer (feat. Laidback Luke & Ms. Dynamite)", "Sweat (Trayze Qh)"),
    ("ASAP Ferg x A-Ha", "Plain Jane (Nick Bike Edit + Acap In & Out)[Clean]"),
    ("Aazar ft. French Montana", "The Carnival (Inst)"),
    ("Aitch & AJ Tracey ft. Tay Keith", "Rain (DJcity Intro - Clean)"),
    ("Big Sean", "Dance (A$$) - Tall Boys Remix (DJcity Intro - Dirty)"),
    ("Big Sean W/Taku", "Dance (A$$)"),
    (
        "Beyonce",
        "Beyonce - Texas Hold Em (Flipout X KON _I Want To Thank You_ Edit)(Instrumental)",
    ),
    ("Evelyn 'Champagne' King", "Im In Love (TRAYZE QUANT REMASTER)"),
    ("Giacca & Flores", "New Monday (Original Mix/Cyberkid Re-Edit)"),
    (
        "L.B.C. Crew (Feat. - Tray D & South Sentrel)",
        "Beware Of My Crew (Dj Pooh Remix Instrumental)",
    ),
    ("S'hustryi Beats", "Force (feat.Theodor) (Remaster)"),
    ("Nina Sky", "Move Ya Body (Trayze Acap-In)"),
    ("Beyoncé", "Break My Soul (Trayze Acap-In Out)"),
    (
        "Talib Kweli feat. Anny Dobson & William Taylor & Nina Simone)",
        "Get By (Trayze Resist Acapella In Edit) (130 8b)",
    ),
    ("Various Artists", "Temptations, The - My Girl (Extended Mix) (120)"),
    ("MISSY ELLIOTT", "GET UR FREAK ON (DIRTY INTRO)"),
    (
        "Spiller & Sophie Ellis-Bextor",
        "Groovejet [If This Ain't Love] {Riva Starr Dub}.mp3",
    ),
    (
        "The Weeknd",
        "Save Your Tears feat Ariana Grande - Flipout Purple Disco Machine Edit",
    ),
    ("Kings Of Tomorrow", "Finally (Sandy Rivera's Classic Mix)"),
];

/// Individual cases covering the main code paths of tag formatting.
const TAG_CASES: &[(&str, &str, &str)] = &[
    ("simple", "Kings Of Tomorrow", "Finally (Sandy Rivera's Classic Mix)"),
    (
        "feat",
        "Daft Punk, Pharrell Williams & Nile Rodgers",
        "Get Lucky (Drumless Edition) (feat. Pharrell Williams and Nile Rodgers)",
    ),
    (
        "messy",
        "Talib Kweli feat. Anny Dobson & William Taylor & Nina Simone)",
        "Get By [Trayze Resist Acap In Edit] - Extended Mix (130 8b)",
    ),
    ("uppercase", "MISSY ELLIOTT", "GET UR FREAK ON (DIRTY INTRO)"),
];

const ALBUMS: &[&str] = &[
    "  Some Album  ",
    "www.example.com",
    "Album \u{2013} Deluxe (Remastered) ()",
    "Rock `n` Roll   Classics ((Vol. 2))",
];

const GENRES: &[&str] = &[
    "Hip-Hop 90's",
    "Deep    House",
    "R'n'B",
    "Drum N Bass",
    "Soul / Funk / Disco",
    "Progressive House",
    "Electronica",
];

const FILENAMES: &[&str] = &[
    "Kings Of Tomorrow - Finally (Sandy Rivera's Classic Mix)",
    "Various Artists - Big Sean - Dance (A$$)",
    "Beyonce\u{301} - Break My Soul (Trayze Acapella In-Out)",
];

/// Benchmark artist and title tag formatting.
fn bench_tags(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("format_tags");
    for (name, artist, title) in TAG_CASES {
        group.bench_function(*name, |bencher| {
            bencher.iter(|| formatting::format_tags_for_artist_and_title(black_box(artist), black_box(title)));
        });
    }
    group.throughput(Throughput::Elements(TAG_CORPUS.len() as u64));
    group.bench_function("corpus", |bencher| {
        bencher.iter(|| {
            for (artist, title) in TAG_CORPUS {
                black_box(formatting::format_tags_for_artist_and_title(
                    black_box(artist),
                    black_box(title),
                ));
            }
        });
    });
    group.finish();
}

/// Benchmark filename, album and genre formatting.
fn bench_other_fields(criterion: &mut Criterion) {
    let formatted: Vec<(String, String)> = TAG_CORPUS
        .iter()
        .map(|(artist, title)| formatting::format_tags_for_artist_and_title(artist, title))
        .collect();

    let mut group = criterion.benchmark_group("format_fields");
    group.throughput(Throughput::Elements(formatted.len() as u64));
    group.bench_function("filename", |bencher| {
        bencher.iter(|| {
            for (artist, title) in &formatted {
                black_box(formatting::format_filename(black_box(artist), black_box(title)));
            }
        });
    });
    group.throughput(Throughput::Elements(ALBUMS.len() as u64));
    group.bench_function("album", |bencher| {
        bencher.iter(|| {
            for album in ALBUMS {
                black_box(formatting::format_album(black_box(album)));
            }
        });
    });
    group.throughput(Throughput::Elements(GENRES.len() as u64));
    group.bench_function("genre", |bencher| {
        bencher.iter(|| {
            for genre_name in GENRES {
                black_box(genre::format_genre(black_box(genre_name)));
            }
        });
    });
    group.finish();
}

/// Benchmark string helpers used when parsing tags and filenames.
fn bench_utils(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("utils");
    group.throughput(Throughput::Elements(FILENAMES.len() as u64));
    group.bench_function("get_tags_from_filename", |bencher| {
        bencher.iter(|| {
            for filename in FILENAMES {
                black_box(utils::get_tags_from_filename(black_box(filename)));
            }
        });
    });
    group.bench_function("normalize_str", |bencher| {
        bencher.iter(|| {
            for filename in FILENAMES {
                black_box(utils::normalize_str(black_box(filename)));
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench_tags, bench_other_fields, bench_utils);
criterion_main!(benches);
