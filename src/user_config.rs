use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context};
use dirs::home_dir;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct UserConfig {
    /// Filenames to ignore
    pub exclude: FileExclusionList,
}

#[derive(Debug, Default, Deserialize)]
pub struct FileExclusionList {
    /// List of filenames to ignore
    pub files: Vec<String>,
}

pub fn get_user_config() -> UserConfig {
    read_user_config().unwrap_or_default()
}

fn read_user_config() -> Option<UserConfig> {
    if let Ok(path) = user_config_file_path() {
        if let Ok(config_string) = fs::read_to_string(path) {
            let config: UserConfig = toml::from_str(&config_string).ok()?;
            return Some(config);
        }
    }
    None
}

fn user_config_file_path() -> anyhow::Result<PathBuf> {
    let home_dir = home_dir().context("Failed to find home directory")?;
    let config_path = home_dir.join(".config/track-rename.toml");
    match config_path.exists() {
        true => Ok(config_path),
        false => Err(anyhow!("Config file not found: {}", config_path.display())),
    }
}
