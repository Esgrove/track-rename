use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io};

use anyhow::Context;
use colored::{ColoredString, Colorize};
use difference::{Changeset, Difference};
use id3::{Error, ErrorKind, Tag};
use unicode_normalization::UnicodeNormalization;

use crate::track::Track;

/// Ask user to confirm action.
/// Note: everything except `n` is a yes.
pub fn confirm() -> bool {
    print!("Proceed (y/n)? ");
    io::stdout().flush().expect("Failed to flush stdout");
    let mut ans = String::new();
    io::stdin().read_line(&mut ans).expect("Failed to read line");
    ans.trim().to_lowercase() != "n"
}

/// Convert the given path to be relative to the current working directory.
/// Returns the original path if the relative path cannot be created.
pub fn get_relative_path_from_current_working_directory(path: &Path) -> PathBuf {
    env::current_dir()
        .map(|current_dir| path.strip_prefix(&current_dir).unwrap_or(path).to_path_buf())
        .unwrap_or(path.to_path_buf())
}

/// Convert path to string with invalid unicode handling.
pub fn path_to_string(path: &Path) -> String {
    if let Some(string) = path.to_str() {
        string.to_string()
    } else {
        let string = path.to_string_lossy().to_string().replace('\u{FFFD}', "");
        eprintln!("{}", "Path contains invalid unicode".red());
        eprintln!("{:?}", path);
        eprintln!("{}", string);
        string
    }
}

/// Get relative path and convert to string with invalid unicode handling.
pub fn path_to_string_relative(path: &Path) -> String {
    path_to_string(&get_relative_path_from_current_working_directory(path))
}

/// Try to read tags from file.
/// Will return empty tags when there are no tags.
pub fn read_tags(track: &Track) -> Option<Tag> {
    match Tag::read_from_path(&track.path) {
        Ok(tag) => Some(tag),
        Err(Error {
            kind: ErrorKind::NoTag, ..
        }) => {
            println!("{}", format!("No tags: {}", track).yellow());
            Some(Tag::new())
        }
        Err(error) => {
            eprintln!("{}", format!("Failed to read tags for: {}\n{}", track, error).red());
            None
        }
    }
}

/// Print a stacked diff of the changes.
pub fn show_diff(old: &str, new: &str) {
    let changeset = Changeset::new(old, new, "");
    let mut old_diff = String::new();
    let mut new_diff = String::new();

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

    println!("{}", old_diff);
    println!("{}", new_diff);
}

/// Print a divider line that matches the length of the reference text.
pub fn print_divider(text: &str) {
    println!("{}", "-".repeat(text.chars().count()));
}

/// Rename track from given path to new path.
pub fn rename_track(path: &Path, new_path: &Path, test_mode: bool) -> anyhow::Result<()> {
    if let Err(error) = fs::rename(path, new_path) {
        let message = format!("Failed to rename file: {}", error);
        if test_mode {
            panic!("{}", message);
        } else {
            eprintln!("{}", message.red());
        }
    }
    Ok(())
}

/// Write a txt log file for failed tracks to current working directory.
pub fn write_log_for_failed_files(paths: &[String]) -> anyhow::Result<()> {
    let filepath = Path::new("track-rename-failed.txt");
    let mut file = File::create(filepath).context("Failed to create output file")?;
    for path in paths.iter() {
        writeln!(file, "{}", path)?;
    }
    println!("Logged failed files to: {}", dunce::canonicalize(filepath)?.display());
    Ok(())
}

/// Format bool value as a coloured string
pub fn colorize_bool(value: bool) -> ColoredString {
    if value {
        "true".green()
    } else {
        "false".yellow()
    }
}

/// Check ffmpeg is found in PATH.
pub fn ffmpeg_available() -> bool {
    Command::new("ffmpeg").arg("-version").output().is_ok()
}

/// Convert filename to artist and title tags.
/// Expects filename to be in format 'artist - title'.
pub fn get_tags_from_filename(filename: &str) -> Option<(String, String)> {
    if !filename.contains(" - ") {
        eprintln!(
            "{}",
            format!("Can't parse tag data from malformed filename: {filename}").red()
        );
        return None;
    }

    let parts: Vec<&str> = filename.splitn(2, " - ").collect();
    if parts.len() == 2 {
        let artist = parts[0].nfc().collect::<String>();
        let title = parts[1].nfc().collect::<String>();
        Some((artist, title))
    } else {
        None
    }
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
        let filename = "ArtistTitle";
        assert_eq!(get_tags_from_filename(filename), None);
    }

    #[test]
    fn test_get_tags_from_filename_with_additional_delimiters() {
        let filename = "Artist - Title - Remix";
        assert_eq!(
            get_tags_from_filename(filename),
            Some(("Artist".to_string(), "Title - Remix".to_string()))
        );
    }

    #[test]
    fn test_get_tags_from_filename_empty_filename() {
        let filename = "";
        assert_eq!(get_tags_from_filename(filename), None);
    }
}
