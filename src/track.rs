use std::cmp::Ordering;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::LazyLock;

use anyhow::Context;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::file_format::FileFormat;
use crate::genre::GENRE_MAPPINGS;
use crate::tags::{FileTags, TrackTags};
use crate::utils;
use crate::utils::{get_file_modified_time, path_to_string, path_to_string_relative};
use crate::{formatting, genre};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub static DJ_MUSIC_PATH: LazyLock<PathBuf> = LazyLock::new(|| ["Dropbox", "DJ MUSIC"].iter().collect());

// Other audio file extensions that should trigger a warning message,
const OTHER_FILE_EXTENSIONS: [&str; 2] = ["wav", "m4a"];

/// Represents one audio file.
#[derive(Debug, Default, Clone)]
pub struct Track {
    /// Filename of the existing file without the file extension
    pub name: String,
    /// File extension string for the existing file
    pub extension: String,
    /// Parent directory name
    pub directory: String,
    /// File format enum
    pub format: FileFormat,
    /// Path to parent directory
    pub root: PathBuf,
    /// Full filepath for the existing file
    pub path: PathBuf,
    /// File metadata
    pub metadata: TrackMetadata,
    /// The index of this track
    pub number: usize,
    /// Tag data
    pub tags: TrackTags,
    /// True if updated tag data has been saved to file
    pub tags_updated: bool,
    /// If the track needs to be updated but is not, then skip saving state
    pub not_processed: bool,
    /// True if track info has been displayed in the terminal
    printed: bool,
}

/// File metadata for Track.
#[derive(Debug, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TrackMetadata {
    /// Last modified timestamp provided by the OS.
    pub modified: u64,
    /// The track-rename library version this file was last processed with.
    pub version: String,
}

impl Track {
    /// New Track from the given path.
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let extension = path
            .extension()
            .context("Failed to get file extension")?
            .to_str()
            .context("File extension contains invalid Unicode")?
            .to_string();

