use std::io;
use std::io::Write;
use std::path::Path;

use colored::Colorize;
use difference::{Changeset, Difference};
use id3::{Error, ErrorKind, Tag, TagLike};

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

/// Try to read artist and title from tags.
/// Fallback to parsing them from filename if tags are empty.
pub fn parse_artist_and_title(track: &&mut Track, tag: &mut Tag) -> (String, String, String) {
    let mut artist = String::new();
    let mut title = String::new();
    let mut current_tags = " - ".to_string();

    match (tag.artist(), tag.title()) {
        (Some(a), Some(t)) => {
            artist = a.to_string();
            title = t.to_string();
            current_tags = format!("{} - {}", artist, title);
        }
        (None, None) => {
            eprintln!("{}", format!("Missing tags: {}", track.path.display()).yellow());
            if let Some((a, t)) = get_tags_from_filename(&track.name) {
                artist = a;
                title = t;
            }
        }
        (None, Some(t)) => {
            eprintln!("{}", format!("Missing artist tag: {}", track.path.display()).yellow());
            if let Some((a, _)) = get_tags_from_filename(&track.name) {
                artist = a;
            }
            title = t.to_string();
            current_tags = format!(" - {}", title);
        }
        (Some(a), None) => {
            eprintln!("{}", format!("Missing title tag: {}", track.path.display()).yellow());
            artist = a.to_string();
            if let Some((_, t)) = get_tags_from_filename(&track.name) {
                title = t;
            }
            current_tags = format!("{} - ", artist);
        }
    }
    (artist, title, current_tags)
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

pub fn print_divider(text: &str) {
    println!("{}", "-".repeat(text.chars().count()));
}

/// Convert filename to artist and title tags.
/// Expects filename to be in format 'artist - title'.
fn get_tags_from_filename(filename: &str) -> Option<(String, String)> {
    if !filename.contains(" - ") {
        eprintln!(
            "{}",
            format!("Can't parse tag data from malformed filename: {filename}").red()
        );
        return None;
    }

    let parts: Vec<&str> = filename.splitn(2, " - ").collect();
    if parts.len() == 2 {
        let artist = parts[0].to_string();
        let title = parts[1].to_string();
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
