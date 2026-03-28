use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use colored::Colorize;
use id3::{Error, ErrorKind, FrameError, FrameErrorKind, Tag, TagLike};
use itertools::Itertools;

use crate::output::{print_diff, print_stacked_diff};
use crate::track::Track;
use crate::utils::normalize_str;

/// Frames whose first field is a null-terminated Latin1 string that third-party
/// encoders sometimes write without the null terminator.
const FIXABLE_FRAMES: &[&str] = &["UFID", "PRIV"];
const FIXABLE_FRAMES_V2: &[&str] = &["UFI"];

/// Track tag data with current and formatted field values.
#[derive(Debug, Default, Clone)]
pub struct TrackTags {
    pub current_artist: String,
    pub current_title: String,
    pub current_album: String,
    pub current_genre: String,
    pub current_name: String,
    pub formatted_name: String,
    pub formatted_artist: String,
    pub formatted_title: String,
    pub formatted_album: String,
    pub formatted_genre: String,
    pub update_needed: bool,
}

impl TrackTags {
    #[must_use]
    pub fn new(name: String, artist: String, title: String, album: String, genre: String) -> Self {
        Self {
            current_name: name,
            current_artist: artist,
            current_title: title,
            current_album: album,
            current_genre: genre,
            ..Default::default()
        }
    }

    /// Try to read tags such as artist and title from tags.
    ///
    /// Fallback to parsing them from filename if tags are empty.
    #[must_use]
    pub fn parse_tag_data(track: &Track, tag: &Tag) -> Self {
        let mut artist = String::new();
        let mut title = String::new();

        // Tags might be formatted correctly but a missing field needs to be written.
        // Store formatted name before parsing missing fields from filename.
        let current_name: String;

        match (tag.artist(), tag.title()) {
            (Some(artist_tag), Some(title_tag)) => {
                artist = normalize_str(artist_tag);
                title = normalize_str(title_tag);
                current_name = format!("{artist} - {title}");
            }
            (None, None) => {
                print_yellow!("\nMissing tags: {}", track.path.display());
                current_name = format!("{artist} - {title}");
                if let Some((parsed_artist, parsed_title)) = crate::utils::get_tags_from_filename(&track.name) {
                    artist = parsed_artist;
                    title = parsed_title;
                }
            }
            (None, Some(title_tag)) => {
                print_yellow!("\nMissing artist tag: {}", track.path.display());
                title = normalize_str(title_tag);
                current_name = format!("{artist} - {title}");
                if let Some((parsed_artist, _)) = crate::utils::get_tags_from_filename(&track.name) {
                    artist = parsed_artist;
                }
            }
            (Some(artist_tag), None) => {
                print_yellow!("\nMissing title tag: {}", track.path.display());
                artist = normalize_str(artist_tag);
                current_name = format!("{artist} - {title}");
                if let Some((_, parsed_title)) = crate::utils::get_tags_from_filename(&track.name) {
                    title = parsed_title;
                }
            }
        }
        let album = normalize_str(tag.album().unwrap_or_default());
        let genre = normalize_str(tag.genre_parsed().unwrap_or_default().as_ref());
        Self::new(current_name, artist, title, album, genre)
    }

    /// Returns true if any of the formatted tag fields differ from their current value,
    /// or artist and/or title tag is missing.
    #[must_use]
    pub fn changed(&self) -> bool {
        self.current_name != self.formatted_name
            || self.current_artist != self.formatted_artist
            || self.current_title != self.formatted_title
            || self.current_album != self.formatted_album
            || self.current_genre != self.formatted_genre
    }

    /// Print coloured diff for changes in tags.
    ///
    /// Prints nothing if there are no changes.
    pub fn show_diff(&self) {
        if self.current_name != self.formatted_name {
            print_stacked_diff(&self.current_name, &self.formatted_name);
        }
        if self.current_album != self.formatted_album {
            print!("{}: ", "Album".bold());
            print_diff(&self.current_album, &self.formatted_album);
        }
        if self.current_genre != self.formatted_genre {
            print!("{}: ", "Genre".bold());
            print_diff(&self.current_genre, &self.formatted_genre);
        }
    }
}

/// Print all tag data.
pub fn print_tag_data(file_tags: &Tag) {
    println!("\n{}", format!("Tags ({}):", file_tags.version()).cyan().bold());
    file_tags
        .frames()
        .map(|frame| format!("{}: {}", frame.id(), frame.content()))
        .sorted_unstable()
        .for_each(|tag_string| println!("  {tag_string}"));
}