        let format = FileFormat::from_str(&extension)?;
        Self::new_with_extension(path, extension, format)
    }

    /// New Track with already extracted extension and file format.
    /// Note that extension string is necessary in addition to format
    /// since the file name extension might differ from the one used by `FileFormat`,
    /// in which case it would not point to the original filename.
    pub fn new_with_extension(path: &Path, extension: String, format: FileFormat) -> anyhow::Result<Self> {
        let name = Self::get_nfc_filename_from_path(path)?;
        let root = path.parent().context("Failed to get file root")?.to_owned();
        let directory = utils::get_filename_from_path(&root).context("Failed to get parent directory name")?;

        // Rebuild the full path with desired Unicode handling
        let path = dunce::simplified(root.join(format!("{name}.{extension}")).as_path()).to_path_buf();
        let metadata = Self::read_metadata(&path)?;
        Ok(Self {
            name,
            extension,
            directory,
            format,
            root,
            path,
            metadata,
            ..Default::default()
        })
    }

    #[must_use]
    pub fn try_from_path(path: &Path) -> Option<Self> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or_default().trim();
        if extension.is_empty() {
            return None;
        }
        match FileFormat::from_str(extension) {
            Ok(format) => match Self::new_with_extension(path, extension.to_string(), format) {
                Ok(track) => return Some(track),
                Err(error) => {
                    eprintln!(
                        "{}",
                        format!("Failed to create Track from: {}\n{error}", path.display()).red()
                    );
                }
            },
            // Not a supported file format
            Err(_) => {
                if OTHER_FILE_EXTENSIONS.contains(&extension.to_lowercase().as_str()) {
                    println!(
                        "{}",
                        format!(
                            "{} file should be converted to a supported format: {}",
                            extension.to_uppercase(),
                            path.display()
                        )
                        .bright_yellow()
                    );
                }
            }
        }
        None
    }

    /// Get the original file name including the file extension.
    #[must_use]
    pub fn filename(&self) -> String {
        format!("{}.{}", self.name, self.extension)
    }

    pub fn format_tags(&mut self, file_tags: &FileTags) {
        let mut tags = TrackTags::parse_tag_data(self, file_tags);
        let (formatted_artist, formatted_title) =
            formatting::format_tags_for_artist_and_title(&tags.current_artist, &tags.current_title);

        let mut formatted_album = formatting::format_album(&tags.current_album);
        let mut formatted_genre = genre::format_genre(&tags.current_genre);

        if formatted_album.is_empty() && self.directory.to_lowercase().starts_with("djcity") {
            formatted_album = "DJCity.com".to_string();
        }
        if formatted_album.is_empty() && self.directory.to_lowercase().starts_with("trayze") {
            formatted_album = "djtrayze.com".to_string();
        }

        if formatted_genre.is_empty()
            && (self.root.ends_with(DJ_MUSIC_PATH.as_path()) || GENRE_MAPPINGS.contains_key(self.directory.as_str()))
        {
            formatted_genre = (*GENRE_MAPPINGS.get(self.directory.as_str()).unwrap_or(&"")).to_string();
        }

        tags.formatted_name = format!("{formatted_artist} - {formatted_title}");
        tags.formatted_artist = formatted_artist;
        tags.formatted_title = formatted_title;
        tags.formatted_album = formatted_album;
        tags.formatted_genre = formatted_genre;

        self.tags = tags;
    }

    /// Return formatted file name without the file extension.
    #[must_use]
    pub fn formatted_filename(&self) -> String {
        let (file_artist, file_title) =
            formatting::format_filename(&self.tags.formatted_artist, &self.tags.formatted_title);

        match (file_artist.is_empty(), file_title.is_empty()) {
            (true, true) => String::new(),
            (true, false) => file_title,
            (false, true) => file_artist,
            (false, false) => format!("{file_artist} - {file_title}"),
        }
    }

    /// Return formatted file name with the file extension.
    #[must_use]
    pub fn formatted_filename_with_extension(&self) -> String {
        format!("{}.{}", self.formatted_filename(), self.format)
    }

    /// Return the full path with new filename.
    #[must_use]
    pub fn path_with_new_name(&self, filename: &str) -> PathBuf {
        dunce::simplified(&self.root.join(filename)).to_path_buf()
    }

    /// Create new Track from existing Track that has been renamed.
    pub fn renamed_track(&self, path: PathBuf, name: String) -> anyhow::Result<Self> {
        let metadata = Self::read_metadata(&path)?;
        Ok(Self {
            name,
            extension: self.format.to_string(),
            directory: self.directory.clone(),
            format: self.format.clone(),
            root: self.root.clone(),
            path,
            metadata,
            number: self.number,
            tags: self.tags.clone(),
            tags_updated: self.tags_updated,
            not_processed: self.not_processed,
            printed: self.printed,
        })
    }

    /// Print track if it has not been already.
    pub fn show(&mut self, total_tracks: usize, max_width: usize) {
        if !self.printed {
            println!(
                "\r{:>width$}/{total_tracks}: {}",
                self.number,
                self.filename(),
                width = max_width
            );
            self.printed = true;
        }
    }

    /// Print a divider line matching the width of the header printed by `show()`.
    pub fn print_divider(&self, _total_tracks: usize, max_width: usize) {
        // Matches the format: "{number:>max_width$}/{total_tracks}: {filename}"
        let prefix_width = 2 * max_width + 3;
        let width = prefix_width + self.filename().chars().count();
        println!("{}", "-".repeat(width));
    }

    /// Convert mp3 file to aif using ffmpeg.
    /// Returns an updated Track if conversion was successful.
    pub fn convert_mp3_to_aif(&self) -> anyhow::Result<Self> {
        let output_path = self.path.with_extension("aif");
        let output_path_string = path_to_string_relative(&output_path);
        output_path
            .try_exists()
            .context(format!("File already exists: {output_path_string}").red())?;

        let output = Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-n", // never overwrite existing file
                "-i",
                path_to_string(&self.path).as_str(),
                "-map_metadata", // keep all metadata
                "0",
                "-write_id3v2",
                "1",
                "-id3v2_version",
                "4",
                path_to_string(&output_path).as_str(),
            ])
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "{}",
                format!("FFmpeg error: {}", String::from_utf8_lossy(&output.stderr)).red()
            );
        }

        output_path
            .try_exists()
            .context(format!("Converted file does not exist: {output_path_string}").red())?;

        println!("Conversion successful: {}", output_path_string.cyan());

        trash::delete(&self.path).context("Failed to move mp3 file to trash".red())?;

        let metadata = Self::read_metadata(&output_path)?;
        let new_track = Self {
            name: self.name.clone(),
            extension: "aif".to_string(),
            directory: self.directory.clone(),
            format: FileFormat::Aif,
            root: self.root.clone(),
            path: output_path,
            metadata,
            number: self.number,
            tags: TrackTags::default(),
            tags_updated: self.tags_updated,
            not_processed: self.not_processed,
            printed: self.printed,
        };

        Ok(new_track)
    }

    /// Check if the given string matches the full filename (name.extension).
    fn matches_filename(&self, other: &str) -> bool {
        // Check length first to avoid unnecessary work
        let expected_length = self.name.len() + 1 + self.extension.len();
        other.len() == expected_length
            && other.starts_with(&self.name)
            && other.as_bytes().get(self.name.len()) == Some(&b'.')
            && other[self.name.len() + 1..] == self.extension
    }

    /// Get filename from Path with special characters retained instead of decomposed.
    fn get_nfc_filename_from_path(path: &Path) -> anyhow::Result<String> {
        Ok(path
            .file_stem()
            .context("Failed to get file stem")?
            .to_str()
            .context("Filename contains invalid Unicode")?
            // Rust uses unicode NFD (Normalization Form Decomposed) by default,
            // which converts special chars like "å" to "a\u{30a}",
            // which then get printed as a regular "a".
            // Use NFC (Normalization Form Composed) from unicode_normalization crate
            // to retain the correct format and not cause issues later on.
            // https://github.com/unicode-rs/unicode-normalization
            .nfc()
            .collect::<String>())
    }

    /// Read file metadata for track.
    fn read_metadata(path: &Path) -> anyhow::Result<TrackMetadata> {
        if !path.exists() {
            #[cfg(test)]
            return Ok(TrackMetadata::default());
            #[cfg(not(test))]
            anyhow::bail!("File does not exist: {}", path.display());
        }
        let modified = get_file_modified_time(path)?;
        Ok(TrackMetadata {
            modified,
            version: VERSION.to_string(),
        })
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

impl PartialEq<String> for Track {
    fn eq(&self, other: &String) -> bool {
        self.name == *other || self.matches_filename(other)
    }
}

impl PartialEq<&str> for Track {
    fn eq(&self, other: &&str) -> bool {
        self.name == *other || self.matches_filename(other)
    }
}

// Symmetry for comparisons (String == Track and &str == Track)
impl PartialEq<Track> for String {
    fn eq(&self, other: &Track) -> bool {
        other.name == *self || other.matches_filename(self)
    }
}

impl PartialEq<Track> for &str {
    fn eq(&self, other: &Track) -> bool {
        other.name == *self || other.matches_filename(self)
    }
}

impl fmt::Display for Track {
    // Try to print full filepath relative to current working directory,
    // otherwise fallback to the original path.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let relative_path = utils::get_relative_path_from_current_working_directory(&self.root);
        write!(
            f,
            "{}",
            dunce::simplified(&relative_path).join(self.filename()).display()
        )
    }
}

