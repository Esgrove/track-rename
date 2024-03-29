use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::string::String;
use std::time::Instant;

use anyhow::{Context, Result};
use colored::Colorize;
use id3::{Tag, TagLike};
use unicode_normalization::UnicodeNormalization;
use walkdir::WalkDir;

use crate::cli_config::CliConfig;
use crate::file_format::FileFormat;
use crate::statistics::Statistics;
use crate::track::Track;
use crate::user_config::UserConfig;
use crate::{formatter, utils, RenamerArgs};

/// Audio track tag and filename formatting.
#[derive(Debug, Default)]
pub struct Renamer {
    root: PathBuf,
    config: CliConfig,
    user_config: UserConfig,
    tracks: Vec<Track>,
    total_tracks: usize,
    stats: Statistics,
}

impl Renamer {
    /// Create Renamer from command line arguments.
    pub fn new(path: PathBuf, args: RenamerArgs) -> Renamer {
        let user_config = UserConfig::get_user_config();
        let mut config = CliConfig::from_args(args);
        config.convert_failed = config.convert_failed || user_config.convert_failed;
        Renamer {
            root: path,
            config,
            user_config,
            ..Default::default()
        }
    }

    #[cfg(test)]
    /// Create Renamer with config directly. Used in tests.
    pub fn new_with_config(path: PathBuf, config: CliConfig) -> Renamer {
        Renamer {
            root: path,
            config,
            ..Default::default()
        }
    }

    /// Gather and process supported audio files.
    pub fn run(&mut self) -> Result<()> {
        if self.config.debug {
            println!("{}", self.config);
            println!("{}", self.user_config);
        }

        if self.config.convert_failed && !utils::ffmpeg_available() {
            anyhow::bail!("Convert failed specified but ffmpeg command was not found!")
        }

        self.gather_files()?;
        self.process_tracks()?;
        Ok(())
    }

    /// Gather audio files recursively from the root path.
    pub fn gather_files(&mut self) -> Result<()> {
        let start_instant = Instant::now();
        let track_list: Vec<Track> = if self.root.is_file() {
            if let Some(mut track) = Track::try_from_path(&self.root) {
                track.number = 1;
                vec![track]
            } else {
                Vec::new()
            }
        } else {
            self.get_tracks_from_root_directory()
        };

        if track_list.is_empty() {
            anyhow::bail!("no supported audio files found");
        }

        self.total_tracks = track_list.len();
        self.tracks = track_list;

        if self.config.verbose {
            if self.total_tracks < 100 {
                let index_width: usize = self.total_tracks.to_string().chars().count();
                for track in self.tracks.iter() {
                    println!("{:>width$}: {}", track.number, track, width = index_width);
                }
            }
            self.print_extension_counts();
        }
        if self.config.debug {
            let duration = start_instant.elapsed();
            println!("Time taken: {:.3}s", duration.as_secs_f64());
        }
        Ok(())
    }

