use crate::fileformat::FileFormat;

use anyhow::Context;
use unicode_normalization::UnicodeNormalization;

use colored::Colorize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fmt};

#[derive(Debug)]
pub struct Track {
    pub name: String,
    pub extension: String,
    pub format: FileFormat,
    pub root: PathBuf,
    pub path: PathBuf,
    pub tags_updated: bool,
    pub renamed: bool,
    pub printed: bool,
}

impl Track {
    #![allow(dead_code)]
    /// New Track from path
    pub fn new(path: PathBuf) -> anyhow::Result<Track> {
        let name = Self::get_nfc_filename_from_path(&path);
        let extension = path
            .extension()
            .context("Failed to get file extension")?
            .to_string_lossy()
            .to_string();

        let format = FileFormat::from_str(&extension)?;
        let root = path.parent().context("Failed to get file root")?.to_owned();
        // Rebuild full path with desired unicode handling
        let path = root.join(format!("{}.{}", name, extension));

        Ok(Track {
            name,
            extension,
            format,
            root,
            path,
            tags_updated: false,
            renamed: false,
            printed: false,
        })
    }

    /// New Track with already extracted extension and file format.
    /// Note that extension string is needed in addition to format
    /// since the extension might differ from the one used by `FileFormat`,
    /// in which case it would not point to the original filename.
    pub fn new_with_extension(path: PathBuf, extension: String, format: FileFormat) -> anyhow::Result<Track> {
        let name = Self::get_nfc_filename_from_path(&path);
        let root = path.parent().context("Failed to get file root")?.to_owned();
        // Rebuild full path with desired unicode handling
        let path = root.join(format!("{}.{}", name, extension));

        Ok(Track {
            name,
            extension,
            format,
            root,
            path,
            tags_updated: false,
            renamed: false,
            printed: false,
        })
    }

    pub fn try_from_path(path: &Path) -> Option<Track> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or_default().trim();
        if extension.is_empty() {
            return None;
        }
        match FileFormat::from_str(extension) {
            Ok(format) => match Track::new_with_extension(path.to_path_buf(), extension.to_string(), format) {
                Ok(track) => return Some(track),
                Err(error) => {
                    eprintln!(
                        "{}",
                        format!("Failed to create Track from: {}\n{error}", path.display()).red()
                    );
                }
            },
            Err(_) => {
                if extension == "wav" {
                    println!(
                        "{}",
                        format!("Wav should be converted to aif: {}", path.display()).yellow()
                    );
                }
            }
        }
        None
    }

    /// Print track if it has not been already.
    pub fn show(&mut self, number: usize, total_tracks: usize) {
        if !self.printed {
            println!("{number}/{total_tracks}:");
            self.printed = true
        }
    }

    /// Get the original file name
    pub fn filename(&self) -> String {
        format!("{}.{}", self.name, self.extension)
    }

    /// Get filename from Path with special characters retained instead of decomposed.
    fn get_nfc_filename_from_path(path: &Path) -> String {
        path
            .file_stem()
            .expect("Failed to get file stem")
            .to_str()
            .expect("Filename contains invalid Unicode")
            // Rust uses unicode NFD (Normalization Form Decomposed) by default,
            // which converts special chars like "å" to "a\u{30a}",
            // which then get printed as a regular "a".
            // Use NFC (Normalization Form Composed) from unicode_normalization crate instead
            // to retain correct format.
            // https://github.com/unicode-rs/unicode-normalization
            .nfc()
            .collect::<String>()
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Track {}

impl PartialOrd for Track {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Track {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl fmt::Display for Track {
    // Try to print full filepath relative to current working directory,
    // otherwise fallback to absolute path.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let current_dir = match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return write!(f, "{}/{}.{}", self.root.display(), self.name, self.format),
        };
        let relative_path = match self.root.strip_prefix(&current_dir) {
            Ok(path) => path,
            Err(_) => &self.root,
        };
        write!(f, "{}/{}.{}", relative_path.display(), self.name, self.format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_track_new_valid_path() {
        let path = PathBuf::from("/users/test/test_song.mp3");
        let track = Track::new(path).expect("Failed to create track");
        assert_eq!(track.name, "test_song");
        assert_eq!(track.extension, "mp3");
        assert_eq!(track.format, FileFormat::Mp3);
        assert_eq!(track.root, PathBuf::from("/users/test"));
        assert_eq!(track.filename(), "test_song.mp3");
    }
    #[test]
    fn test_track_with_special_characters() {
        let path = PathBuf::from("/Users/akseli/Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix).mp3");
        let track = Track::new(path).expect("Failed to create track");
        assert_eq!(track.name, "Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix)");
        assert_eq!(track.extension, "mp3");
        assert_eq!(track.format, FileFormat::Mp3);
        assert_eq!(track.root, PathBuf::from("/Users/akseli"));
        assert_eq!(
            track.filename(),
            "Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix).mp3"
        );
        assert_eq!(
            track.path.to_str().expect("Failed to convert track path to string"),
            "/Users/akseli/Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix).mp3"
        );
    }
    #[test]
    fn test_track_new_with_extension() {
        let path = PathBuf::from("/users/test/another/artist - test song.aiff");
        let track = Track::new_with_extension(path, "aiff".to_string(), FileFormat::Aif)
            .expect("Failed to create track with extension");
        assert_eq!(track.name, "artist - test song");
        assert_eq!(track.extension, "aiff");
        assert_eq!(track.format, FileFormat::Aif);
        assert_eq!(track.root, PathBuf::from("/users/test/another"));
        assert_eq!(track.filename(), "artist - test song.aiff");
    }
    #[test]
    fn test_track_equality() {
        let track1 = Track::new(PathBuf::from("/users/test/Test - song1.mp3")).expect("Failed to create track");
        let track2 = Track::new(PathBuf::from("/users/other/Test - song1.aif")).expect("Failed to create track");
        assert_eq!(track1.extension, "mp3");
        assert_eq!(track1.format, FileFormat::Mp3);
        assert_eq!(track2.extension, "aif");
        assert_eq!(track2.format, FileFormat::Aif);
        assert_eq!(track1, track2);
    }
    #[test]
    fn test_track_display() {
        let dir = env::current_dir().expect("Failed to get current dir");
        let track = Track::new(dir.join("artist - title.mp3")).expect("Failed to create track");
        let displayed = format!("{}", track);
        assert!(displayed.contains("artist - title.mp3"));

        let path_display = format!("{}", track.path.display());
        assert!(path_display.contains("artist - title.mp3"));
    }
    #[test]
    fn test_track_display_with_special_characters() {
        let dir = env::current_dir().expect("Failed to get current dir");
        let track = Track::new(dir.join("Ääkköset - Test.aif")).expect("Failed to create track");
        assert_eq!(track.extension, "aif");
        assert_eq!(track.format, FileFormat::Aif);

        let displayed = format!("{}", track);
        assert!(displayed.contains("Ääkköset - Test.aif"));

        let path_display = format!("{}", track.path.display());
        assert!(path_display.contains("Ääkköset - Test.aif"));
    }
}
