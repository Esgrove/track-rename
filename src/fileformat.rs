use anyhow::{anyhow, Result};
use std::fmt::Display;

use std::str::FromStr;

#[derive(Debug)]
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
