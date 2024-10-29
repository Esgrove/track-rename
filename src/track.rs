use std::cmp::Ordering;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::LazyLock;

use anyhow::Context;
use colored::Colorize;
use id3::Tag;
use unicode_normalization::UnicodeNormalization;

use crate::file_format::FileFormat;
use crate::genre::GENRE_MAPPINGS;
use crate::state::TrackMetadata;
use crate::tags::TrackTags;
use crate::utils;
use crate::utils::{get_file_modified_time, path_to_string, path_to_string_relative};
use crate::{formatting, genre};

// Other audio file extensions that should trigger a warning message,
const OTHER_FILE_EXTENSIONS: [&str; 3] = ["wav", "flac", "m4a"];
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub static DJ_MUSIC_PATH: LazyLock<PathBuf> = LazyLock::new(|| ["Dropbox", "DJ MUSIC"].iter().collect());

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
                };
            }
        }
        None
    }

    /// Get the original file name including the file extension.
    #[must_use]
    pub fn filename(&self) -> String {
        format!("{}.{}", self.name, self.extension)
    }

    pub fn format_tags(&mut self, file_tags: &Tag) {
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
            directory: self.directory.to_string(),
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
            directory: self.directory.to_string(),
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
        format!("{}.{}", self.name, self.extension) == *other || self.name == *other
    }
}

impl PartialEq<&str> for Track {
    fn eq(&self, other: &&str) -> bool {
        format!("{}.{}", self.name, self.extension) == *other || self.name == *other
    }
}

// Symmetry for comparisons (String == Track and &str == Track)
impl PartialEq<Track> for String {
    fn eq(&self, other: &Track) -> bool {
        *self == format!("{}.{}", other.name, other.extension) || *self == other.name
    }
}

impl PartialEq<Track> for &str {
    fn eq(&self, other: &Track) -> bool {
        *self == format!("{}.{}", other.name, other.extension) || *self == other.name
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
mod tests {
    use super::*;

    use std::env;
    use std::path::PathBuf;

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
        let path = Path::new("/Users/akseli/Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix).mp3");
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
            if cfg!(target_os = "windows") {
                "/Users/akseli\\Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix).mp3"
            } else {
                "/Users/akseli/Räntä & Benjamin Mùll - Sippa På En Tequila (Ö Remix).mp3"
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
}