/// Try to read tag data from file.
///
/// Returns empty tags when there is no tag data.
/// If the tag reading fails,
/// returns the partial tag data that was read successfully before the error occurred,
/// or `None` if no tag data could be read.
///
/// Malformed UFID frames (e.g. Beatport tracks with a missing null terminator)
/// are automatically repaired by patching the raw ID3 tag bytes in-place:
/// the missing null delimiter is inserted so the UFID frame becomes valid,
/// preserving all other tag data (including frames after the UFID).
#[must_use]
pub fn read_tags(track: &Track, verbose: bool) -> Option<Tag> {
    match Tag::read_from_path(&track.path) {
        Ok(tag) => Some(tag),
        Err(Error {
            kind: ErrorKind::NoTag, ..
        }) => Some(Tag::new()),
        Err(error) if is_malformed_frame_error(&error) => repair_malformed_frame(track, error, verbose),
        Err(error) => {
            eprintln!("\n{}", format!("Failed to read tags for: {track}\n{error}").red());
            if verbose && let Some(ref partial_tags) = error.partial_tag {
                print_tag_data(partial_tags);
            }
            error.partial_tag
        }
    }
}

/// Check if an id3 parsing error is caused by a missing null delimiter
/// in a frame we know how to fix (UFID, PRIV, etc.).
fn is_malformed_frame_error(error: &Error) -> bool {
    matches!(
        &error.kind,
        ErrorKind::FrameParsing(FrameError {
            frame_id,
            kind: FrameErrorKind::DelimiterNotFound { .. },
            ..
        }) if FIXABLE_FRAMES.iter().any(|id| id == frame_id)
    )
}