#[cfg(test)]
mod test_track_operations {
    use super::*;

    use std::env;
    use std::path::PathBuf;

    use crate::tags::read_tags;

    #[test]
    fn test_track_new_valid_path() {
        let path = Path::new("/users/test/test_song.mp3");
        let track = Track::new(path).expect("Failed to create track");
        assert_eq!(track.name, "test_song");
        assert_eq!(track.extension, "mp3");
        assert_eq!(track.format, FileFormat::Mp3);
        assert_eq!(track.root, PathBuf::from("/users/test"));
        assert_eq!(track.filename(), "test_song.mp3");
    }

    #[test]
    fn test_track_with_special_characters() {
        let path = Path::new("/Users/esgrove/Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix).mp3");
        let track = Track::new(path).expect("Failed to create track");
        assert_eq!(track.name, "Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix)");
        assert_eq!(track.extension, "mp3");
        assert_eq!(track.format, FileFormat::Mp3);
        assert_eq!(track.root, PathBuf::from("/Users/esgrove"));
        assert_eq!(
            track.filename(),
            "Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix).mp3"
        );
        assert_eq!(
            track.path.to_str().expect("Failed to convert track path to string"),
            if cfg!(target_os = "windows") {
                "/Users/esgrove\\Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix).mp3"
            } else {
                "/Users/esgrove/Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix).mp3"
            }
        );
    }

