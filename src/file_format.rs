use std::fmt::Display;
use std::str::FromStr;

use anyhow::{anyhow, Result};

/// Supported audio file formats.
// TODO: add support for "flac" and "m4a"
#[derive(Debug, Default, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum FileFormat {
    #[default]
    Mp3,
    Aif,
}

impl FromStr for FileFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mp3" => Ok(Self::Mp3),
            "aif" | "aiff" => Ok(Self::Aif),
            _ => Err(anyhow!("Unsupported file format: {}", s)),
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
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_valid_formats() {
        assert_eq!(FileFormat::from_str("mp3").unwrap(), FileFormat::Mp3);
        assert_eq!(FileFormat::from_str("Mp3").unwrap(), FileFormat::Mp3);
        assert_eq!(FileFormat::from_str("MP3").unwrap(), FileFormat::Mp3);
        assert_eq!(FileFormat::from_str("aif").unwrap(), FileFormat::Aif);
        assert_eq!(FileFormat::from_str("aiff").unwrap(), FileFormat::Aif);
        assert_eq!(FileFormat::from_str("Aif").unwrap(), FileFormat::Aif);
        assert_eq!(FileFormat::from_str("Aiff").unwrap(), FileFormat::Aif);
        assert_eq!(FileFormat::from_str("AIF").unwrap(), FileFormat::Aif);
        assert_eq!(FileFormat::from_str("AIFF").unwrap(), FileFormat::Aif);
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
    }
}