/// Attempt to repair a file with a malformed frame by patching raw bytes.
///
/// Some third-party encoders (Beatport, Google Play, etc.) write frames like
/// UFID and PRIV without the required null terminator after `owner_identifier`.
///
/// The fix overwrites the first content byte with `0x00`,
/// which creates an empty `owner_identifier` (null-terminated)
/// and keeps the remaining bytes as the data payload.
/// This is a single-byte in-place change that preserves the frame size and all other tag data.
fn repair_malformed_frame(track: &Track, error: Error, verbose: bool) -> Option<Tag> {
    let frame_id = match &error.kind {
        ErrorKind::FrameParsing(fe) => fe.frame_id.clone(),
        _ => String::from("unknown"),
    };

    eprintln!(
        "\n{}",
        format!(
            "Malformed {frame_id} frame in: {track}\n  {}\n  Attempting to fix...",
            error.description
        )
        .yellow()
    );

    match fix_malformed_frames_raw(&track.path) {
        Ok(fixed) => {
            // Re-read the now-fixed file.
            match Tag::read_from_path(&track.path) {
                Ok(tag) => {
                    let summary = fixed
                        .iter()
                        .map(|(id, n)| {
                            let label = if *n == 1 { "frame" } else { "frames" };
                            format!("{n} {id} {label}")
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    eprintln!("{}", format!("  Fixed {summary} in: {track}").green());
                    if verbose {
                        print_tag_data(&tag);
                    }
                    Some(tag)
                }
                Err(reread_err) => {
                    eprintln!(
                        "{}",
                        format!("  Re-read after fix still failed for: {track}\n  {reread_err}").red()
                    );
                    reread_err.partial_tag
                }
            }
        }
        Err(err) => {
            eprintln!(
                "{}",
                format!("  Failed to fix {frame_id} frame in: {track}\n  {err}").red()
            );
            error.partial_tag
        }
    }
}

/// Patch malformed frames directly in the raw file bytes.
///
/// Supports MP3 (`ID3v2` at byte 0), AIFF (`FORM` → `ID3 ` chunk), and WAV (`RIFF` → `ID3 ` chunk).
/// Walks the `ID3v2` tag frame-by-frame looking for frames (UFID, PRIV, etc.)
/// whose first null-terminated string field has no null terminator.
/// For each one found, the first content byte is overwritten with `0x00`,
/// creating the missing delimiter.
/// Returns a list of `(frame_id, count)` pairs.
fn fix_malformed_frames_raw(path: &Path) -> anyhow::Result<Vec<(String, usize)>> {
    let mut data = std::fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;

    anyhow::ensure!(data.len() >= 12, "File too small to contain any tag data");

    // Find the byte offset where the ID3v2 header starts.
    let id3_offset =
        find_id3_header_offset(&data).with_context(|| format!("No ID3v2 header found in: {}", path.display()))?;

    let id3 = &data[id3_offset..];
    anyhow::ensure!(id3.len() >= 10, "Not enough data for an ID3v2 header");

    let version = id3[3]; // 2, 3, or 4
    let flags = id3[5];

    let tag_size = decode_synchsafe(&id3[6..10]) as usize;
    let tag_end_abs = id3_offset + 10 + tag_size;
    anyhow::ensure!(
        data.len() >= tag_end_abs,
        "File truncated: tag declares {} bytes but file is {} bytes",
        tag_end_abs,
        data.len()
    );

    // Frame header geometry differs between ID3 versions.
    let (frame_id_len, frame_header_len): (usize, usize) = match version {
        2 => (3, 6),
        3 | 4 => (4, 10),
        v => anyhow::bail!("Unsupported ID3v2.{v} version"),
    };

    // Skip extended header if the flag is set.
    let mut offset: usize = id3_offset + 10;
    if flags & 0x40 != 0 {
        anyhow::ensure!(
            offset + 4 <= tag_end_abs,
            "Extended header flag set but not enough data"
        );
        let ext_size = if version == 4 {
            // v2.4: synchsafe, size includes itself.
            decode_synchsafe(&data[offset..offset + 4]) as usize
        } else {
            // v2.3: big-endian u32, does NOT include the 4 size bytes.
            u32::from_be_bytes(
                data[offset..offset + 4]
                    .try_into()
                    .context("Extended header size bytes")?,
            ) as usize
                + 4
        };
        offset += ext_size;
    }

    let fixable: Vec<&[u8]> = if version == 2 {
        FIXABLE_FRAMES_V2.iter().map(|s| s.as_bytes()).collect()
    } else {
        FIXABLE_FRAMES.iter().map(|s| s.as_bytes()).collect()
    };
    let mut fixed: HashMap<String, usize> = HashMap::new();

    // Walk frames.
    while offset + frame_header_len <= tag_end_abs {
        let frame_id = &data[offset..offset + frame_id_len];

        // All-zero bytes mean we've reached padding.
        if frame_id.iter().all(|&b| b == 0) {
            break;
        }

        let frame_size: usize = if version == 2 {
            // ID3v2.2: 3-byte big-endian size.
            (u32::from(data[offset + 3]) << 16 | u32::from(data[offset + 4]) << 8 | u32::from(data[offset + 5]))
                as usize
        } else if version == 4 {
            // ID3v2.4: synchsafe integer.
            decode_synchsafe(&data[offset + 4..offset + 8]) as usize
        } else {
            // ID3v2.3: regular big-endian u32.
            u32::from_be_bytes(data[offset + 4..offset + 8].try_into().context("Frame size bytes")?) as usize
        };

        let content_start = offset + frame_header_len;
        let content_end = content_start + frame_size;

        if content_end > tag_end_abs {
            // Corrupted frame — stop scanning but don't fail; the id3 crate
            // will deal with whatever comes after our fix.
            break;
        }

        if frame_size > 0 && fixable.contains(&frame_id) {
            let content = &data[content_start..content_end];

            // Only fix frames that have no null byte at all (the actual bug).
            if !content.contains(&0x00) {
                let id_str = String::from_utf8_lossy(frame_id).to_string();
                // Replace the first byte with 0x00.  This turns the spurious
                // encoding byte into a null terminator, giving an empty
                // owner_identifier and keeping the rest as the data payload.
                data[content_start] = 0x00;
                *fixed.entry(id_str).or_insert(0) += 1;
            }
        }

        offset = content_end;
    }

    anyhow::ensure!(!fixed.is_empty(), "No malformed frames found to fix");

    std::fs::write(path, &data).with_context(|| format!("Failed to write patched file: {}", path.display()))?;

    let mut result: Vec<(String, usize)> = fixed.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

/// Locate the byte offset of the `ID3` header within a file's raw bytes.
///
/// - **MP3**: the `ID3` header is at byte 0.
/// - **AIFF** (`FORM`): the `ID3` header is inside an `ID3 ` chunk.
/// - **WAV** (`RIFF`): the `ID3` header is inside an `ID3 ` chunk.
///
/// Returns `None` if no `ID3v2` header can be found.
fn find_id3_header_offset(data: &[u8]) -> Option<usize> {
    // Direct ID3v2 header at the start (MP3 and similar).
    if data.len() >= 10 && &data[0..3] == b"ID3" {
        return Some(0);
    }

    // AIFF (FORM, big-endian) or WAV (RIFF, little-endian) container.
    let (root_tag, big_endian) = if data.len() >= 12 && &data[0..4] == b"FORM" {
        (b"FORM", true)
    } else if data.len() >= 12 && &data[0..4] == b"RIFF" {
        (b"RIFF", false)
    } else {
        return None;
    };
    let _ = root_tag; // validated above

    // Root chunk size (bytes 4..8) — we mostly care about scanning to EOF.
    // Skip past root header (tag 4 + size 4 + format 4 = 12 bytes).
    let mut pos: usize = 12;

    while pos + 8 <= data.len() {
        let chunk_tag = &data[pos..pos + 4];
        let chunk_size = if big_endian {
            u32::from_be_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]) as usize
        } else {
            u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]) as usize
        };

        let chunk_data_start = pos + 8;

        if chunk_tag == b"ID3 " {
            // The chunk data should start with an ID3v2 header.
            if chunk_data_start + 10 <= data.len() && &data[chunk_data_start..chunk_data_start + 3] == b"ID3" {
                return Some(chunk_data_start);
            }
        }

        // Advance to the next chunk (chunks are word-aligned in AIFF/WAV).
        let padded_size = chunk_size + (chunk_size % 2);
        pos = chunk_data_start + padded_size;
    }

    None
}

