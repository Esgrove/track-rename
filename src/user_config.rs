use std::{fmt, fs, path::PathBuf};

use anyhow::{anyhow, Context};
use colored::Colorize;
use dirs::home_dir;
use serde::Deserialize;

use track_rename::utils;

/// User config options from a config file.
#[derive(Debug, Default, Deserialize)]
pub struct UserConfig {
    /// Filenames to ignore
    pub exclude: Vec<String>,
    #[serde(default)]
    /// Convert files that could not be read to AIFF
    pub convert_failed: bool,
}

impl UserConfig {
    /// Try to read user config from file if it exists.
    /// Otherwise, fall back to default config.
    pub fn get_user_config() -> UserConfig {
        Self::read_user_config().unwrap_or_default()
    }

    /// Read and parse user config if it exists.
    fn read_user_config() -> Option<UserConfig> {
        Self::user_config_file_path()
            .ok()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|config_string| toml::from_str(&config_string).ok())
    }

    /// Get user config file if it exists.
    fn user_config_file_path() -> anyhow::Result<PathBuf> {
        let home_dir = home_dir().context("Failed to find home directory")?;
        let config_path = home_dir.join(".config/track-rename.toml");
        match config_path.exists() {
            true => Ok(config_path),
            false => Err(anyhow!("Config file not found: {}", config_path.display())),
        }
    }
}

impl fmt::Display for UserConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let excluded_files: String = self
            .exclude
            .iter()
            .map(|file| format!("    {}", file.yellow()))
            .collect::<Vec<_>>()
            .join("\n");

        writeln!(f, "{}", "UserConfig:".bold())?;
        writeln!(f, "  convert_failed: {}", utils::colorize_bool(self.convert_failed))?;
        if excluded_files.is_empty() {
            writeln!(f, "  exclude: []")
        } else {
            writeln!(f, "  exclude:\n{}", excluded_files)
        }
    }
}
