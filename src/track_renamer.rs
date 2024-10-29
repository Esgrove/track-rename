use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::string::String;
use std::time::Instant;

use anyhow::{Context, Result};
use colored::Colorize;
use id3::{Tag, TagLike};
use itertools::Itertools;
use rayon::prelude::*;

use crate::config::Config;
use crate::statistics::Statistics;
use crate::RenamerArgs;

use track_rename::file_format::FileFormat;
use track_rename::genre::GENRE_MAPPINGS;
use track_rename::serato;
use track_rename::state;
use track_rename::state::State;
use track_rename::track::{Track, DJ_MUSIC_PATH};
use track_rename::utils;

/// Audio track tag and filename formatting.
#[derive(Debug, Default)]
pub struct TrackRenamer {
    root: PathBuf,
    config: Config,
    state: State,
    stats: Statistics,
    tracks: Vec<Track>,
    tracks_count: usize,
}

impl TrackRenamer {
    /// Create Renamer from command line arguments.
    pub fn new(path: PathBuf, args: &RenamerArgs) -> Self {
        Self {
            root: path,
            config: Config::from_args(args),
            state: state::load_state(),
            ..Default::default()
        }
    }

    #[cfg(test)]
    /// Create Renamer with config directly. Used in tests.
    pub fn new_with_config(path: PathBuf, config: Config) -> Self {
        Self {
            root: path,
            config,
            ..Default::default()
        }
    }

    /// Gather and process supported audio files.
    pub fn run(&mut self) -> Result<()> {
        if self.config.debug {
            println!("{}", self.config);
            println!("State: {}", self.state.len());
        }

        if self.config.convert_failed && !utils::ffmpeg_available() {
            anyhow::bail!("Convert failed specified but ffmpeg command was not found!")
        }

        self.gather_files()?;
        self.process_tracks()?;
        self.update_state();
        if self.config.debug {
            println!("State: {}", self.state.len());
        }
        state::save_state(&self.state)?;
        Ok(())
    }

