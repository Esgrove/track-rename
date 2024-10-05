use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;
use std::{env, fs, io};

use anyhow::Context;
use colored::{ColoredString, Colorize};
use difference::{Changeset, Difference};
use id3::{Error, ErrorKind, Tag};
use itertools::Itertools;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;
use walkdir::WalkDir;

use crate::track::Track;

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
pub fn colorize_bool(value: bool) -> ColoredString {
    if value {
        "true".green()
    } else {
        "false".yellow()
    }
}

/// Create a coloured diff for the given strings.
pub fn color_diff(old: &str, new: &str, stacked: bool) -> (String, String) {
    let changeset = Changeset::new(old, new, "");
    let mut old_diff = String::new();
    let mut new_diff = String::new();

    if stacked {
        // Find the starting index of the first matching sequence.
        for diff in &changeset.diffs {
            if let Difference::Same(ref x) = diff {
                if x.chars().all(char::is_whitespace) {
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
                    new_diff.push_str(&x.to_string().on_green().to_string());
                } else {
                    new_diff.push_str(&x.to_string().green().to_string());
                }
            }
            Difference::Rem(ref x) => {
                if x.chars().all(char::is_whitespace) {
                    old_diff.push_str(&x.to_string().on_red().to_string());
                } else {
                    old_diff.push_str(&x.to_string().red().to_string());
                }
            }
        }
    }

    (old_diff, new_diff)
}

/// Ask user to confirm action.
/// Note: everything except `n` is a yes.
pub fn confirm() -> bool {
    print!("Proceed (y/n)? ");
    io::stdout().flush().expect("Failed to flush stdout");
    let mut ans = String::new();
    io::stdin().read_line(&mut ans).expect("Failed to read line");
    ans.trim().to_lowercase() != "n"
}

/// Check if the given path contains the sub path.
pub fn contains_subpath(main_path: &Path, subpath: &Path) -> bool {
    let main_components: Vec<_> = main_path.components().collect();
    let sub_components: Vec<_> = subpath.components().collect();

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

/// Calculate SHA256 hash for the given file.
pub fn compute_file_hash(path: &Path) -> anyhow::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Check ffmpeg is found in PATH.
pub fn ffmpeg_available() -> bool {
    Command::new("ffmpeg").arg("-version").output().is_ok()
}

/// Get file modified time as seconds since unix epoch.
pub fn get_file_modified_time(path: &Path) -> anyhow::Result<u64> {
    let metadata = fs::metadata(path)?;
    let modified_time = metadata.modified()?;
    let duration = modified_time
        .duration_since(UNIX_EPOCH)
        .context("Failed to get duration since unix epoch")?;
    Ok(duration.as_secs())
}

/// Convert the given path to be relative to the current working directory.
/// Returns the original path if the relative path cannot be created.
pub fn get_relative_path_from_current_working_directory(path: &Path) -> PathBuf {
    env::current_dir().map_or_else(
        |_| path.to_path_buf(),
        |current_dir| path.strip_prefix(&current_dir).unwrap_or(path).to_path_buf(),
    )
}

/// Convert filename to artist and title tags.
/// Expects filename to be in format 'artist - title'.
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
pub fn normalize_str(input: &str) -> String {
    input.nfc().collect::<String>()
}

/// Convert a path to string with invalid Unicode handling.
pub fn path_to_string(path: &Path) -> String {
    path.to_str().map_or_else(
        || {
            let string = path.to_string_lossy().to_string().replace('\u{FFFD}', "");
            eprintln!("{}", "Path contains invalid unicode".red());
            eprintln!("{path:?}");
            eprintln!("{string}");
            string
        },
        ToString::to_string,
    )
}

/// Get the relative path and convert to string with invalid unicode handling.
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

/// Print all tag data.
pub fn print_tag_data(file_tags: &Tag) {
    println!("\n{}", format!("Tags ({}):", file_tags.version()).cyan().bold());
    file_tags
        .frames()
        .map(|frame| format!("{}: {}", frame.id(), frame.content()))
        .sorted_unstable()
        .for_each(|string| println!("  {string}"));
}

/// Try to read tags from file.
/// Will return empty tags when there are no tags.
pub fn read_tags(track: &Track, verbose: bool) -> Option<Tag> {
    match Tag::read_from_path(&track.path) {
        Ok(tag) => Some(tag),
        Err(Error {
            kind: ErrorKind::NoTag, ..
        }) => {
            println!("\n{}", format!("No tags: {track}").yellow());
            Some(Tag::new())
        }
        Err(error) => {
            eprintln!("\n{}", format!("Failed to read tags for: {track}\n{error}").red());
            if verbose {
                if let Some(ref partial_tags) = error.partial_tag {
                    print_tag_data(partial_tags);
                }
            }
            error.partial_tag
        }
    }
}

/// Rename track from given path to new path.
pub fn rename_track(path: &Path, new_path: &Path, test_mode: bool) -> anyhow::Result<()> {
    if let Err(error) = fs::rename(path, new_path) {
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
pub fn resolve_input_path(path: &Option<String>) -> anyhow::Result<PathBuf> {
    let input_path = path.clone().unwrap_or_default().trim().to_string();
    let filepath = if input_path.is_empty() {
        env::current_dir().context("Failed to get current working directory")?
    } else {
        PathBuf::from(input_path)
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

/// Write a txt log file for failed tracks to current working directory.
pub fn write_log_for_failed_files(paths: &[String]) -> anyhow::Result<()> {
    let filepath = Path::new("track-rename-failed.txt");
    let mut file = File::create(filepath).context("Failed to create output file")?;
    for path in paths {
        writeln!(file, "{path}")?;
    }
    println!("Logged failed files to: {}", dunce::canonicalize(filepath)?.display());
    Ok(())
}

/// Get filename string for given Path.
pub fn get_filename_from_path(path: &Path) -> anyhow::Result<String> {
    let file_name = path
        .file_name()
        .context("Failed to get zip file name")?
        .to_string_lossy()
        .replace('\u{FFFD}', "");
    Ok(file_name)
}

pub fn print_error(message: &str) {
    eprintln!("{}", message.red());
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
