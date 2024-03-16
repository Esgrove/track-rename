use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::string::String;

use anyhow::{Context, Result};
use colored::Colorize;
use id3::TagLike;
use walkdir::WalkDir;

use crate::cli_config::CliConfig;
use crate::track::Track;
use crate::user_config::{get_user_config, UserConfig};
use crate::{formatter, utils, RenamerArgs};

/// Audio track tag and filename formatting.
#[derive(Debug)]
pub struct Renamer {
    root: PathBuf,
    config: CliConfig,
    user_config: UserConfig,
    file_list: Vec<Track>,
    total_tracks: usize,
    num_tags: usize,
    num_tags_fixed: usize,
    num_to_rename: usize,
    num_renamed: usize,
    num_to_remove: usize,
    num_removed: usize,
    num_duplicates: usize,
    num_failed: usize,
}

impl Renamer {
    /// Create Renamer from command line arguments.
    pub fn new(path: PathBuf, args: RenamerArgs) -> Renamer {
        Renamer {
            root: path,
            config: CliConfig::from_args(args),
            user_config: get_user_config(),
            file_list: Vec::new(),
            total_tracks: 0,
            num_tags: 0,
            num_tags_fixed: 0,
            num_to_rename: 0,
            num_renamed: 0,
            num_to_remove: 0,
            num_removed: 0,
            num_duplicates: 0,
            num_failed: 0,
        }
    }

    #[cfg(test)]
    /// Create Renamer with config directly. Used in tests.
    pub fn new_with_config(path: PathBuf, config: CliConfig) -> Renamer {
        Renamer {
            root: path,
            config,
            user_config: UserConfig::default(),
            file_list: Vec::new(),
            total_tracks: 0,
            num_tags: 0,
            num_tags_fixed: 0,
            num_to_rename: 0,
            num_renamed: 0,
            num_to_remove: 0,
            num_removed: 0,
            num_duplicates: 0,
            num_failed: 0,
        }
    }

    /// Gather and process supported audio files.
    pub fn run(&mut self) -> Result<()> {
        if self.config.debug {
            println!("{:?}", self.config);
            println!("{:?}", self.user_config);
        }
        self.gather_files()?;
        self.process_files()?;
        self.print_stats();
        Ok(())
    }

    /// Gather audio files recursively from the root path.
    pub fn gather_files(&mut self) -> Result<()> {
        let mut file_list: Vec<Track> = Vec::new();
        if self.root.is_file() {
            if let Some(track) = Track::try_from_path(&self.root) {
                file_list.push(track);
            }
        } else {
            println!("Getting audio files from {}", format!("{}", self.root.display()).cyan());
            file_list = self.get_tracks_from_root_directory();
        }
        if file_list.is_empty() {
            anyhow::bail!("no supported audio files found");
        }

        self.total_tracks = file_list.len();
        if self.config.sort_files {
            file_list.sort();
        }

        self.file_list = file_list;
        if self.config.verbose {
            if self.file_list.len() < 100 {
                let index_width: usize = self.file_list.len().to_string().chars().count();
                for (number, track) in self.file_list.iter().enumerate() {
                    println!("{:>width$}: {}", number + 1, track, width = index_width);
                }
            }
            self.print_extension_counts();
        }

        Ok(())
    }

