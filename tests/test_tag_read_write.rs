use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use track_rename::tags::write_tags;
use track_rename::track::Track;

struct FixtureCase {
    fixture_directory: &'static str,
    base_name: &'static str,
    extension: &'static str,
}

fn fixture_path(case: &FixtureCase) -> PathBuf {
    Path::new("tests/files")
        .join(case.fixture_directory)
        .join(format!("{}.{}", case.base_name, case.extension))
}

fn make_temp_fixture_copy(case: &FixtureCase) -> PathBuf {
    let source = fixture_path(case);
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System clock should be after UNIX_EPOCH")
        .as_nanos();
    let tmp = std::env::temp_dir().join(format!(
        "track_rename_test_{}_{}_{}.{}",
        case.fixture_directory, case.extension, unique_suffix, case.extension
    ));
    std::fs::copy(&source, &tmp).unwrap_or_else(|_| panic!("Failed to copy fixture: {}", source.display()));
    tmp
}

fn assert_tags_match_fixture(
    case: &FixtureCase,
    artist: Option<&str>,
    title: Option<&str>,
    album: Option<&str>,
    genre: Option<&str>,
) {
    let source = fixture_path(case);
    if !source.exists() {
        eprintln!("Test file not found, skipping: {}", source.display());
        return;
    }

    let tmp = make_temp_fixture_copy(case);
    let track = Track::try_from_path(&tmp).expect("Failed to create Track from fixture");
    let tag = track.read_tags(false).expect("Failed to read fixture tags");

    assert_eq!(tag.artist(), artist, "Artist mismatch for {}", source.display());
    assert_eq!(tag.title(), title, "Title mismatch for {}", source.display());
    assert_eq!(tag.album(), album, "Album mismatch for {}", source.display());
    assert_eq!(tag.genre(), genre, "Genre mismatch for {}", source.display());

    drop(tag);
    drop(track);
    std::fs::remove_file(&tmp).expect("Failed to remove temp fixture file");
}

fn assert_roundtrip_for_fixture(case: &FixtureCase) {
    let source = fixture_path(case);
    if !source.exists() {
        eprintln!("Test file not found, skipping: {}", source.display());
        return;
    }

    let tmp = make_temp_fixture_copy(case);
    let mut track = Track::try_from_path(&tmp).expect("Failed to create Track from fixture");
    let mut file_tags = track.read_tags(false).expect("Failed to read fixture tags");

    let artist = format!("{} {} Artist", case.fixture_directory, case.extension);
    let title = format!("{} {} Title", case.fixture_directory, case.extension);
    let album = format!("{} {} Album", case.fixture_directory, case.extension);
    let genre = format!("{} {} Genre", case.fixture_directory, case.extension);

    track.tags.formatted_artist.clone_from(&artist);
    track.tags.formatted_title.clone_from(&title);
    track.tags.formatted_album.clone_from(&album);
    track.tags.formatted_genre.clone_from(&genre);

    write_tags(&track, &mut file_tags).expect("Failed to write updated tags");

    let reread_track = Track::try_from_path(&tmp).expect("Failed to recreate Track from updated fixture");
    let reread_tags = reread_track.read_tags(false).expect("Failed to re-read updated tags");

    assert_eq!(
        reread_tags.artist(),
        Some(artist.as_str()),
        "Artist mismatch for {}",
        source.display()
    );
    assert_eq!(
        reread_tags.title(),
        Some(title.as_str()),
        "Title mismatch for {}",
        source.display()
    );
    assert_eq!(
        reread_tags.album(),
        Some(album.as_str()),
        "Album mismatch for {}",
        source.display()
    );
    assert_eq!(
        reread_tags.genre(),
        Some(genre.as_str()),
        "Genre mismatch for {}",
        source.display()
    );

    drop(reread_tags);
    drop(reread_track);
    drop(file_tags);
    drop(track);
    std::fs::remove_file(&tmp).expect("Failed to remove temp fixture file");
}