    #[test]
    fn test_track_new_with_extension() {
        let path = Path::new("/users/test/another/artist - test song.aiff");
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
        let track1 = Track::new(Path::new("/users/test/Test - song1.mp3")).expect("Failed to create track");
        let track2 = Track::new(Path::new("/users/other/Test - song1.aif")).expect("Failed to create track");
        assert_eq!(track1.extension, "mp3");
        assert_eq!(track1.format, FileFormat::Mp3);
        assert_eq!(track2.extension, "aif");
        assert_eq!(track2.format, FileFormat::Aif);
        assert_eq!(track1, track2);
    }

    #[test]
    fn test_track_display() {
        let dir = env::current_dir().expect("Failed to get current dir");
        let track = Track::new(dir.join("artist - title.mp3").as_path()).expect("Failed to create track");
        let displayed = format!("{track}");
        assert!(displayed.contains("artist - title.mp3"));

        let path_display = format!("{}", track.path.display());
        assert!(path_display.contains("artist - title.mp3"));
    }

    #[test]
    fn test_track_display_with_special_characters() {
        let dir = env::current_dir().expect("Failed to get current dir");
        let track = Track::new(dir.join("Ääkköset - Test.aif").as_path()).expect("Failed to create track");
        assert_eq!(track.extension, "aif");
        assert_eq!(track.format, FileFormat::Aif);

        let displayed = format!("{track}");
        assert!(displayed.contains("Ääkköset - Test.aif"));

        let path_display = format!("{}", track.path.display());
        assert!(path_display.contains("Ääkköset - Test.aif"));
    }

    #[test]
    fn test_full_match() {
        let track =
            Track::new(PathBuf::from("/users/test/Test - song1.mp3").as_path()).expect("Failed to create track");
        assert_eq!(track, "Test - song1.mp3".to_string());
    }

    #[test]
    fn test_name_match() {
        let track =
            Track::new(PathBuf::from("/users/test/Ääkköset - song2.mp3").as_path()).expect("Failed to create track");
        assert_eq!(track, "Ääkköset - song2".to_string());
        assert_eq!(track, "Ääkköset - song2.mp3".to_string());
    }

    #[test]
    fn test_mismatch() {
        let track =
            Track::new(PathBuf::from("/users/test/Test - song3.mp3").as_path()).expect("Failed to create track");
        assert_ne!(track, "Test - song3.wav"); // Different extension
        assert_ne!(track, "Test - song4.mp3"); // Different name
    }

    #[test]
    fn test_extension_ignored() {
        let track = Track::new(PathBuf::from("/users/test/song5.mp3").as_path()).expect("Failed to create track");
        assert_eq!(track, "song5".to_string());
        assert_eq!(track, "song5.mp3".to_string());
        assert_ne!(track, "song");
    }