    /// Find and return a list of audio tracks from the root directory.
    fn get_tracks_from_root_directory(&mut self) -> Vec<Track> {
        let mut file_list: Vec<Track> = Vec::new();
        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            if let Some(track) = Track::try_from_path(entry.path()) {
                file_list.push(track);
            }
        }
        file_list
    }

    /// Format all tracks.
    pub fn process_files(&mut self) -> Result<()> {
        println!("{}", format!("Processing {} tracks...", self.total_tracks).bold());
        if self.config.print_only {
            println!("{}", "Running in print-only mode".yellow().bold())
        }
        let mut failed_files: Vec<String> = Vec::new();
        let mut current_path = self.root.clone();
        for (number, track) in self.file_list.iter_mut().enumerate() {
            if !self.config.sort_files {
                // Print current directory when iterating in directory order
                if current_path != track.root {
                    current_path = track.root.clone();
                    println!(
                        "{}",
                        match current_path.strip_prefix(&self.root) {
                            Ok(relative_path) => format!("{}", relative_path.display()).magenta(),
                            Err(_) => format!("{}", current_path.display()).magenta(),
                        }
                    );
                }
            }

            // Skip filenames in user configs exclude list
            if self
                .user_config
                .exclude
                .files
                .iter()
                .any(|excluded_file| excluded_file == track)
            {
                if self.config.verbose {
                    track.show(number + 1, self.total_tracks);
                    let message = format!("Skipping track in exclude list: {}", track);
                    println!("{}", message.yellow());
                    utils::print_divider(&message);
                }
                continue;
            }

            // File might have been deleted between gathering files and now,
            // for example when handling duplicates.
            if !track.path.exists() {
                track.show(number + 1, self.total_tracks);
                let message = format!("Track no longer exists: {}", track);
                eprintln!("{}", message.red());
                utils::print_divider(&message);
                continue;
            }

            let mut tags = match utils::read_tags(track) {
                Some(tag) => tag,
                None => {
                    self.num_failed += 1;
                    if self.config.log_failures {
                        failed_files.push(utils::path_to_string(&track.path));
                    }
                    continue;
                }
            };
            let (artist, title, current_tags) = utils::parse_artist_and_title(track, &tags);
            let (formatted_artist, formatted_title) = formatter::format_tags(&artist, &title);
            let formatted_tags = format!("{} - {}", formatted_artist, formatted_title);
            if current_tags != formatted_tags {
                self.num_tags += 1;
                track.show(number + 1, self.total_tracks);
                println!("{}", "Fix tags:".blue().bold());
                utils::show_diff(&current_tags, &formatted_tags);
                if !self.config.print_only && (self.config.force || utils::confirm()) {
                    tags.set_artist(formatted_artist.clone());
                    tags.set_title(formatted_title.clone());
                    if let Err(error) = tags.write_to_path(&track.path, id3::Version::Id3v24) {
                        eprintln!("{}", format!("Failed to write tags: {}", error).red());
                    }
                    track.tags_updated = true;
                    self.num_tags_fixed += 1;
                }
                utils::print_divider(&formatted_tags);
            }

            if self.config.tags_only {
                continue;
            }

            let (file_artist, file_title) = formatter::format_filename(&formatted_artist, &formatted_title);
            let new_file_name = if file_artist.is_empty() {
                format!("{}.{}", file_title, track.format)
            } else {
                format!("{} - {}.{}", file_artist, file_title, track.format)
            };

            let new_path = track.path_with_new_name(&new_file_name);

            // Convert paths to strings for additional comparisons.
            // macOS and Windows paths are case-insensitive by default,
            // so `is_file()` will ignore differences in capitalization.
            let new_path_string = utils::path_to_string_relative(&new_path);
            let original_path_string = utils::path_to_string_relative(&track.path);

            if new_path_string != original_path_string {
                let mut capitalization_change_only = false;
                if new_path_string.to_lowercase() == original_path_string.to_lowercase() {
                    // File path contains only capitalization changes:
                    // Need to use a temp file to workaround case-insensitive file systems.
                    capitalization_change_only = true;
                }
                if !new_path.is_file() || capitalization_change_only {
                    // Rename files if the flag was given or if tags were not changed
                    if self.config.rename_files || !track.tags_updated {
                        track.show(number + 1, self.total_tracks);
                        println!("{}", "Rename file:".yellow().bold());
                        utils::show_diff(&track.filename(), &new_file_name);
                        self.num_to_rename += 1;
                        if !self.config.print_only && (self.config.force || utils::confirm()) {
                            if capitalization_change_only {
                                let temp_file = new_path.with_extension(format!("{}.{}", track.format, "tmp"));
                                utils::rename_track(&track.path, &temp_file, self.config.test_mode)?;
                                utils::rename_track(&temp_file, &new_path, self.config.test_mode)?;
                            } else {
                                utils::rename_track(&track.path, &new_path, self.config.test_mode)?;
                            }
                            if self.config.test_mode && new_path.exists() {
                                fs::remove_file(new_path).context("Failed to remove renamed file")?;
                            }
                            self.num_renamed += 1;
                        }
                        utils::print_divider(&new_file_name);
                    }
                } else if new_path != track.path {
                    // A file with the new name already exists
                    track.show(number + 1, self.total_tracks);
                    println!("{}", "Duplicate:".bright_red().bold());
                    println!("Old: {}", original_path_string);
                    println!("New: {}", new_path_string);
                    utils::print_divider(&new_file_name);
                    self.num_duplicates += 1;
                }
            }

            // TODO: handle duplicates and same track in different file format
        }

        if self.config.log_failures && !failed_files.is_empty() {
            println!("Logging failed filepaths");
            utils::write_log_for_failed_files(&failed_files)?;
        }

        Ok(())
    }

    /// Count and print the total number of each file extension in the file list.
    fn print_extension_counts(&self) {
        let mut file_format_counts: HashMap<String, usize> = HashMap::new();

        for track in &self.file_list {
            *file_format_counts.entry(track.format.to_string()).or_insert(0) += 1;
        }

        // Collect the HashMap into a Vec for sorting
        let mut counts: Vec<(&String, &usize)> = file_format_counts.iter().collect();

        // Sort the Vec in decreasing order
        counts.sort_by(|a, b| b.1.cmp(a.1));

        println!("{}", "File format counts:".bold());
        for (format, count) in counts {
            println!("{}: {}", format, count);
        }
    }

    /// Print number of changes made.
    fn print_stats(&self) {
        println!("{}", "Finished".green());
        println!("Fix tags:   {} / {}", self.num_tags_fixed, self.num_tags);
        println!("Renamed:    {} / {}", self.num_renamed, self.num_to_rename);
        if self.num_to_remove > 0 {
            println!("Deleted:    {} / {}", self.num_removed, self.num_to_remove);
        }
        if self.num_duplicates > 0 {
            println!("Duplicate:  {}", self.num_duplicates);
        }
        if self.num_failed > 0 {
            println!("Failed:     {}", self.num_failed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::fs::copy;
    use std::path::Path;
    use std::path::PathBuf;

    use once_cell::sync::Lazy;
    use rand::{distributions::Alphanumeric, Rng};

    static NO_TAGS_DIR: Lazy<PathBuf> = Lazy::new(|| ["tests", "files", "no_tags"].iter().collect());
    static BASIC_TAGS_DIR: Lazy<PathBuf> = Lazy::new(|| ["tests", "files", "basic_tags"].iter().collect());
    static EXTENDED_TAGS_DIR: Lazy<PathBuf> = Lazy::new(|| ["tests", "files", "extended_tags"].iter().collect());

    #[test]
    fn test_no_tags() {
        run_test_on_files(&NO_TAGS_DIR, |temp_file| {
            let track = Track::try_from_path(&temp_file).expect("Failed to create Track for temp file");
            let tags = utils::read_tags(&track).expect("Tags should be present");
            assert!(tags.artist().is_none());
            assert!(tags.title().is_none());
            fs::remove_file(temp_file).expect("Failed to remove temp file");
        });
    }

    #[test]
    fn test_basic_tags() {
        run_test_on_files(&BASIC_TAGS_DIR, |temp_file| {
            let track = Track::try_from_path(&temp_file).expect("Failed to create Track for temp file");
            let tags = utils::read_tags(&track).expect("Tags should be present");
            assert!(!tags.artist().unwrap().is_empty());
            assert!(!tags.title().unwrap().is_empty());
            fs::remove_file(temp_file).expect("Failed to remove temp file");
        });
    }

    #[test]
    fn test_extended_tags() {
        run_test_on_files(&EXTENDED_TAGS_DIR, |temp_file| {
            let track = Track::try_from_path(&temp_file).expect("Failed to create Track for temp file");
            let tags = utils::read_tags(&track).expect("Tags should be present");
            assert!(!tags.artist().unwrap().is_empty());
            assert!(!tags.title().unwrap().is_empty());
            fs::remove_file(temp_file).expect("Failed to remove temp file");
        });
    }

    #[test]
    fn test_rename_no_tags() {
        run_test_on_files(&NO_TAGS_DIR, |temp_file| {
            let mut renamer = Renamer::new_with_config(temp_file, CliConfig::new_for_tests());
            renamer.run().expect("Rename failed");
        });
    }

    #[test]
    fn test_rename_basic_tags() {
        run_test_on_files(&BASIC_TAGS_DIR, |temp_file| {
            let mut renamer = Renamer::new_with_config(temp_file, CliConfig::new_for_tests());
            renamer.run().expect("Rename failed");
        });
    }

    #[test]
    fn test_rename_extended_tags() {
        run_test_on_files(&EXTENDED_TAGS_DIR, |temp_file| {
            let mut renamer = Renamer::new_with_config(temp_file, CliConfig::new_for_tests());
            renamer.run().expect("Rename failed");
        });
    }

    /// Generic test function that takes a function or closure with one PathBuf as input argument
    fn run_test_on_files<F: Fn(PathBuf)>(test_dir: &Path, test_func: F) {
        for entry in fs::read_dir(test_dir).expect("Failed to read test directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_file() && not_hidden_file(&path) {
                let temp_file = temp_test_file(&path).expect("Failed to create temp file path");
                copy(&path, &temp_file).expect("Failed to copy test file");
                assert!(temp_file.exists());
                test_func(temp_file.clone());
            }
        }
    }

    /// Check if this is a hidden file like ".DS_Store" on macOS
    fn not_hidden_file(path: &Path) -> bool {
        !path
            .file_name()
            .unwrap()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    }

    /// Create a new temporary file with an added random string in the name
    fn temp_test_file(path: &Path) -> Option<PathBuf> {
        let file_stem = path.file_stem()?.to_owned();
        let extension = path.extension()?.to_owned();
        let random_string: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        let temp_dir = format!("track-rename-{random_string}");
        let temp_dir_path = env::temp_dir().join(temp_dir);
        fs::create_dir_all(&temp_dir_path).expect("Failed to create temp subdir");

        let new_file_name = format!(
            "{} ({}).{}",
            file_stem.to_string_lossy(),
            random_string,
            extension.to_string_lossy()
        );

        let temp_file_path = temp_dir_path.join(new_file_name);
        Some(temp_file_path)
    }
}