/// Decode a synchsafe integer (each byte uses only 7 bits, MSB is always 0).
fn decode_synchsafe(data: &[u8]) -> u32 {
    debug_assert!(data.len() == 4);
    (u32::from(data[0]) << 21) | (u32::from(data[1]) << 14) | (u32::from(data[2]) << 7) | u32::from(data[3])
}

#[cfg(test)]
mod test_tag_internals {
    use super::*;

    use id3::{Encoding, Error, ErrorKind, FrameError, FrameErrorKind};

    #[test]
    fn all_zeros_returns_zero() {
        let input = [0x00, 0x00, 0x00, 0x00];
        assert_eq!(decode_synchsafe(&input), 0);
    }

    #[test]
    fn least_significant_bit_set() {
        let input = [0x00, 0x00, 0x00, 0x01];
        assert_eq!(decode_synchsafe(&input), 1);
    }

    #[test]
    fn second_byte_one_gives_128() {
        let input = [0x00, 0x00, 0x01, 0x00];
        assert_eq!(decode_synchsafe(&input), 128);
    }

    #[test]
    fn third_byte_one_gives_16384() {
        let input = [0x00, 0x01, 0x00, 0x00];
        assert_eq!(decode_synchsafe(&input), 16384);
    }

    #[test]
    fn fourth_byte_one_gives_2097152() {
        let input = [0x01, 0x00, 0x00, 0x00];
        assert_eq!(decode_synchsafe(&input), 2_097_152);
    }

    #[test]
    fn max_synchsafe_value() {
        let input = [0x7f, 0x7f, 0x7f, 0x7f];
        assert_eq!(decode_synchsafe(&input), 268_435_455);
    }