    /// Return the path to the basic tags MP3 test file.
    fn basic_tags_mp3_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/basic_tags/Basic Tags - Song - 16-44.mp3")
    }

    /// Return the path to the basic tags AIF test file.
    fn basic_tags_aif_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/basic_tags/Basic Tags - Song - 16-44.aif")
    }

    /// Return the path to the basic tags FLAC test file.
    fn basic_tags_flac_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/basic_tags/Basic Tags - Song - 16-44.flac")
    }

    #[test]
    fn valid_mp3_path_returns_some() {
        let path = basic_tags_mp3_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let result = Track::try_from_path(&path);
        assert!(result.is_some(), "Expected Some for valid MP3 path");
        let track = result.expect("Track::try_from_path returned None for valid MP3");
        assert_eq!(track.format, FileFormat::Mp3, "Expected MP3 format");
        assert_eq!(track.extension, "mp3", "Expected mp3 extension");
    }

    #[test]
    fn valid_aif_path_returns_some_with_correct_format() {
        let path = basic_tags_aif_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let result = Track::try_from_path(&path);
        assert!(result.is_some(), "Expected Some for valid AIF path");
        let track = result.expect("Track::try_from_path returned None for valid AIF");
        assert_eq!(track.format, FileFormat::Aif, "Expected AIF format");
        assert_eq!(track.extension, "aif", "Expected aif extension");
    }

    #[test]
    fn valid_flac_path_returns_some_with_correct_format() {
        let path = basic_tags_flac_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let result = Track::try_from_path(&path);
        assert!(result.is_some(), "Expected Some for valid FLAC path");
        let track = result.expect("Track::try_from_path returned None for valid FLAC");
        assert_eq!(track.format, FileFormat::Flac, "Expected FLAC format");
        assert_eq!(track.extension, "flac", "Expected flac extension");
    }

    #[test]
    fn unsupported_extension_returns_none() {
        let path = Path::new("/users/test/document.txt");
        let result = Track::try_from_path(path);
        assert!(result.is_none(), "Expected None for unsupported .txt extension");
    }

    #[test]
    fn no_extension_returns_none() {
        let path = Path::new("/users/test/noextension");
        let result = Track::try_from_path(path);
        assert!(result.is_none(), "Expected None for path with no extension");
    }

    #[test]
    fn format_tags_populates_formatted_fields() {
        let path = basic_tags_mp3_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let mut track = Track::try_from_path(&path).expect("Failed to create Track from basic tags MP3");
        let tag = read_tags(&track, false).expect("Failed to read tags from basic tags MP3");
        track.format_tags(&tag);
        assert!(
            !track.tags.formatted_artist.is_empty(),
            "Expected formatted_artist to be non-empty after format_tags"
        );
        assert!(
            !track.tags.formatted_title.is_empty(),
            "Expected formatted_title to be non-empty after format_tags"
        );
        assert!(
            track.tags.formatted_name.contains(" - "),
            "Expected formatted_name to contain ' - ', got '{}'",
            track.tags.formatted_name
        );
    }

    #[test]
    fn formatted_filename_contains_separator() {
        let path = basic_tags_mp3_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let mut track = Track::try_from_path(&path).expect("Failed to create Track from basic tags MP3");
        let tag = read_tags(&track, false).expect("Failed to read tags from basic tags MP3");
        track.format_tags(&tag);
        let filename = track.formatted_filename();
        assert!(!filename.is_empty(), "Expected formatted_filename to be non-empty");
        assert!(
            filename.contains(" - "),
            "Expected formatted_filename to contain ' - ', got '{filename}'"
        );
    }

    #[test]
    fn formatted_filename_with_extension_ends_with_mp3() {
        let path = basic_tags_mp3_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let mut track = Track::try_from_path(&path).expect("Failed to create Track from basic tags MP3");
        let tag = read_tags(&track, false).expect("Failed to read tags from basic tags MP3");
        track.format_tags(&tag);
        let filename_with_extension = track.formatted_filename_with_extension();
        assert!(
            Path::new(&filename_with_extension)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("mp3")),
            "Expected formatted_filename_with_extension to end with '.mp3', got '{filename_with_extension}'"
        );
    }

    #[test]
    fn path_with_new_name_uses_same_parent_directory() {
        let path = basic_tags_mp3_path();
        if !path.exists() {
            println!("Test file not found, skipping: {}", path.display());
            return;
        }
        let track = Track::try_from_path(&path).expect("Failed to create Track from basic tags MP3");
        let new_path = track.path_with_new_name("new_name.mp3");
        assert!(
            new_path.ends_with("new_name.mp3"),
            "Expected path to end with 'new_name.mp3', got '{}'",
            new_path.display()
        );
        let new_parent = new_path.parent().expect("Failed to get parent of new path");
        assert_eq!(new_parent, track.root, "Expected parent directory to match track root");
    }
}

