use std::fmt;

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{utils, RenamerArgs};

/// Renamer CLI settings.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CliConfig {
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
}

impl CliConfig {
    /// Create config from command line args.
    pub fn from_args(args: RenamerArgs) -> Self {
        CliConfig {
            force: args.force,
            rename_files: args.rename,
            sort_files: args.sort,
            print_only: args.print,
            tags_only: args.tags_only,
            verbose: args.verbose,
            debug: args.debug,
            test_mode: args.test,
            log_failures: args.log,
            convert_failed: args.convert,
            write_all_tags: args.all_tags,
        }
    }

    #[cfg(test)]
    /// Used in tests.
    pub fn new_for_tests() -> Self {
        CliConfig {
            force: true,
            rename_files: true,
            test_mode: true,
            ..Default::default()
        }
    }
}

impl fmt::Display for CliConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Serialize the struct to a serde_json::Value in place of reflection
        // to automatically handle each member variable.
        writeln!(f, "{}", "CliConfig:".bold())?;
        let members = serde_json::to_value(self).expect("Failed to serialize CliConfig");
        if let serde_json::Value::Object(map) = members {
            for (key, value) in map {
                let bool_value = value.as_bool().expect("Expected a boolean value");
                writeln!(f, "  {}: {}", key, utils::colorize_bool(bool_value))?;
            }
        }
        Ok(())
    }
}