    /// Gather audio files recursively from the root path.
    pub fn gather_files(&mut self) -> Result<()> {
        let start_instant = Instant::now();
        let mut track_list: Vec<Track> = if self.root.is_file() {
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

        track_list.par_iter_mut().enumerate().for_each(|(number, track)| {
            track.number = number + 1;
        });

        self.tracks_count = track_list.len();
        self.tracks = track_list;

        if self.config.verbose {
            if self.tracks_count < 100 {
                let index_width: usize = self.tracks_count.to_string().chars().count();
                for track in &self.tracks {
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
    fn get_tracks_from_root_directory(&self) -> Vec<Track> {
        if self.config.verbose || self.config.debug {
            println!(
                "Getting audio files from: {}",
                format!("{}", self.root.display()).cyan()
            );
        }

        let mut track_list = utils::collect_tracks(&self.root);

        if self.config.sort_files {
            // Sort by filename, ignoring parent dir
            track_list.par_sort_unstable();
        } else {
            // Sort by full path so directories are in sorted order
            track_list.par_sort_unstable_by(|a, b| a.path.cmp(&b.path));
        }

        track_list
    }

    // Format tags and rename files if needed.
    pub fn process_tracks(&mut self) -> Result<()> {
        if self.tracks_count == 0 {
            println!("{}", "No tracks to process".green());
            return Ok(());
        } else if self.tracks_count == 1 {
            println!("{}", "Processing 1 track...".bold());
        } else {
            println!("{}", format!("Processing {} tracks...", self.tracks_count).bold());
        }

        let dryrun_header = if self.config.print_only {
            println!("{}", "Running in print-only mode".yellow().bold());
            " (dryrun)"
        } else {
            ""
        };
        let fix_tags_header = format!("Fix tags{dryrun_header}:").blue().bold();
        let rename_file_header = format!("Rename file{dryrun_header}:").cyan().bold();
        let max_index_width: usize = self.tracks_count.to_string().chars().count();

        let mut failed_files: Vec<String> = Vec::new();
        let mut processed_files: HashMap<String, Vec<Track>> = HashMap::new();
        let mut genres: HashMap<String, usize> = HashMap::new();
        let mut tag_versions: HashMap<String, usize> = HashMap::new();
        let mut checked_genre_mappings: HashSet<String> = HashSet::new();
        let mut current_path = self.root.clone();

        let start_instant = Instant::now();
        for track in &mut self.tracks {
            if !self.config.sort_files {
                // Print current directory when iterating in directory order
                if current_path != track.root {
                    current_path.clone_from(&track.root);
                    let path = utils::path_to_string_relative(&current_path);
                    if !path.is_empty() {
                        println!("\n{}", path.magenta());
                    }
                }
            }

            // If this is a DJ MUSIC subdirectory, check genre mappings
            if !checked_genre_mappings.contains(track.directory.as_str())
                && utils::contains_subpath(&track.root, DJ_MUSIC_PATH.as_path())
            {
                if !GENRE_MAPPINGS.contains_key(track.directory.as_str()) {
                    eprintln!(
                        "\n{}",
                        format!("WARNING: DJ music folder missing genre mapping: {}", track.directory).yellow()
                    );
                } else if GENRE_MAPPINGS.get(track.directory.as_str()).unwrap_or(&"").is_empty() {
                    eprintln!(
                        "\n{}",
                        format!("WARNING: Empty genre mapping for: {}", track.directory).yellow()
                    );
                }
                checked_genre_mappings.insert(track.directory.clone());
            }

            Self::print_running_index(self.tracks_count, track.number, max_index_width);

            // Skip filenames in user configs exclude list
            if self
                .config
                .excluded_tracks
                .iter()
                .any(|excluded_file| excluded_file == track)
            {
                if self.config.verbose {
                    track.show(self.tracks_count, max_index_width);
                    let message = format!("Skipping track in exclude list: {track}");
                    println!("{}", message.yellow());
                    utils::print_divider(&message);
                }
                continue;
            }

            // File might have been deleted between gathering files and now,
            // for example when handling duplicates.
            if !track.path.exists() {
                track.show(self.tracks_count, max_index_width);
                let message = format!("Track no longer exists: {track}");
                utils::print_error(&message);
                utils::print_divider(&message);
                continue;
            }

            let needs_processing = self.config.no_state
                || match self.state.get(&track.path) {
                    Some(state) => state.modified < track.metadata.modified || state.version != track.metadata.version,
                    None => true,
                };

            if needs_processing {
                let mut tag_result = utils::read_tags(track, self.config.verbose || self.config.debug);
                if tag_result.is_none() && self.config.convert_failed && track.format == FileFormat::Mp3 {
                    println!("Converting MP3 to AIF...");
                    match track.convert_mp3_to_aif() {
                        Ok(aif_track) => {
                            self.stats.converted += 1;
                            *track = aif_track;
                            tag_result = utils::read_tags(track, self.config.verbose || self.config.debug);
                        }
                        Err(e) => {
                            eprintln!("{e}");
                        }
                    }
                }
                let Some(mut file_tags) = tag_result else {
                    self.stats.failed += 1;
                    if self.config.log_failures {
                        failed_files.push(utils::path_to_string(&track.path));
                    }
                    continue;
                };

                // Store id3 tag version count
                *tag_versions.entry(file_tags.version().to_string()).or_insert(0) += 1;

                if self.config.debug && self.config.verbose {
                    utils::print_tag_data(&file_tags);
                    serato::print_serato_tags(&file_tags);
                }

                track.format_tags(&file_tags);
                let formatted_name = track.formatted_filename();
                if formatted_name.is_empty() {
                    eprintln!(
                        "\n{}",
                        format!("Formatted name should never be empty: {}", track.path.display()).red()
                    );
                }
                let tags_changed = track.tags.changed();
                if tags_changed || self.config.write_all_tags {
                    if tags_changed {
                        track.show(self.tracks_count, max_index_width);
                        self.stats.tags += 1;
                        println!("{fix_tags_header}");
                        track.tags.show_diff();
                    }
                    if !self.config.print_only
                        && (self.config.force || utils::confirm())
                        && Self::write_tags(track, &mut file_tags)
                    {
                        if tags_changed {
                            track.tags_updated = true;
                            self.stats.tags_fixed += 1;
                        }
                    } else {
                        track.not_processed = true;
                    }
                    if tags_changed {
                        utils::print_divider(&track.tags.formatted_name);
                    }
                }

                // Store unique genre count
                if !track.tags.formatted_genre.is_empty() {
                    *genres.entry(track.tags.formatted_genre.clone()).or_insert(0) += 1;
                }

                if self.config.tags_only {
                    processed_files
                        .entry(formatted_name.to_lowercase())
                        .or_default()
                        .push(track.clone());

                    continue;
                }

                let formatted_file_name = track.formatted_filename_with_extension();
                let formatted_path = track.path_with_new_name(&formatted_file_name);

                // Convert paths to strings for additional comparisons.
                // macOS and Windows paths are case-insensitive by default,
                // so `is_file()` will ignore differences in capitalization.
                let formatted_path_string = utils::path_to_string_relative(&formatted_path);
                let original_path_string = utils::path_to_string_relative(&track.path);

                if formatted_path_string != original_path_string {
                    let capitalization_change_only =
                        if formatted_path_string.to_lowercase() == original_path_string.to_lowercase() {
                            // File path contains only capitalization changes:
                            // Need to use a temp file to workaround case-insensitive file systems.
                            true
                        } else {
                            false
                        };
                    if !formatted_path.is_file() || self.config.overwrite_existing || capitalization_change_only {
                        // Rename files if the flag was given or if tags were not changed
                        if self.config.rename_files || !track.tags_updated {
                            track.show(self.tracks_count, max_index_width);
                            println!("{rename_file_header}");
                            utils::print_stacked_diff(&track.filename(), &formatted_file_name);
                            self.stats.to_rename += 1;
                            if !self.config.print_only && (self.config.force || utils::confirm()) {
                                if formatted_path.is_file() && self.config.overwrite_existing {
                                    println!(
                                        "{}",
                                        format!("Overwriting existing file: {formatted_path_string}").yellow()
                                    );
                                }
                                if capitalization_change_only {
                                    let temp_file =
                                        formatted_path.with_extension(format!("{}.{}", track.format, "tmp"));
                                    utils::rename_track(&track.path, &temp_file, self.config.test_mode)?;
                                    utils::rename_track(&temp_file, &formatted_path, self.config.test_mode)?;
                                } else {
                                    utils::rename_track(&track.path, &formatted_path, self.config.test_mode)?;
                                }
                                if self.config.test_mode && formatted_path.exists() {
                                    fs::remove_file(formatted_path).context("Failed to remove renamed file")?;
                                } else {
                                    // Update track data with the renamed path
                                    let renamed_track = track.renamed_track(formatted_path, formatted_name.clone())?;
                                    *track = renamed_track;
                                }
                                self.stats.renamed += 1;
                            } else {
                                track.not_processed = true;
                            }
                            utils::print_divider(&formatted_file_name);
                        }
                    } else if formatted_path != track.path {
                        // A file with the formatted name already exists
                        track.show(self.tracks_count, max_index_width);
                        println!("{}", "Duplicate:".bright_red().bold());
                        println!("Rename:   {original_path_string}");
                        println!("Existing: {formatted_path_string}");
                        utils::print_divider(&formatted_file_name);
                        self.stats.duplicates += 1;
                    }
                }
                processed_files
                    .entry(formatted_name.to_lowercase())
                    .or_default()
                    .push(track.clone());
            } else {
                processed_files
                    .entry(track.name.to_string())
                    .or_default()
                    .push(track.clone());
            }
        }

        println!("{}", "\nFinished".green());
        if self.config.debug {
            let duration = start_instant.elapsed();
            println!("Time taken: {:.3}s", duration.as_secs_f64());
        }
        println!("{}", self.stats);
        if self.config.log_failures && !failed_files.is_empty() {
            utils::write_log_for_failed_files(&failed_files)?;
        }
        if self.config.verbose {
            Self::print_tag_version_counts(tag_versions);
        }
        if self.config.genre_statistics {
            println!("{}", format!("Genres ({}):", genres.len()).cyan().bold());
            let mut genre_list: Vec<(String, usize)> =
                genres.into_iter().sorted_unstable_by(|a, b| b.1.cmp(&a.1)).collect();
            Self::print_top_genres(&genre_list);
            genre_list.sort_unstable();
            Self::write_genre_log(&genre_list)?;
        }
        Self::print_all_duplicates(processed_files);

        Ok(())
    }

    #[inline]
    /// Print running index
    fn print_running_index(total_tracks: usize, number: usize, max_index_width: usize) {
        print!("\r{number:>max_index_width$}/{total_tracks}");
        io::stdout().flush().expect("Failed to flush stdout");
    }

    /// Count and print the total number of each file extension in the file list.
    fn print_extension_counts(&self) {
        println!("{}", "File format counts:".bold());
        self.tracks
            .iter()
            .map(|track| track.format.to_string())
            .counts()
            .into_iter()
            .sorted_unstable_by(|a, b| b.1.cmp(&a.1))
            .for_each(|(format, count)| println!("{format}: {count}"));
    }

    fn update_state(&self) {
        self.tracks
            .par_iter()
            .filter(|track| !track.not_processed)
            .for_each(|track| {
                self.state.insert(track.path.clone(), track.metadata.clone());
            });
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

        duplicate_tracks.sort_unstable();

        println!(
            "{}",
            format!("Duplicates ({}):", duplicate_tracks.len()).magenta().bold()
        );
        for (_, tracks) in &duplicate_tracks {
            println!("{}", tracks[0].name.yellow());
            for track in tracks {
                println!("  {track}");
            }
        }
    }

    fn print_tag_version_counts(tag_versions: HashMap<String, usize>) {
        println!("{}", "Tag versions:".cyan().bold());
        let total: usize = tag_versions.values().sum();
        tag_versions
            .into_iter()
            .sorted_unstable_by(|a, b| b.1.cmp(&a.1))
            .map(|(tag, count)| {
                format!(
                    "{tag}   {count:>width$} ({:.1}%)",
                    count as f64 / total as f64 * 100.0,
                    width = total.to_string().chars().count()
                )
            })
            .for_each(|string| println!("{string}"));
    }

    fn print_top_genres(genre_list: &[(String, usize)]) {
        let max_length = genre_list
            .iter()
            .take(20)
            .map(|g| g.0.chars().count())
            .max()
            .unwrap_or(60);

        for (genre, count) in genre_list.iter().take(20) {
            println!("{genre:<max_length$}   {count}");
        }
    }

    /// Write a txt log file for failed tracks to current working directory.
    fn write_genre_log(genres: &[(String, usize)]) -> Result<()> {
        let filepath = Path::new("genres.txt");
        let mut file = File::create(filepath).context("Failed to create output file")?;
        for (genre, _) in genres {
            writeln!(file, "{genre}")?;
        }

        println!("Logged genres to: {}", dunce::canonicalize(filepath)?.display());
        Ok(())
    }

    fn write_tags(track: &Track, file_tags: &mut Tag) -> bool {
        // Remove genre first to try to get rid of old ID3v1 genre IDs
        file_tags.remove_genre();
        file_tags.remove_disc();
        file_tags.remove_total_discs();
        file_tags.remove_track();
        file_tags.remove_total_tracks();
        file_tags.remove_all_lyrics();
        file_tags.remove_all_synchronised_lyrics();
        if let Err(error) = file_tags.write_to_path(&track.path, id3::Version::Id3v24) {
            eprintln!(
                "\n{}",
                format!("Failed to remove tags for: {}\n{}", track.path.display(), error).red()
            );
        }
        file_tags.set_artist(track.tags.formatted_artist.clone());
        file_tags.set_title(track.tags.formatted_title.clone());
        file_tags.set_album(track.tags.formatted_album.clone());
        file_tags.set_genre(track.tags.formatted_genre.clone());
        if let Err(error) = file_tags.write_to_path(&track.path, id3::Version::Id3v24) {
            eprintln!(
                "\n{}",
                format!("Failed to write tags for: {}\n{}", track.path.display(), error).red()
            );
            false
        } else {
            true
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
    use std::sync::LazyLock;

    use rand::{distributions::Alphanumeric, Rng};

    static NO_TAGS_DIR: LazyLock<PathBuf> = LazyLock::new(|| ["tests", "files", "no_tags"].iter().collect());
    static BASIC_TAGS_DIR: LazyLock<PathBuf> = LazyLock::new(|| ["tests", "files", "basic_tags"].iter().collect());
    static EXTENDED_TAGS_DIR: LazyLock<PathBuf> =
        LazyLock::new(|| ["tests", "files", "extended_tags"].iter().collect());

    #[test]
    fn test_no_tags() {
        run_test_on_files(&NO_TAGS_DIR, |temp_file| {
            let track = Track::try_from_path(&temp_file).expect("Failed to create Track for temp file");
            let tags = utils::read_tags(&track, true).expect("Tags should be present");
            assert!(tags.artist().is_none());
            assert!(tags.title().is_none());
            fs::remove_file(temp_file).expect("Failed to remove temp file");
        });
    }

    #[test]
    fn test_basic_tags() {
        run_test_on_files(&BASIC_TAGS_DIR, |temp_file| {
            let track = Track::try_from_path(&temp_file).expect("Failed to create Track for temp file");
            let tags = utils::read_tags(&track, true).expect("Tags should be present");
            assert!(!tags.artist().unwrap().is_empty());
            assert!(!tags.title().unwrap().is_empty());
            fs::remove_file(temp_file).expect("Failed to remove temp file");
        });
    }

    #[test]
    fn test_extended_tags() {
        run_test_on_files(&EXTENDED_TAGS_DIR, |temp_file| {
            let track = Track::try_from_path(&temp_file).expect("Failed to create Track for temp file");
            let tags = utils::read_tags(&track, true).expect("Tags should be present");
            assert!(!tags.artist().unwrap().is_empty());
            assert!(!tags.title().unwrap().is_empty());
            fs::remove_file(temp_file).expect("Failed to remove temp file");
        });
    }

    #[test]
    fn test_rename_no_tags() {
        run_test_on_files(&NO_TAGS_DIR, |temp_file| {
            let mut renamer = TrackRenamer::new_with_config(temp_file, Config::new_for_tests());
            renamer.run().expect("Rename failed");
        });
    }

    #[test]
    fn test_rename_basic_tags() {
        run_test_on_files(&BASIC_TAGS_DIR, |temp_file| {
            let mut renamer = TrackRenamer::new_with_config(temp_file, Config::new_for_tests());
            renamer.run().expect("Rename failed");
        });
    }

    #[test]
    fn test_rename_extended_tags() {
        run_test_on_files(&EXTENDED_TAGS_DIR, |temp_file| {
            let mut renamer = TrackRenamer::new_with_config(temp_file, Config::new_for_tests());
            renamer.run().expect("Rename failed");
        });
    }

    /// Generic test function that takes a function or closure with one `PathBuf` as input argument.
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

    /// Check if this is a hidden file like `.DS_Store` on macOS
    fn not_hidden_file(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map_or(true, |s| !s.starts_with('.'))
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
