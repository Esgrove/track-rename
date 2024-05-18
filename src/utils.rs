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

/// Format bool value as a coloured string
pub fn colorize_bool(value: bool) -> ColoredString {
    if value {
        "true".green()
    } else {
        "false".yellow()
    }
}

/// Create a colored diff for given strings
pub fn color_diff(old: &str, new: &str) -> (String, String) {
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

/// Check ffmpeg is found in PATH.
pub fn ffmpeg_available() -> bool {
    Command::new("ffmpeg").arg("-version").output().is_ok()
}

/// Convert the given path to be relative to the current working directory.
/// Returns the original path if the relative path cannot be created.
pub fn get_relative_path_from_current_working_directory(path: &Path) -> PathBuf {
    env::current_dir()
        .map(|current_dir| path.strip_prefix(&current_dir).unwrap_or(path).to_path_buf())
        .unwrap_or(path.to_path_buf())
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

    let parts: Vec<&str> = filename.splitn(2, " - ").collect();
    if parts.len() == 2 {
        let artist = normalize_str(parts[0]);
        let title = normalize_str(parts[1]);
        Some((artist, title))
    } else {
        None
    }
}

pub fn normalize_str(input: &str) -> String {
    input.nfc().collect::<String>()
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

/// Print a single line diff of the changes.
pub fn print_diff(old: &str, new: &str) {
    let (old_diff, new_diff) = color_diff(old, new);
    println!("{} -> {}", old_diff, new_diff);
}

/// Print a stacked diff of the changes.
pub fn print_stacked_diff(old: &str, new: &str) {
    let (old_diff, new_diff) = color_diff(old, new);
    println!("{}", old_diff);
    println!("{}", new_diff);
}

/// Print a divider line that matches the length of the reference text.
pub fn print_divider(text: &str) {
    println!("{}", "-".repeat(text.chars().count()));
}

/// Try to read tags from file.
/// Will return empty tags when there are no tags.
pub fn read_tags(track: &Track) -> Option<Tag> {
    match Tag::read_from_path(&track.path) {
        Ok(tag) => Some(tag),
        Err(Error {
            kind: ErrorKind::NoTag, ..
        }) => {
            println!("\n{}", format!("No tags: {}", track).yellow());
            Some(Tag::new())
        }
        Err(error) => {
            eprintln!("\n{}", format!("Failed to read tags for: {}\n{}", track, error).red());
            None
        }
    }
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

/// Write a txt log file for failed tracks to current working directory.
pub fn write_genre_log(genres: &[(String, usize)]) -> anyhow::Result<()> {
    let filepath = Path::new("genres.txt");
    let mut file = File::create(filepath).context("Failed to create output file")?;
    for (genre, _) in genres.iter() {
        writeln!(file, "{}", genre)?;
    }

    println!("Logged genres to: {}", dunce::canonicalize(filepath)?.display());
    Ok(())
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
            Some(("".into(), "Songtitle (Remix)".into()))
        );
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