    #[test]
    fn mp3_file_starting_with_id3_returns_zero() {
        let mut data = Vec::new();
        data.extend_from_slice(b"ID3");
        // version, flags, and synchsafe size (7 bytes to reach 10 total)
        data.extend_from_slice(&[0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        let result = find_id3_header_offset(&data);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn data_too_short_returns_none() {
        let data = b"ID3abcde"; // only 8 bytes, need at least 10
        let result = find_id3_header_offset(data);
        assert_eq!(result, None);
    }

    #[test]
    fn data_with_no_known_header_returns_none() {
        let data = b"NOT_A_VALID_HEADER_AT_ALL";
        let result = find_id3_header_offset(data);
        assert_eq!(result, None);
    }

    #[test]
    fn aiff_container_with_id3_chunk() {
        let mut data = Vec::new();
        data.extend_from_slice(b"FORM");
        data.extend_from_slice(&100u32.to_be_bytes());
        data.extend_from_slice(b"AIFF");
        // Non-ID3 chunk first (COMM)
        data.extend_from_slice(b"COMM");
        data.extend_from_slice(&8u32.to_be_bytes());
        data.extend_from_slice(&[0u8; 8]);
        // ID3 chunk
        data.extend_from_slice(b"ID3 ");
        let id3_payload_offset = data.len() + 4; // after the size field
        data.extend_from_slice(&20u32.to_be_bytes());
        // ID3 header marker inside the chunk
        data.extend_from_slice(b"ID3");
        // version 2.4, no flags, synchsafe size of 10, plus padding
        data.extend_from_slice(&[0x04, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00]);

        let result = find_id3_header_offset(&data);
        assert_eq!(
            result,
            Some(id3_payload_offset),
            "Expected ID3 header at offset {id3_payload_offset}"
        );
    }

    #[test]
    fn ufid_delimiter_not_found_returns_true() {
        let frame_error = FrameError {
            frame_id: String::from("UFID"),
            field: String::from("owner_identifier"),
            kind: FrameErrorKind::DelimiterNotFound {
                encoding: Encoding::Latin1,
                remaining_bytes: 10,
                hex_preview: String::from("aa bb cc"),
                ascii_preview: String::from("..."),
            },
        };
        let error = Error::frame_parsing(frame_error);
        assert!(
            is_malformed_frame_error(&error),
            "Expected UFID with DelimiterNotFound to be recognized as malformed"
        );
    }

    #[test]
    fn priv_delimiter_not_found_returns_true() {
        let frame_error = FrameError {
            frame_id: String::from("PRIV"),
            field: String::from("owner_identifier"),
            kind: FrameErrorKind::DelimiterNotFound {
                encoding: Encoding::Latin1,
                remaining_bytes: 5,
                hex_preview: String::from("de ad"),
                ascii_preview: String::from(".."),
            },
        };
        let error = Error::frame_parsing(frame_error);
        assert!(
            is_malformed_frame_error(&error),
            "Expected PRIV with DelimiterNotFound to be recognized as malformed"
        );
    }

    #[test]
    fn unknown_frame_id_returns_false() {
        let frame_error = FrameError {
            frame_id: String::from("APIC"),
            field: String::from("mime_type"),
            kind: FrameErrorKind::DelimiterNotFound {
                encoding: Encoding::Latin1,
                remaining_bytes: 20,
                hex_preview: String::from("ff"),
                ascii_preview: String::from("."),
            },
        };
        let error = Error::frame_parsing(frame_error);
        assert!(
            !is_malformed_frame_error(&error),
            "Expected non-fixable frame ID to not be recognized as malformed"
        );
    }

    #[test]
    fn frame_other_error_kind_returns_false() {
        let frame_error = FrameError {
            frame_id: String::from("UFID"),
            field: String::from("owner_identifier"),
            kind: FrameErrorKind::Other(String::from("some other error")),
        };
        let error = Error::frame_parsing(frame_error);
        assert!(
            !is_malformed_frame_error(&error),
            "Expected FrameErrorKind::Other to not be recognized as malformed"
        );
    }

    #[test]
    fn non_frame_parsing_error_returns_false() {
        let error = Error::new(ErrorKind::Parsing, "general parsing failure");
        assert!(
            !is_malformed_frame_error(&error),
            "Expected non-FrameParsing ErrorKind to return false"
        );
    }

    #[test]
    fn no_tag_error_returns_false() {
        let error = Error::new(ErrorKind::NoTag, "no tag found");
        assert!(
            !is_malformed_frame_error(&error),
            "Expected NoTag error to return false"
        );
    }
}

#[cfg(test)]
mod test_track_tags {
    use super::*;

    /// Helper to create a `TrackTags` with all formatted fields matching current fields.
    fn create_matching_tags() -> TrackTags {
        let mut tags = TrackTags::new(
            String::from("Artist - Title"),
            String::from("Artist"),
            String::from("Title"),
            String::from("Album"),
            String::from("Genre"),
        );
        tags.formatted_name = String::from("Artist - Title");
        tags.formatted_artist = String::from("Artist");
        tags.formatted_title = String::from("Title");
        tags.formatted_album = String::from("Album");
        tags.formatted_genre = String::from("Genre");
        tags
    }

    #[test]
    fn all_fields_match_returns_false() {
        let tags = create_matching_tags();
        assert!(!tags.changed(), "Expected changed() to be false when all fields match");
    }

    #[test]
    fn different_formatted_name_returns_true() {
        let mut tags = create_matching_tags();
        tags.formatted_name = String::from("Different Artist - Different Title");
        assert!(
            tags.changed(),
            "Expected changed() to be true when formatted_name differs"
        );
    }

    #[test]
    fn different_formatted_artist_returns_true() {
        let mut tags = create_matching_tags();
        tags.formatted_artist = String::from("Different Artist");
        assert!(
            tags.changed(),
            "Expected changed() to be true when formatted_artist differs"
        );
    }

    #[test]
    fn different_formatted_title_returns_true() {
        let mut tags = create_matching_tags();
        tags.formatted_title = String::from("Different Title");
        assert!(
            tags.changed(),
            "Expected changed() to be true when formatted_title differs"
        );
    }

    #[test]
    fn different_formatted_album_returns_true() {
        let mut tags = create_matching_tags();
        tags.formatted_album = String::from("Different Album");
        assert!(
            tags.changed(),
            "Expected changed() to be true when formatted_album differs"
        );
    }

    #[test]
    fn different_formatted_genre_returns_true() {
        let mut tags = create_matching_tags();
        tags.formatted_genre = String::from("Different Genre");
        assert!(
            tags.changed(),
            "Expected changed() to be true when formatted_genre differs"
        );
    }

    #[test]
    fn current_fields_are_set_correctly() {
        let tags = TrackTags::new(
            String::from("Artist - Title"),
            String::from("Artist"),
            String::from("Title"),
            String::from("Album"),
            String::from("Genre"),
        );
        assert_eq!(tags.current_name, "Artist - Title");
        assert_eq!(tags.current_artist, "Artist");
        assert_eq!(tags.current_title, "Title");
        assert_eq!(tags.current_album, "Album");
        assert_eq!(tags.current_genre, "Genre");
    }

    #[test]
    fn formatted_fields_default_to_empty_strings() {
        let tags = TrackTags::new(
            String::from("Name"),
            String::from("Artist"),
            String::from("Title"),
            String::from("Album"),
            String::from("Genre"),
        );
        assert_eq!(tags.formatted_name, "");
        assert_eq!(tags.formatted_artist, "");
        assert_eq!(tags.formatted_title, "");
        assert_eq!(tags.formatted_album, "");
        assert_eq!(tags.formatted_genre, "");
    }

    #[test]
    fn update_needed_defaults_to_false() {
        let tags = TrackTags::new(
            String::from("Name"),
            String::from("Artist"),
            String::from("Title"),
            String::from("Album"),
            String::from("Genre"),
        );
        assert!(!tags.update_needed, "Expected update_needed to default to false");
    }

    /// Create a `TrackTags` where the formatted name differs from the current name.
    fn create_tags_with_name_difference() -> TrackTags {
        let mut tags = TrackTags::new(
            String::from("old artist - old title"),
            String::from("old artist"),
            String::from("old title"),
            String::from("Old Album"),
            String::from("Old Genre"),
        );
        tags.formatted_name = String::from("New Artist - New Title");
        tags.formatted_artist = String::from("New Artist");
        tags.formatted_title = String::from("New Title");
        tags.formatted_album = String::from("New Album");
        tags.formatted_genre = String::from("New Genre");
        tags
    }

    /// Create a `TrackTags` where all formatted fields match current fields.
    fn create_tags_with_no_difference() -> TrackTags {
        let mut tags = TrackTags::new(
            String::from("Artist - Title"),
            String::from("Artist"),
            String::from("Title"),
            String::from("Album"),
            String::from("Genre"),
        );
        tags.formatted_name = String::from("Artist - Title");
        tags.formatted_artist = String::from("Artist");
        tags.formatted_title = String::from("Title");
        tags.formatted_album = String::from("Album");
        tags.formatted_genre = String::from("Genre");
        tags
    }

    #[test]
    fn show_diff_with_changes_does_not_panic() {
        let tags = create_tags_with_name_difference();
        tags.show_diff();
    }

    #[test]
    fn show_diff_with_no_changes_does_not_panic() {
        let tags = create_tags_with_no_difference();
        tags.show_diff();
    }
}

#[cfg(test)]
mod test_parse_tag_data {
    use super::*;
    use std::path::Path;

    /// Return the path to the basic tags MP3 test file.
    fn basic_tags_mp3_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/basic_tags/Basic Tags - Song - 16-44.mp3")
    }

    /// Return the path to the no tags MP3 test file.
    fn no_tags_mp3_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/no_tags/No Tags - Song - 16-44.mp3")
    }

    #[test]
    fn parses_artist_and_title_from_basic_tags() {
        let path = basic_tags_mp3_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let track = Track::try_from_path(&path).expect("Failed to create Track from basic tags MP3");
        let tag = Tag::read_from_path(&path).expect("Failed to read tags from basic tags MP3");
        let tags = TrackTags::parse_tag_data(&track, &tag);
        assert_eq!(
            tags.current_artist, "Basic Tags",
            "Expected current_artist to be 'Basic Tags', got '{}'",
            tags.current_artist
        );
        assert_eq!(
            tags.current_title, "Song - 16-44",
            "Expected current_title to be 'Song - 16-44', got '{}'",
            tags.current_title
        );
        assert!(
            tags.current_name.contains("Basic Tags"),
            "Expected current_name to contain 'Basic Tags', got '{}'",
            tags.current_name
        );
        assert!(
            tags.current_name.contains("Song"),
            "Expected current_name to contain 'Song', got '{}'",
            tags.current_name
        );
    }

    #[test]
    fn parses_artist_and_title_from_filename_when_no_tags() {
        let path = no_tags_mp3_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let track = Track::try_from_path(&path).expect("Failed to create Track from no tags MP3");
        let tag = Tag::new();
        let tags = TrackTags::parse_tag_data(&track, &tag);
        let has_artist = !tags.current_artist.is_empty();
        let has_title = !tags.current_title.is_empty();
        assert!(
            has_artist || has_title,
            "Expected at least artist or title to be parsed from filename, got artist='{}' title='{}'",
            tags.current_artist,
            tags.current_title
        );
    }
}

#[cfg(test)]
mod test_read_tags {
    use super::*;
    use std::path::Path;