    /// Find and return a list of audio tracks from the root directory.
    fn get_tracks_from_root_directory(&mut self) -> Vec<Track> {
        if self.config.verbose {
            println!(
                "Getting audio files from: {}",
                format!("{}", self.root.display()).cyan()
            );
        }
        let mut track_list: Vec<Track> = Vec::new();
        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            if let Some(track) = Track::try_from_path(entry.path()) {
                track_list.push(track);
            }
        }
        if self.config.sort_files {
            track_list.sort();
        }
        for (number, track) in track_list.iter_mut().enumerate() {
            track.number = number + 1;
        }
        track_list
    }

    // Format tags and rename files if needed.
    pub fn process_tracks(&mut self) -> Result<()> {
        println!("{}", format!("Processing {} tracks...", self.total_tracks).bold());
        if self.config.print_only {
            println!("{}", "Running in print-only mode".yellow().bold())
        }
        let start_instant = Instant::now();
        let mut failed_files: Vec<String> = Vec::new();
        let mut processed_files: HashMap<String, Vec<Track>> = HashMap::new();
        let mut current_path = self.root.clone();
        for track in self.tracks.iter_mut() {
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
                .iter()
                .any(|excluded_file| excluded_file == track)
            {
                if self.config.verbose {
                    track.show(self.total_tracks);
                    let message = format!("Skipping track in exclude list: {}", track);
                    println!("{}", message.yellow());
                    utils::print_divider(&message);
                }
                continue;
            }

            // File might have been deleted between gathering files and now,
            // for example when handling duplicates.
            if !track.path.exists() {
                track.show(self.total_tracks);
                let message = format!("Track no longer exists: {}", track);
                eprintln!("{}", message.red());
                utils::print_divider(&message);
                continue;
            }

            let mut tag_result = utils::read_tags(track);
            if tag_result.is_none() && self.config.convert_failed && track.format == FileFormat::Mp3 {
                println!("Converting mp3 file...");
                match track.convert_mp3_to_aif() {
                    Ok(aif_track) => {
                        self.stats.num_converted += 1;
                        *track = aif_track;
                        tag_result = utils::read_tags(track);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
            let mut tags = match tag_result {
                Some(tag) => tag,
                None => {
                    self.stats.num_failed += 1;
                    if self.config.log_failures {
                        failed_files.push(utils::path_to_string(&track.path));
                    }
                    continue;
                }
            };

            let (artist, title, current_tags) = Self::parse_artist_and_title(track, &tags);
            let (formatted_artist, formatted_title) = formatter::format_tags(&artist, &title);
            let formatted_tags = format!("{} - {}", formatted_artist, formatted_title);
            let (file_artist, file_title) = formatter::format_filename(&formatted_artist, &formatted_title);
            if current_tags != formatted_tags {
                self.stats.num_tags += 1;
                track.show(self.total_tracks);
                println!("{}", "Fix tags:".blue().bold());
                utils::show_diff(&current_tags, &formatted_tags);
                if !self.config.print_only && (self.config.force || utils::confirm()) {
                    tags.set_artist(formatted_artist);
                    tags.set_title(formatted_title);
                    if let Err(error) = tags.write_to_path(&track.path, id3::Version::Id3v24) {
                        eprintln!("{}", format!("Failed to write tags: {}", error).red());
                    }
                    track.tags_updated = true;
                    self.stats.num_tags_fixed += 1;
                }
                utils::print_divider(&formatted_tags);
            }

            let formatted_name = if file_artist.is_empty() {
                file_title
            } else {
                format!("{} - {}", file_artist, file_title)
            };

            if self.config.tags_only {
                processed_files.entry(formatted_name).or_default().push(track.clone());
                continue;
            }

            let formatted_file_name = format!("{}.{}", formatted_name, track.format);
            let formatted_path = track.path_with_new_name(&formatted_file_name);

            // Convert paths to strings for additional comparisons.
            // macOS and Windows paths are case-insensitive by default,
            // so `is_file()` will ignore differences in capitalization.
            let formatted_path_string = utils::path_to_string_relative(&formatted_path);
            let original_path_string = utils::path_to_string_relative(&track.path);

            if formatted_path_string != original_path_string {
                let mut capitalization_change_only = false;
                if formatted_path_string.to_lowercase() == original_path_string.to_lowercase() {
                    // File path contains only capitalization changes:
                    // Need to use a temp file to workaround case-insensitive file systems.
                    capitalization_change_only = true;
                }
                if !formatted_path.is_file() || capitalization_change_only {
                    // Rename files if the flag was given or if tags were not changed
                    if self.config.rename_files || !track.tags_updated {
                        track.show(self.total_tracks);
                        println!("{}", "Rename file:".yellow().bold());
                        utils::show_diff(&track.filename(), &formatted_file_name);
                        self.stats.num_to_rename += 1;
                        if !self.config.print_only && (self.config.force || utils::confirm()) {
                            if capitalization_change_only {
                                let temp_file = formatted_path.with_extension(format!("{}.{}", track.format, "tmp"));
                                utils::rename_track(&track.path, &temp_file, self.config.test_mode)?;
                                utils::rename_track(&temp_file, &formatted_path, self.config.test_mode)?;
                            } else {
                                utils::rename_track(&track.path, &formatted_path, self.config.test_mode)?;
                            }
                            if self.config.test_mode && formatted_path.exists() {
                                fs::remove_file(formatted_path).context("Failed to remove renamed file")?;
                            } else {
                                *track = track.renamed_track(formatted_path, formatted_name.clone());
                            }
                            self.stats.num_renamed += 1;
                        }
                        utils::print_divider(&formatted_file_name);
                    }
                } else if formatted_path != track.path {
                    // A file with the new name already exists
                    track.show(self.total_tracks);
                    println!("{}", "Duplicate:".bright_red().bold());
                    println!("Rename:   {}", original_path_string);
                    println!("Existing: {}", formatted_path_string);
                    utils::print_divider(&formatted_file_name);
                    self.stats.num_duplicates += 1;
                }
            }

            processed_files.entry(formatted_name).or_default().push(track.clone());
        }

        println!("{}", "Finished".green());
        if self.config.debug {
            let duration = start_instant.elapsed();
            println!("Time taken: {:.3}s", duration.as_secs_f64());
        }
        println!("{}", self.stats);
        if self.config.log_failures && !failed_files.is_empty() {
            utils::write_log_for_failed_files(&failed_files)?;
        }

        Self::print_all_duplicates(processed_files);

        Ok(())
    }

    /// Count and print the total number of each file extension in the file list.
    fn print_extension_counts(&self) {
        let mut file_format_counts: HashMap<String, usize> = HashMap::new();

        for track in &self.tracks {
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

    /// Try to read artist and title from tags.
    /// Fallback to parsing them from filename if tags are empty.
    fn parse_artist_and_title(track: &Track, tag: &Tag) -> (String, String, String) {
        let mut artist = String::new();
        let mut title = String::new();
        let mut current_tags = " - ".to_string();

        match (tag.artist(), tag.title()) {
            (Some(a), Some(t)) => {
                artist = a.nfc().collect::<String>();
                title = t.nfc().collect::<String>();
                current_tags = format!("{} - {}", artist, title);
            }
            (None, None) => {
                eprintln!("{}", format!("Missing tags: {}", track.path.display()).yellow());
                if let Some((a, t)) = utils::get_tags_from_filename(&track.name) {
                    artist = a;
                    title = t;
                }
            }
            (None, Some(t)) => {
                eprintln!("{}", format!("Missing artist tag: {}", track.path.display()).yellow());
                if let Some((a, _)) = utils::get_tags_from_filename(&track.name) {
                    artist = a;
                }
                title = t.nfc().collect::<String>();
                current_tags = format!(" - {}", title);
            }
            (Some(a), None) => {
                eprintln!("{}", format!("Missing title tag: {}", track.path.display()).yellow());
                artist = a.nfc().collect::<String>();
                if let Some((_, t)) = utils::get_tags_from_filename(&track.name) {
                    title = t;
                }
                current_tags = format!("{} - ", artist);
            }
        }
        (artist, title, current_tags)
    }

    /// Print all paths for duplicate tracks with the same name.
    fn print_all_duplicates(processed_files: HashMap<String, Vec<Track>>) {
        // Get all tracks with multiple paths for the same name.
        // Convert to vector so names can be sorted.
        let mut duplicate_tracks: Vec<(String, Vec<Track>)> = processed_files
            .into_iter()
            .filter(|(_, tracks)| tracks.len() > 1)
            .collect();

        if duplicate_tracks.is_empty() {
            return;
        }

        duplicate_tracks.sort();

        println!(
            "{}",
            format!("Duplicates ({}):", duplicate_tracks.len()).magenta().bold()
        );
        for (name, tracks) in duplicate_tracks.iter() {
            println!("{}", name.yellow());
            for track in tracks {
                println!("  {}", track);
            }
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

    /// Generic test function that takes a function or closure with one PathBuf as input argument.
    /// It will create temporary test files and run the test function with them.
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

        let test_file_name = format!(
            "{} ({}).{}",
            file_stem.to_string_lossy(),
            random_string,
            extension.to_string_lossy()
        );

        let temp_file_path = temp_dir_path.join(test_file_name);
        Some(temp_file_path)
    }
}