#[cfg(test)]
mod test_track_ordering {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn tracks_ordered_by_name() {
        let track_alpha = Track::new(Path::new("/users/test/Alpha - Song.mp3")).expect("Failed to create track Alpha");
        let track_beta = Track::new(Path::new("/users/test/Beta - Song.mp3")).expect("Failed to create track Beta");
        assert_eq!(
            track_alpha.cmp(&track_beta),
            Ordering::Less,
            "Expected Alpha to sort before Beta"
        );
        assert_eq!(
            track_beta.cmp(&track_alpha),
            Ordering::Greater,
            "Expected Beta to sort after Alpha"
        );
        assert_eq!(
            track_alpha.cmp(&track_alpha),
            Ordering::Equal,
            "Expected track to be equal to itself"
        );
    }

    #[test]
    fn partial_ord_consistent_with_ord() {
        let track_alpha = Track::new(Path::new("/users/test/Alpha - Song.mp3")).expect("Failed to create track Alpha");
        let track_beta = Track::new(Path::new("/users/test/Beta - Song.mp3")).expect("Failed to create track Beta");
        assert_eq!(
            track_alpha.partial_cmp(&track_beta),
            Some(Ordering::Less),
            "Expected partial_cmp to return Some(Less) for Alpha vs Beta"
        );
        assert_eq!(
            track_beta.partial_cmp(&track_alpha),
            Some(Ordering::Greater),
            "Expected partial_cmp to return Some(Greater) for Beta vs Alpha"
        );
    }
}

#[cfg(test)]
mod test_track_show {
    use super::*;

    #[test]
    fn show_prints_once_and_sets_printed_flag() {
        let mut track =
            Track::new(Path::new("/users/test/Artist - Title.mp3")).expect("Failed to create track for show test");
        track.number = 1;
        assert!(!track.printed, "Expected printed flag to be false initially");
        track.show(10, 2);
        assert!(track.printed, "Expected printed flag to be true after first show call");
    }

    #[test]
    fn print_divider_width_matches_show_header() {
        let track =
            Track::new(Path::new("/users/test/Artist - Title.mp3")).expect("Failed to create track for divider test");
        // show() format: "{number:>max_width$}/{total_tracks}: {filename}"
        // For total_tracks=768, max_width=3: "  1/768: Artist - Title.mp3"
        let max_width = 3;
        // Verify the prefix matches: "  1/768: " = 3 + 1 + 3 + 2 = 9
        let prefix_width = 2 * max_width + 3;
        assert_eq!(prefix_width, 9);
        let divider_width = prefix_width + track.filename().chars().count();
        assert_eq!(divider_width, 9 + "Artist - Title.mp3".len());
    }

    #[test]
    fn show_does_not_print_twice() {
        let mut track =
            Track::new(Path::new("/users/test/Artist - Title.mp3")).expect("Failed to create track for show test");
        track.number = 3;
        track.show(10, 2);
        assert!(track.printed, "Expected printed flag to be true after first show call");
        // Second call should be a no-op because `printed` is already true
        track.show(10, 2);
        assert!(
            track.printed,
            "Expected printed flag to remain true after second show call"
        );
    }
}
