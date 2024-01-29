use anyhow::{anyhow, Result};
use std::fmt::Display;

use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum FileFormat {
    Mp3,
    Aif,
}

impl FromStr for FileFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mp3" => Ok(FileFormat::Mp3),
            "aif" | "aiff" => Ok(FileFormat::Aif),
            _ => Err(anyhow!("Unsupported file format: {}", s)),
        }
    }
}

impl Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            FileFormat::Mp3 => "mp3".to_string(),
            FileFormat::Aif => "aif".to_string(),
        };
        write!(f, "{}", str)
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
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", FileFormat::Mp3), "mp3");
        assert_eq!(format!("{}", FileFormat::Aif), "aif");
    }
}