#[test]
fn test_missing_title_is_read_from_each_format() {
    for case in [
        FixtureCase {
            fixture_directory: "missing_title",
            base_name: "Missing Title - Song - 16-44",
            extension: "mp3",
        },
        FixtureCase {
            fixture_directory: "missing_title",
            base_name: "Missing Title - Song - 16-44",
            extension: "aif",
        },
        FixtureCase {
            fixture_directory: "missing_title",
            base_name: "Missing Title - Song - 16-44",
            extension: "flac",
        },
    ] {
        assert_tags_match_fixture(
            &case,
            Some("Missing Title"),
            None,
            Some("Missing Title"),
            Some("Electronic"),
        );
    }
}

#[test]
fn test_basic_tags_are_read_from_each_format() {
    for case in [
        FixtureCase {
            fixture_directory: "basic_tags",
            base_name: "Basic Tags - Song - 16-44",
            extension: "mp3",
        },
        FixtureCase {
            fixture_directory: "basic_tags",
            base_name: "Basic Tags - Song - 16-44",
            extension: "aif",
        },
        FixtureCase {
            fixture_directory: "basic_tags",
            base_name: "Basic Tags - Song - 16-44",
            extension: "flac",
        },
    ] {
        assert_tags_match_fixture(&case, Some("Basic Tags"), Some("Song - 16-44"), None, None);
    }
}

#[test]
fn test_extended_tags_are_read_from_each_format() {
    for case in [
        FixtureCase {
            fixture_directory: "extended_tags",
            base_name: "Extended Tags - Song - 16-44",
            extension: "mp3",
        },
        FixtureCase {
            fixture_directory: "extended_tags",
            base_name: "Extended Tags - Song - 16-44",
            extension: "aif",
        },
        FixtureCase {
            fixture_directory: "extended_tags",
            base_name: "Extended Tags - Song - 16-44",
            extension: "flac",
        },
    ] {
        assert_tags_match_fixture(
            &case,
            Some("Extended Tags"),
            Some("Song - 16-44"),
            Some("Extended Tags"),
            Some("Electronic"),
        );
    }
}

#[test]
fn test_no_tags_are_read_from_each_format() {
    for case in [
        FixtureCase {
            fixture_directory: "no_tags",
            base_name: "No Tags - Song - 16-44",
            extension: "mp3",
        },
        FixtureCase {
            fixture_directory: "no_tags",
            base_name: "No Tags - Song - 16-44",
            extension: "aif",
        },
        FixtureCase {
            fixture_directory: "no_tags",
            base_name: "No Tags - Song - 16-44",
            extension: "flac",
        },
    ] {
        assert_tags_match_fixture(&case, None, None, None, None);
    }
}

#[test]
fn test_mp3_roundtrip_writes_all_tag_fields_for_all_fixture_sets() {
    for case in [
        FixtureCase {
            fixture_directory: "no_tags",
            base_name: "No Tags - Song - 16-44",
            extension: "mp3",
        },
        FixtureCase {
            fixture_directory: "basic_tags",
            base_name: "Basic Tags - Song - 16-44",
            extension: "mp3",
        },
        FixtureCase {
            fixture_directory: "extended_tags",
            base_name: "Extended Tags - Song - 16-44",
            extension: "mp3",
        },
    ] {
        assert_roundtrip_for_fixture(&case);
    }
}

#[test]
fn test_aif_roundtrip_writes_all_tag_fields_for_all_fixture_sets() {
    for case in [
        FixtureCase {
            fixture_directory: "no_tags",
            base_name: "No Tags - Song - 16-44",
            extension: "aif",
        },
        FixtureCase {
            fixture_directory: "basic_tags",
            base_name: "Basic Tags - Song - 16-44",
            extension: "aif",
        },
        FixtureCase {
            fixture_directory: "extended_tags",
            base_name: "Extended Tags - Song - 16-44",
            extension: "aif",
        },
    ] {
        assert_roundtrip_for_fixture(&case);
    }
}

#[test]
fn test_flac_roundtrip_writes_all_tag_fields_for_all_fixture_sets() {
    for case in [
        FixtureCase {
            fixture_directory: "no_tags",
            base_name: "No Tags - Song - 16-44",
            extension: "flac",
        },
        FixtureCase {
            fixture_directory: "basic_tags",
            base_name: "Basic Tags - Song - 16-44",
            extension: "flac",
        },
        FixtureCase {
            fixture_directory: "extended_tags",
            base_name: "Extended Tags - Song - 16-44",
            extension: "flac",
        },
    ] {
        assert_roundtrip_for_fixture(&case);
    }
}
