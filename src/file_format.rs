use std::fmt::Display;
use std::str::FromStr;

use anyhow::{Result, anyhow};

/// Supported audio file formats.
#[derive(Debug, Default, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum FileFormat {
    /// MPEG Layer III audio with ID3 metadata.
    #[default]
    Mp3,
    /// AIFF audio carrying ID3 metadata.
    Aif,
    /// FLAC audio with Vorbis comments.
    Flac,
}

impl FromStr for FileFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mp3" => Ok(Self::Mp3),
            "aif" | "aiff" => Ok(Self::Aif),
            "flac" => Ok(Self::Flac),
            _ => Err(anyhow!("Unsupported file format: {s}")),
        }
    }
}

impl Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Mp3 => "mp3",
                Self::Aif => "aif",
                Self::Flac => "flac",
            }
        )
    }
}

#[cfg(test)]
mod test_file_format_parsing {
    use super::*;

    #[test]
    fn test_from_str_valid_formats() {
        let cases = [
            ("mp3", FileFormat::Mp3),
            ("Mp3", FileFormat::Mp3),
            ("MP3", FileFormat::Mp3),
            ("aif", FileFormat::Aif),
            ("aiff", FileFormat::Aif),
            ("Aif", FileFormat::Aif),
            ("Aiff", FileFormat::Aif),
            ("AIF", FileFormat::Aif),
            ("AIFF", FileFormat::Aif),
            ("flac", FileFormat::Flac),
            ("Flac", FileFormat::Flac),
            ("FLAC", FileFormat::Flac),
        ];
        for (extension, expected) in cases {
            let format = FileFormat::from_str(extension).expect("Extension should parse as a file format");
            assert_eq!(format, expected, "Unexpected format for {extension}");
        }
    }

    #[test]
    fn test_from_str_invalid_format() {
        assert!(FileFormat::from_str("wav").is_err());
        assert!(FileFormat::from_str("m4a").is_err());
        assert!(FileFormat::from_str("zip").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", FileFormat::Mp3), "mp3");
        assert_eq!(format!("{}", FileFormat::Aif), "aif");
        assert_eq!(format!("{}", FileFormat::Flac), "flac");
    }
}
