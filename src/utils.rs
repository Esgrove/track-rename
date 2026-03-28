use std::cmp::Ordering;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use clap::Command as ClapCommand;
use clap_complete::Shell;
use colored::{ColoredString, Colorize};
use difference::{Changeset, Difference};
use id3::{Error, ErrorKind, FrameError, FrameErrorKind, Tag};
use itertools::Itertools;
use rayon::prelude::*;
use unicode_normalization::UnicodeNormalization;
use walkdir::WalkDir;

use crate::track::Track;

/// Frames whose first field is a null-terminated Latin1 string that third-party
/// encoders sometimes write without the null terminator.
const FIXABLE_FRAMES: &[&str] = &["UFID", "PRIV"];
const FIXABLE_FRAMES_V2: &[&str] = &["UFI"];

/// Recursively collect all supported audio tracks from given root path.
pub fn collect_tracks(root: &Path) -> Vec<Track> {
    WalkDir::new(root)
        .into_iter()
        .par_bridge()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_file())
        .filter_map(|entry| Track::try_from_path(entry.path()))
        .collect()
}

/// Format bool value as a coloured string.
#[must_use]
pub fn colorize_bool(value: bool) -> ColoredString {
    if value { "true".green() } else { "false".yellow() }
}

/// Create a coloured diff for the given strings.
pub fn color_diff(old: &str, new: &str, stacked: bool) -> (String, String) {
    let changeset = Changeset::new(old, new, "");
    let mut old_diff = String::new();
    let mut new_diff = String::new();

    if stacked {
        // Find the starting index of the first matching sequence for a nicer visual alignment.
        // For example:
        //   Constantine - Onde As Satisfaction (Club Tool).aif
        //        Darude - Onde As Satisfaction (Constantine Club Tool).aif
        // Instead of:
        //   Constantine - Onde As Satisfaction (Club Tool).aif
        //   Darude - Onde As Satisfaction (Constantine Club Tool).aif
        for diff in &changeset.diffs {
            if let Difference::Same(x) = diff {
                if x.chars().all(char::is_whitespace) || x.chars().count() < 2 {
                    continue;
                }

                let old_first_match_index = old.find(x);
                let new_first_match_index = new.find(x);

                // Add leading whitespace so that the first matching sequence lines up.
                if let (Some(old_index), Some(new_index)) = (old_first_match_index, new_first_match_index) {
                    match old_index.cmp(&new_index) {
                        Ordering::Greater => {
                            new_diff = " ".repeat(old_index.saturating_sub(new_index));
                        }
                        Ordering::Less => {
                            old_diff = " ".repeat(new_index.saturating_sub(old_index));
                        }
                        Ordering::Equal => {}
                    }
                    break;
                }
            }
        }
    }

    for diff in changeset.diffs {
        match diff {
            Difference::Same(ref x) => {
                old_diff.push_str(x);
                new_diff.push_str(x);
            }
            Difference::Add(ref x) => {
                if x.chars().all(char::is_whitespace) {
                    new_diff.push_str(&x.on_green().to_string());
                } else {
                    new_diff.push_str(&x.green().to_string());
                }
            }
            Difference::Rem(ref x) => {
                if x.chars().all(char::is_whitespace) {
                    old_diff.push_str(&x.on_red().to_string());
                } else {
                    old_diff.push_str(&x.red().to_string());
                }
            }
        }
    }

    (old_diff, new_diff)
}