    /// Return the path to the basic tags MP3 test file.
    fn basic_tags_mp3_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/basic_tags/Basic Tags - Song - 16-44.mp3")
    }

    #[test]
    fn reads_tags_from_basic_mp3() {
        let path = basic_tags_mp3_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let track = Track::try_from_path(&path).expect("Failed to create Track from basic tags MP3");
        let result = read_tags(&track, false);
        assert!(result.is_some(), "Expected read_tags to return Some for basic tags MP3");
        let tag = result.expect("read_tags returned None for basic tags MP3");
        assert!(
            tag.artist().is_some(),
            "Expected tag to have an artist for basic tags MP3"
        );
    }

    /// Return the path to the no tags MP3 test file.
    fn no_tags_mp3_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/no_tags/No Tags - Song - 16-44.mp3")
    }

    #[test]
    fn reads_empty_tags_from_no_tags_mp3() {
        let path = no_tags_mp3_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let track = Track::try_from_path(&path).expect("Failed to create Track from no tags MP3");
        let result = read_tags(&track, false);
        assert!(
            result.is_some(),
            "Expected read_tags to return Some (empty Tag) for no tags MP3"
        );
    }

    /// Return the path to the extended tags MP3 test file.
    fn extended_tags_mp3_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/extended_tags/Extended Tags - Song - 16-44.mp3")
    }

    #[test]
    fn reads_tags_from_extended_mp3_in_verbose_mode() {
        let path = extended_tags_mp3_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let track = Track::try_from_path(&path).expect("Failed to create Track from extended tags MP3");
        let result = read_tags(&track, true);
        assert!(
            result.is_some(),
            "Expected read_tags to return Some for extended tags MP3"
        );
        let tag = result.expect("read_tags returned None for extended tags MP3");
        assert!(
            tag.artist().is_some(),
            "Expected tag to have an artist for extended tags MP3"
        );
        assert!(
            tag.title().is_some(),
            "Expected tag to have a title for extended tags MP3"
        );
    }

    #[test]
    fn prints_tag_data_without_panic() {
        let path = basic_tags_mp3_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let tag = Tag::read_from_path(&path).expect("Failed to read tags from basic tags MP3");
        print_tag_data(&tag);
    }
}
