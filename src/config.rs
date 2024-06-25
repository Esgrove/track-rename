use std::{fmt, fs};

use anyhow::{anyhow, Context};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::RenamerArgs;

use track_rename::utils;

const CONFIG_FILE_DIR: &str = ".config";
const CONFIG_FILE_NAME: &str = "track-rename.toml";

/// Renamer settings combined from CLI options and user config file.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub force: bool,
    pub rename_files: bool,
    pub sort_files: bool,
    pub print_only: bool,
    pub tags_only: bool,
    pub verbose: bool,
    pub debug: bool,
    pub test_mode: bool,
    pub log_failures: bool,
    pub convert_failed: bool,
    pub write_all_tags: bool,
    pub genre_statistics: bool,
    pub excluded_tracks: Vec<String>,
}

/// User config options from a config file.
#[derive(Debug, Default, Deserialize)]
struct UserConfig {
    /// Filenames to ignore
    pub exclude: Vec<String>,
    #[serde(default)]
    /// Convert files that could not be read to AIFF
    pub convert_failed: bool,
    #[serde(default)]
    pub genre_statistics: bool,
    #[serde(default)]
    pub log_failures: bool,
}

impl Config {
    /// Create config from given command line args and user config file.
    pub fn from_args(args: RenamerArgs) -> Self {
        let user_config = UserConfig::get_user_config();
        Config {
            force: args.force,
            rename_files: args.rename,
            sort_files: args.sort,
            print_only: args.print,
            tags_only: args.tags_only,
            verbose: args.verbose,
            debug: args.debug,
            test_mode: false,
            log_failures: args.log || user_config.log_failures,
            convert_failed: args.convert || user_config.convert_failed,
            write_all_tags: args.all_tags,
            genre_statistics: args.genre || user_config.genre_statistics,
            excluded_tracks: user_config.exclude,
        }
    }

    #[cfg(test)]
    /// Used in test cases.
    pub fn new_for_tests() -> Self {
        Config {
            force: true,
            rename_files: true,
            test_mode: true,
            ..Default::default()
        }
    }
}

impl UserConfig {
    /// Try to read user config from file if it exists.
    /// Otherwise, fall back to default config.
    fn get_user_config() -> UserConfig {
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
        let home_dir = dirs::home_dir().context("Failed to get home directory path")?;
        let config_path = home_dir.join(CONFIG_FILE_DIR).join(CONFIG_FILE_NAME);
        match config_path.exists() {
            true => Ok(config_path),
            false => Err(anyhow!("Config file not found: {}", config_path.display())),
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Serialize the struct to a serde_json::Value in place of reflection
        // to automatically handle each member variable.
        writeln!(f, "{}", "Config:".bold())?;
        writeln!(f, "  force: {}", utils::colorize_bool(self.force))?;
        writeln!(f, "  rename_files: {}", utils::colorize_bool(self.rename_files))?;
        writeln!(f, "  sort_files: {}", utils::colorize_bool(self.sort_files))?;
        writeln!(f, "  print_only: {}", utils::colorize_bool(self.print_only))?;
        writeln!(f, "  tags_only: {}", utils::colorize_bool(self.tags_only))?;
        writeln!(f, "  verbose: {}", utils::colorize_bool(self.verbose))?;
        writeln!(f, "  debug: {}", utils::colorize_bool(self.debug))?;
        writeln!(f, "  test_mode: {}", utils::colorize_bool(self.test_mode))?;
        writeln!(f, "  log_failures: {}", utils::colorize_bool(self.log_failures))?;
        writeln!(f, "  convert_failed: {}", utils::colorize_bool(self.convert_failed))?;
        writeln!(f, "  write_all_tags: {}", utils::colorize_bool(self.write_all_tags))?;
        writeln!(f, "  genre_statistics: {}", utils::colorize_bool(self.genre_statistics))?;
        if self.excluded_tracks.is_empty() {
            writeln!(f, "  excluded_tracks: []")?
        } else {
            let excluded_tracks: String = self
                .excluded_tracks
                .iter()
                .map(|name| format!("    {}", name.yellow()))
                .collect::<Vec<_>>()
                .join("\n");
            writeln!(f, "  excluded_tracks:\n{}", excluded_tracks)?
        }
        Ok(())
    }
}

impl fmt::Display for UserConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", "UserConfig:".bold())?;
        writeln!(f, "  convert_failed: {}", utils::colorize_bool(self.convert_failed))?;
        writeln!(f, "  genre_statistics: {}", utils::colorize_bool(self.convert_failed))?;
        writeln!(f, "  log_failures: {}", utils::colorize_bool(self.convert_failed))?;
        if self.exclude.is_empty() {
            writeln!(f, "  exclude: []")
        } else {
            let excluded_files: String = self
                .exclude
                .iter()
                .map(|name| format!("    {}", name.yellow()))
                .collect::<Vec<_>>()
                .join("\n");
            writeln!(f, "  exclude:\n{}", excluded_files)
        }
    }
}