/// Ask user to confirm action.
///
/// Note: everything except `n` or `N` is a yes.
#[must_use]
pub fn confirm() -> bool {
    print!("Proceed (y/n)? ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    let mut ans = String::new();
    std::io::stdin().read_line(&mut ans).expect("Failed to read line");
    ans.trim().to_lowercase() != "n"
}

/// Check if the given path contains the subpath.
///
/// Checks if `subpath` is a part of `path`,
/// starting from the first matching path component in `path`.
/// Returns `true` if `subpath` exists within `path` and `false` otherwise.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use track_rename::utils::contains_subpath;
/// let main_path = Path::new("/a/b/c/d");
/// let subpath = Path::new("b/c");
/// // `b/c` is a subpath of `/a/b/c/d`
/// assert!(contains_subpath(main_path, subpath));
///
/// let subpath = Path::new("c/d");
/// // `c/d` is a subpath of `/a/b/c/d`
/// assert!(contains_subpath(main_path, subpath));
///
/// let subpath = Path::new("x/y");
/// // `x/y` is not a subpath of `/a/b/c/d`
/// assert!(!contains_subpath(main_path, subpath));
///
/// let subpath = Path::new("b/c/x");
/// // `b/c/x` is not a subpath of `/a/b/c/d`
/// assert!(!contains_subpath(main_path, subpath));
///
/// let subpath = Path::new("/a/b/c/d/e");
/// // `/a/b/c/d/e` is longer than `/a/b/c/d`
/// assert!(!contains_subpath(main_path, subpath));
/// ```
#[must_use]
pub fn contains_subpath(path: &Path, subpath: &Path) -> bool {
    let main_components: Vec<_> = path.components().collect();
    let sub_components: Vec<_> = subpath.components().collect();

    // Sanity check
    if sub_components.len() > main_components.len() {
        return false;
    }

    // Find the start index of the first subpath component in the main path
    if let Some(first_sub_component) = sub_components.first() {
        for (index, main_component) in main_components.iter().enumerate() {
            if main_component == first_sub_component {
                // Check all the subcomponents match starting from this index
                if main_components[index..]
                    .iter()
                    .zip(sub_components.iter())
                    .all(|(main, sub)| main == sub)
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Check ffmpeg is found in PATH.
#[must_use]
pub fn ffmpeg_available() -> bool {
    Command::new("ffmpeg").arg("-version").output().is_ok()
}

/// Get file modified time as seconds since unix epoch.
pub fn get_file_modified_time(path: &Path) -> anyhow::Result<u64> {
    let metadata = std::fs::metadata(path)?;
    let modified_time = metadata.modified()?;
    let duration = modified_time
        .duration_since(UNIX_EPOCH)
        .context("Failed to get duration since unix epoch")?;
    Ok(duration.as_secs())
}

/// Convert the given path to be relative to the current working directory.
/// Returns the original path if the relative path cannot be created.
#[must_use]
pub fn get_relative_path_from_current_working_directory(path: &Path) -> PathBuf {
    std::env::current_dir().map_or_else(
        |_| path.to_path_buf(),
        |current_dir| path.strip_prefix(&current_dir).unwrap_or(path).to_path_buf(),
    )
}

/// Convert filename to artist and title tags.
/// Expects filename to be in format 'artist - title'.
#[must_use]
pub fn get_tags_from_filename(filename: &str) -> Option<(String, String)> {
    if !filename.contains(" - ") {
        eprintln!(
            "{}",
            format!("Can't parse full tag data from malformed filename: {filename}").yellow()
        );
        return if filename.is_empty() {
            None
        } else {
            Some((String::new(), filename.to_string()))
        };
    }
    let trimmed_filename = filename.trim_start_matches("Various Artists - ").trim().to_string();
    let parts: Vec<&str> = trimmed_filename.splitn(2, " - ").collect();
    if parts.len() == 2 {
        let artist = normalize_str(parts[0].trim());
        let title = normalize_str(parts[1].trim());
        Some((artist, title))
    } else {
        None
    }
}

/// Normalize unicode.
#[must_use]
pub fn normalize_str(input: &str) -> String {
    input.nfc().collect::<String>()
}

/// Convert a path to string with invalid Unicode handling.
#[allow(clippy::unnecessary_debug_formatting)]
pub fn path_to_string(path: &Path) -> String {
    path.to_str().map_or_else(
        || {
            let string = path.to_string_lossy().to_string().replace('\u{FFFD}', "");
            eprintln!("{}", "Path contains invalid unicode:".red());
            eprintln!("{path:?}");
            eprintln!("{string}");
            string
        },
        ToString::to_string,
    )
}

/// Get the relative path and convert to string with invalid unicode handling.
#[must_use]
pub fn path_to_string_relative(path: &Path) -> String {
    path_to_string(&get_relative_path_from_current_working_directory(path))
}

/// Print a single line diff of the changes.
pub fn print_diff(old: &str, new: &str) {
    let (old_diff, new_diff) = color_diff(old, new, false);
    println!("{old_diff} -> {new_diff}");
}

/// Print a stacked diff of the changes.
pub fn print_stacked_diff(old: &str, new: &str) {
    let (old_diff, new_diff) = color_diff(old, new, true);
    println!("{old_diff}");
    println!("{new_diff}");
}

/// Print a divider line that matches the length of the reference text.
pub fn print_divider(text: &str) {
    println!("{}", "-".repeat(text.chars().count()));
}

/// Print error message with red color to stderr.
#[inline]
pub fn print_error(message: &str) {
    eprintln!("{}", format!("Error: {message}").red());
}

/// Print formatted error message with red color to stderr.
#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        $crate::utils::print_error(&format!($($arg)*))
    };
}

/// Print warning message with yellow color to stderr.
#[inline]
pub fn print_yellow(message: &str) {
    eprintln!("{}", message.yellow());
}

/// Print formatted warning message with yellow color to stderr.
#[macro_export]
macro_rules! print_yellow {
    ($($arg:tt)*) => {
        $crate::utils::print_yellow(&format!($($arg)*))
    };
}

/// Print message with green color.
#[inline]
pub fn print_green(message: &str) {
    println!("{}", message.green());
}

/// Print formatted message with green color.
#[macro_export]
macro_rules! print_green {
    ($($arg:tt)*) => {
        $crate::utils::print_green(&format!($($arg)*))
    };
}

/// Print message with magenta color.
#[inline]
pub fn print_magenta(message: &str) {
    println!("{}", message.magenta());
}

/// Print formatted message with magenta color.
#[macro_export]
macro_rules! print_magenta {
    ($($arg:tt)*) => {
        $crate::utils::print_magenta(&format!($($arg)*))
    };
}

/// Print message with bold magenta color.
#[inline]
pub fn print_magenta_bold(message: &str) {
    println!("{}", message.magenta().bold());
}

/// Print formatted message with bold magenta color.
#[macro_export]
macro_rules! print_magenta_bold {
    ($($arg:tt)*) => {
        $crate::utils::print_magenta_bold(&format!($($arg)*))
    };
}

/// Print message with cyan color.
#[inline]
pub fn print_cyan(message: &str) {
    println!("{}", message.cyan());
}

/// Print formatted message with cyan color.
#[macro_export]
macro_rules! print_cyan {
    ($($arg:tt)*) => {
        $crate::utils::print_cyan(&format!($($arg)*))
    };
}

/// Print message with bold style.
#[inline]
pub fn print_bold(message: &str) {
    println!("{}", message.bold());
}

/// Print formatted message with bold style.
#[macro_export]
macro_rules! print_bold {
    ($($arg:tt)*) => {
        $crate::utils::print_bold(&format!($($arg)*))
    };
}

/// Print message with dimmed style.
#[inline]
pub fn print_dimmed(message: &str) {
    println!("{}", message.dimmed());
}

/// Print formatted message with dimmed style.
#[macro_export]
macro_rules! print_dimmed {
    ($($arg:tt)*) => {
        $crate::utils::print_dimmed(&format!($($arg)*))
    };
}

/// Print all tag data.
pub fn print_tag_data(file_tags: &Tag) {
    println!("\n{}", format!("Tags ({}):", file_tags.version()).cyan().bold());
    file_tags
        .frames()
        .map(|frame| format!("{}: {}", frame.id(), frame.content()))
        .sorted_unstable()
        .for_each(|string| println!("  {string}"));
}

/// Try to read tag data from file.
///
/// Returns empty tags when there is no tag data.
/// If the tag reading fails,
/// returns the partial tag data that was read succesfully before the error occured,
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

/// Rename track from given path to new path.
pub fn rename_track(path: &Path, new_path: &Path, test_mode: bool) -> anyhow::Result<()> {
    if let Err(error) = std::fs::rename(path, new_path) {
        let message = format!("Failed to rename file: {error}");
        if test_mode {
            panic!("{}", message);
        } else {
            print_error(&message);
        }
    }
    Ok(())
}

/// Resolve optional input path or otherwise use current working dir.
pub fn resolve_input_path(path: Option<&Path>) -> anyhow::Result<PathBuf> {
    let filepath = match path {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().context("Failed to get current working directory")?,
    };
    if !filepath.exists() {
        anyhow::bail!(
            "Input path does not exist or is not accessible: '{}'",
            dunce::simplified(&filepath).display()
        );
    }
    let absolute_input_path = dunce::canonicalize(filepath)?;
    Ok(absolute_input_path)
}

/// Generate a shell completion script for the given shell.
pub fn generate_shell_completion(
    shell: Shell,
    mut command: ClapCommand,
    install: bool,
    verbose: bool,
    command_name: &str,
) -> anyhow::Result<()> {
    if install {
        let out_dir = get_shell_completion_dir(shell, command_name)?;
        let path = clap_complete::generate_to(shell, &mut command, command_name, out_dir)?;
        if verbose {
            println!("Completion file generated to: {}", path.display());
        }
    } else {
        clap_complete::generate(shell, &mut command, command_name, &mut std::io::stdout());
    }
    Ok(())
}

/// Write a txt log file for failed tracks to current working directory.
pub fn write_log_for_failed_files(paths: &[String]) -> anyhow::Result<()> {
    let filepath = Path::new("track-rename-failed.txt");
    let mut file = std::fs::File::create(filepath).context("Failed to create output file")?;
    for path in paths {
        writeln!(file, "{path}")?;
    }
    println!("Logged failed files to: {}", dunce::canonicalize(filepath)?.display());
    Ok(())
}

/// Get filename string for given Path.
pub fn get_filename_from_path(path: &Path) -> anyhow::Result<String> {
    Ok(path
        .file_name()
        .context("Failed to get zip file name")?
        .to_string_lossy()
        .replace('\u{FFFD}', ""))
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
    let mut fixed: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

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

/// Determine the appropriate directory for storing shell completions.
///
/// First checks if the user-specific directory exists,
/// then checks for the global directory.
/// If neither exist, creates and uses the user-specific dir.
fn get_shell_completion_dir(shell: Shell, name: &str) -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().expect("Failed to get home directory");

    // Special handling for oh-my-zsh.
    // Create custom "plugin", which will then have to be loaded in .zshrc
    if shell == Shell::Zsh {
        let omz_plugins = home.join(".oh-my-zsh/custom/plugins");
        if omz_plugins.exists() {
            let plugin_dir = omz_plugins.join(name);
            std::fs::create_dir_all(&plugin_dir)?;
            return Ok(plugin_dir);
        }
    }

    let user_dir = match shell {
        Shell::PowerShell => {
            if cfg!(windows) {
                home.join(r"Documents\PowerShell\completions")
            } else {
                home.join(".config/powershell/completions")
            }
        }
        Shell::Bash => home.join(".bash_completion.d"),
        Shell::Elvish => home.join(".elvish"),
        Shell::Fish => home.join(".config/fish/completions"),
        Shell::Zsh => home.join(".zsh/completions"),
        _ => anyhow::bail!("Unsupported shell"),
    };

    if user_dir.exists() {
        return Ok(user_dir);
    }

    let global_dir = match shell {
        Shell::PowerShell => {
            if cfg!(windows) {
                home.join(r"Documents\PowerShell\completions")
            } else {
                home.join(".config/powershell/completions")
            }
        }
        Shell::Bash => PathBuf::from("/etc/bash_completion.d"),
        Shell::Fish => PathBuf::from("/usr/share/fish/completions"),
        Shell::Zsh => PathBuf::from("/usr/share/zsh/site-functions"),
        _ => anyhow::bail!("Unsupported shell"),
    };

    if global_dir.exists() {
        return Ok(global_dir);
    }

    std::fs::create_dir_all(&user_dir)?;
    Ok(user_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tags_from_filename() {
        let filename = "Artist - Title";
        assert_eq!(
            get_tags_from_filename(filename),
            Some(("Artist".to_string(), "Title".to_string()))
        );
    }

    #[test]
    fn test_get_tags_from_filename_no_delimiter() {
        let filename = "Songtitle (Remix)";
        assert_eq!(
            get_tags_from_filename(filename),
            Some((String::new(), "Songtitle (Remix)".into()))
        );
    }

    #[test]
    fn test_get_tags_from_filename_with_additional_delimiters() {
        let filename = "Various Artists - Dave & Maurissa  - Look At The Stars (Dave’s Starshine Club Mix)";
        assert_eq!(
            get_tags_from_filename(filename),
            Some((
                "Dave & Maurissa".to_string(),
                "Look At The Stars (Dave’s Starshine Club Mix)".to_string()
            ))
        );
    }

    #[test]
    fn test_get_tags_from_filename_empty_filename() {
        let filename = "";
        assert_eq!(get_tags_from_filename(filename), None);
    }

    #[test]
    fn test_get_tags_from_filename_various_artists() {
        let filename = "";
        assert_eq!(get_tags_from_filename(filename), None);
    }
}
