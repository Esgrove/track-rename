//! Regression tests for the id3 GEOB "delimiter not found" parsing bug.
//!
//! The id3 crate decodes frames sequentially and aborts on the first parse error,
//! returning only the frames decoded so far as a partial tag. Serato GEOB frames
//! can trigger a "delimiter not found" error in `string_delimited()`, which means
//! any frames physically **after** them in the tag stream are silently lost.
//!
//! When `set_title` / `set_artist` etc. append new text frames, they end up after
//! the existing GEOB/APIC frames and get dropped on the next read.
//!
//! The workaround is a two-phase write:
//!   1. Write only text frames (strip GEOB/APIC first).
//!   2. Re-read, append the saved binary frames, write again.
//!
//! This ensures text frames appear first in the stream so they are always decoded
//! before any GEOB error.

use id3::{Tag, TagLike, Version};

/// Path to an MP3 with Serato GEOB frames, an artist tag, but no title or album tag.
/// Generated from the `extended_tags` test file by removing TIT2 and TALB.
const TEST_FILE: &str = "tests/files/missing_title/Extended Tags - Song - 16-44.mp3";

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn test_file_exists() -> bool {
    std::path::Path::new(TEST_FILE).exists()
}

fn make_temp_copy(suffix: &str) -> std::path::PathBuf {
    let tmp = std::env::temp_dir().join(format!("track_rename_test_{suffix}.mp3"));
    std::fs::copy(TEST_FILE, &tmp).expect("Failed to copy test file");
    tmp
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

/// Verify the test fixture: artist present, title missing.
#[test]
fn test_missing_title_tag_detected() {
    if !test_file_exists() {
        eprintln!("Test file not found, skipping: {TEST_FILE}");
        return;
    }
    let tmp = make_temp_copy("missing_title");
    let tag = Tag::read_from_path(&tmp).expect("Failed to read tags");

    assert_eq!(tag.artist(), Some("Extended Tags"), "Artist tag should be present");
    assert_eq!(tag.title(), None, "Title tag should be missing");
    assert_eq!(tag.album(), None, "Album tag should be missing");

    std::fs::remove_file(&tmp).expect("Failed to remove temp file");
}

/// Verify that a naive read-modify-write round-trip preserves new text frames.
///
/// NOTE: The id3 "delimiter not found" bug only manifests with very large
/// GEOB/APIC data (e.g. 24 KB Serato Offsets_ + 138 KB album art).  This
/// small test fixture does NOT trigger it. The two-phase write workaround
/// is still applied unconditionally as a safety net for files that do.
#[test]
fn test_write_roundtrip_with_geob() {
    if !test_file_exists() {
        eprintln!("Test file not found, skipping: {TEST_FILE}");
        return;
    }
    let tmp = make_temp_copy("roundtrip_geob");

    let mut tag = Tag::read_from_path(&tmp).expect("Failed to read tags");
    let has_geob = tag.frames().any(|f| f.id() == "GEOB");
    assert!(has_geob, "Test file should contain GEOB (Serato) frames");

    tag.set_title("Song (16-44)");
    tag.set_album("Test Album");
    tag.write_to_path(&tmp, Version::Id3v24)
        .expect("write_to_path should not return an error");

    let tag2 = Tag::read_from_path(&tmp).expect("Failed to re-read tags");
    assert_eq!(tag2.artist(), Some("Extended Tags"));
    assert_eq!(tag2.title(), Some("Song (16-44)"));
    assert_eq!(tag2.album(), Some("Test Album"));
    assert!(
        tag2.frames().any(|f| f.id() == "GEOB"),
        "GEOB frames should be preserved"
    );

    std::fs::remove_file(&tmp).expect("Failed to remove temp file");
}

/// A fresh tag with only text frames (no GEOB) round-trips correctly.
#[test]
fn test_fresh_text_only_tag_write_roundtrips() {
    if !test_file_exists() {
        eprintln!("Test file not found, skipping: {TEST_FILE}");
        return;
    }
    let tmp = make_temp_copy("fresh_text");

    let mut tag = Tag::new();
    tag.set_artist("Extended Tags");
    tag.set_title("Song (16-44)");
    tag.set_album("Test Album");
    tag.write_to_path(&tmp, Version::Id3v24).expect("Failed to write tags");

    let tag2 = Tag::read_from_path(&tmp).expect("Failed to re-read tags");
    assert_eq!(tag2.artist(), Some("Extended Tags"));
    assert_eq!(tag2.title(), Some("Song (16-44)"));
    assert_eq!(tag2.album(), Some("Test Album"));

    std::fs::remove_file(&tmp).expect("Failed to remove temp file");
}

/// The two-phase write workaround:
///   Phase 1 — write text frames without GEOB/APIC.
///   Phase 2 — re-read, append the binary frames, write again.
///
/// Text frames now appear first in the stream and survive even if a
/// GEOB frame triggers a parse error on the next read.
#[test]
fn test_two_phase_write_preserves_text_and_binary_frames() {
    if !test_file_exists() {
        eprintln!("Test file not found, skipping: {TEST_FILE}");
        return;
    }
    let tmp = make_temp_copy("two_phase");

    // Read original and separate binary frames.
    let old_tag = Tag::read_from_path(&tmp).expect("Failed to read tags");
    let binary_frames: Vec<_> = old_tag
        .frames()
        .filter(|f| f.id() == "GEOB" || f.id() == "APIC")
        .cloned()
        .collect();
    let original_geob_count = binary_frames.iter().filter(|f| f.id() == "GEOB").count();
    assert!(original_geob_count > 0, "Test file should have GEOB frames");

    // Phase 1: write text frames only (no GEOB/APIC).
    let mut text_tag = Tag::new();
    for frame in old_tag.frames() {
        if frame.id() != "GEOB" && frame.id() != "APIC" {
            text_tag.add_frame(frame.clone());
        }
    }
    text_tag.set_artist("Extended Tags");
    text_tag.set_title("Song (16-44)");
    text_tag.set_album("Test Album");
    text_tag
        .write_to_path(&tmp, Version::Id3v24)
        .expect("Phase 1 write failed");

    // Verify text frames survived phase 1.
    let check = Tag::read_from_path(&tmp).expect("Failed to re-read after phase 1");
    assert_eq!(check.artist(), Some("Extended Tags"));
    assert_eq!(check.title(), Some("Song (16-44)"));
    assert_eq!(check.album(), Some("Test Album"));

    // Phase 2: re-read, add binary frames back, write again.
    let mut full_tag = Tag::read_from_path(&tmp).expect("Failed to re-read for phase 2");
    for frame in &binary_frames {
        full_tag.add_frame(frame.clone());
    }
    full_tag
        .write_to_path(&tmp, Version::Id3v24)
        .expect("Phase 2 write failed");

    // Verify text frames AND binary frames are present.
    let final_tag = Tag::read_from_path(&tmp).expect("Failed to read final tags");
    assert_eq!(final_tag.artist(), Some("Extended Tags"));
    assert_eq!(final_tag.title(), Some("Song (16-44)"));
    assert_eq!(final_tag.album(), Some("Test Album"));

    let final_geob_count = final_tag.frames().filter(|f| f.id() == "GEOB").count();
    assert_eq!(
        original_geob_count, final_geob_count,
        "Serato GEOB frames should be preserved after two-phase write"
    );

    std::fs::remove_file(&tmp).expect("Failed to remove temp file");
}

/// Running the two-phase write a second time should be a no-op: all tags
/// are already present and in the right order, so nothing is lost.
#[test]
fn test_two_phase_write_is_idempotent() {
    if !test_file_exists() {
        eprintln!("Test file not found, skipping: {TEST_FILE}");
        return;
    }
    let tmp = make_temp_copy("idempotent");

    // --- first pass (simulates initial fix) ---
    let old_tag = Tag::read_from_path(&tmp).expect("Failed to read tags");
    let binary_frames: Vec<_> = old_tag
        .frames()
        .filter(|f| f.id() == "GEOB" || f.id() == "APIC")
        .cloned()
        .collect();

    let mut text_tag = Tag::new();
    for frame in old_tag.frames() {
        if frame.id() != "GEOB" && frame.id() != "APIC" {
            text_tag.add_frame(frame.clone());
        }
    }
    text_tag.set_artist("Extended Tags");
    text_tag.set_title("Song (16-44)");
    text_tag.set_album("Test Album");
    text_tag
        .write_to_path(&tmp, Version::Id3v24)
        .expect("First pass phase 1 write failed");

    let mut full_tag = Tag::read_from_path(&tmp).expect("Failed to re-read after first pass phase 1");
    for frame in &binary_frames {
        full_tag.add_frame(frame.clone());
    }
    full_tag
        .write_to_path(&tmp, Version::Id3v24)
        .expect("First pass phase 2 write failed");

    // --- second pass (simulates running the tool again) ---
    let tag_pass2 = Tag::read_from_path(&tmp).expect("Failed to read tags on second pass");

    assert_eq!(
        tag_pass2.artist(),
        Some("Extended Tags"),
        "Artist should be present on second pass"
    );
    assert_eq!(
        tag_pass2.title(),
        Some("Song (16-44)"),
        "Title should be present on second pass — must not be flagged as missing again"
    );
    assert_eq!(
        tag_pass2.album(),
        Some("Test Album"),
        "Album should be present on second pass"
    );

    let has_geob = tag_pass2.frames().any(|f| f.id() == "GEOB");
    assert!(has_geob, "Serato GEOB frames should still be present on second pass");

    std::fs::remove_file(&tmp).expect("Failed to remove temp file");
}

/// Stripping binary frames from the in-memory tag, writing, re-reading
/// and re-adding them also preserves everything.
#[test]
fn test_strip_and_restore_binary_frames() {
    if !test_file_exists() {
        eprintln!("Test file not found, skipping: {TEST_FILE}");
        return;
    }
    let tmp = make_temp_copy("strip_restore");

    let old_tag = Tag::read_from_path(&tmp).expect("Failed to read tags");
    let binary_frames: Vec<_> = old_tag
        .frames()
        .filter(|f| f.id() == "GEOB" || f.id() == "APIC")
        .cloned()
        .collect();

    // Strip GEOB/APIC from the tag in memory, set new fields, write.
    let mut tag = Tag::read_from_path(&tmp).expect("Failed to read tags");
    tag.remove("GEOB");
    tag.remove("APIC");
    tag.set_title("Song (16-44)");
    tag.set_album("Test Album");
    tag.write_to_path(&tmp, Version::Id3v24)
        .expect("Text-only write failed");

    let check = Tag::read_from_path(&tmp).expect("Failed to re-read");
    assert_eq!(check.artist(), Some("Extended Tags"));
    assert_eq!(check.title(), Some("Song (16-44)"));
    assert_eq!(check.album(), Some("Test Album"));

    // Re-add binary frames.
    let mut tag = Tag::read_from_path(&tmp).expect("Failed to re-read for binary add");
    for frame in &binary_frames {
        tag.add_frame(frame.clone());
    }
    tag.write_to_path(&tmp, Version::Id3v24)
        .expect("Binary frame write failed");

    let final_tag = Tag::read_from_path(&tmp).expect("Failed to read final tags");
    assert_eq!(final_tag.artist(), Some("Extended Tags"));
    assert_eq!(final_tag.title(), Some("Song (16-44)"));
    assert_eq!(final_tag.album(), Some("Test Album"));
    assert!(
        final_tag.frames().any(|f| f.id() == "GEOB"),
        "GEOB frames should be restored"
    );

    std::fs::remove_file(&tmp).expect("Failed to remove temp file");
}
