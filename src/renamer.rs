use anyhow::{Context, Result};
use colored::*;
use difference::{Changeset, Difference};
use id3::{Error, ErrorKind, Tag, TagLike};
use walkdir::WalkDir;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::String;

use crate::fileformat::FileFormat;
use crate::formatter::TrackFormatter;
use crate::track::Track;

/// Audio track tag and filename formatting.
pub struct Renamer {
    root: PathBuf,
    force: bool,
    rename_files: bool,
    sort_files: bool,
    print_only: bool,
    tags_only: bool,
    verbose: bool,
    file_list: Vec<Track>,
    total_tracks: usize,
    num_tags_fixed: usize,
    num_renamed: usize,
    num_removed: usize,
    num_duplicates: usize,
    formatter: TrackFormatter,
}

impl Renamer {
    pub fn new(
        path: PathBuf,
        force: bool,
        rename_files: bool,
        sort_files: bool,
        print_only: bool,
        tags_only: bool,
        verbose: bool,
    ) -> Renamer {
        Renamer {
            root: path,
            force,
            rename_files,
            sort_files,
            print_only,
            tags_only,
            verbose,
            file_list: Vec::new(),
            total_tracks: 0,
            num_tags_fixed: 0,
            num_renamed: 0,
            num_removed: 0,
            num_duplicates: 0,
            formatter: TrackFormatter::new(),
        }
    }

    /// Gather and process supported audio files.
    pub fn run(&mut self) -> Result<()> {
        self.gather_files()?;
        self.process_files()?;
        if self.verbose {
            self.print_stats();
        }
        Ok(())
    }

    /// Gather audio files recursively from the root path.
    pub fn gather_files(&mut self) -> Result<()> {
        println!("Getting audio files from {}", format!("{}", self.root.display()).cyan());
        let mut file_list = self.get_tracks_from_root();
        if file_list.is_empty() {
            anyhow::bail!("no supported audio files found");
        }

        self.total_tracks = file_list.len();
        if self.sort_files {
            file_list.sort();
        }

        self.file_list = file_list;
        if self.verbose {
            if self.file_list.len() < 100 {
                for track in &self.file_list {
                    println!("{}", track);
                }
            }
            self.print_extension_counts();
        }

        Ok(())
    }

    /// Find and return a list of audio tracks from the root directory.
    fn get_tracks_from_root(&mut self) -> Vec<Track> {
        let mut file_list: Vec<Track> = Vec::new();
        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let path = entry.path();
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or_default().trim();

            if extension.is_empty() {
                continue;
            }
            match FileFormat::from_str(extension) {
                Ok(format) => {
                    if let Ok(track) = Track::new_with_extension(path.to_path_buf(), extension.to_string(), format) {
                        file_list.push(track);
                    } else {
                        eprintln!("{}", format!("Failed to create Track from: {}", path.display()).red());
                    }
                }
                Err(e) => {
                    if extension == "wav" {
                        println!(
                            "{}",
                            format!("Wav should be converted to aif: {}", path.display()).yellow()
                        );
                    } else if self.verbose {
                        eprintln!("{}", e);
                    }
                }
            }
        }
        file_list
    }

    /// Format all tracks.
    pub fn process_files(&mut self) -> Result<()> {
        println!("{}", format!("Processing {} tracks...", self.total_tracks).bold());
        if self.print_only {
            println!("{}", "Running in print-only mode".yellow().bold())
        }
        let mut current_path = self.root.clone();
        for (number, track) in self.file_list.iter_mut().enumerate() {
            if !self.sort_files {
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

            let mut tag = match Renamer::read_tags(track) {
                Some(tag) => tag,
                None => continue,
            };
            let (artist, title, current_tags) = Self::parse_artist_and_title(&track, &mut tag);
            let (formatted_artist, formatted_title) = self.formatter.format_tags(&artist, &title);
            let formatted_tags = format!("{} - {}", formatted_artist, formatted_title);

            if current_tags != formatted_tags {
                track.show(number + 1, self.total_tracks);
                println!("{}", "Fix tags:".blue().bold());
                Renamer::show_diff(&current_tags, &formatted_tags);
                self.num_tags_fixed += 1;
                if !self.print_only && (self.force || Renamer::confirm()) {
                    tag.set_artist(formatted_artist.clone());
                    tag.set_title(formatted_title.clone());
                    tag.write_to_path(&track.path, id3::Version::Id3v23)
                        .context("Failed to write tags")?;
                    track.tags_updated = true;
                }
                Self::print_divider(&formatted_tags);
            }

            if self.tags_only {
                continue;
            }

            let (file_artist, file_title) = self.formatter.format_filename(&formatted_artist, &formatted_title);
            let new_file_name = format!("{} - {}.{}", file_artist, file_title, track.format);
            let new_path = track.root.join(&new_file_name);

            if !new_path.is_file() {
                // Rename files if flag was given or if tags were not changed
                if self.rename_files || !track.tags_updated {
                    track.show(number + 1, self.total_tracks);
                    println!("{}", "Rename file:".yellow().bold());
                    Renamer::show_diff(&track.filename(), &new_file_name);
                    self.num_renamed += 1;
                    if !self.print_only && (self.force || Renamer::confirm()) {
                        fs::rename(&track.path, &new_path)?;
                    }
                    Self::print_divider(&new_file_name);
                }
            } else if new_path != track.path {
                track.show(number + 1, self.total_tracks);
                println!("{}", "Duplicate:".red().bold());
                println!("{}", track.path.display());
                println!("{}", new_path.display());
                Self::print_divider(&new_file_name);
                self.num_duplicates += 1;
            }

            // TODO: handle duplicates and same track in different file formats
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

    /// Try to read tags from path.
    /// Will return empty tags when there are no tags.
    fn read_tags(track: &Track) -> Option<Tag> {
        let result = match track.format {
            FileFormat::Aif => Tag::read_from_aiff_path(&track.path),
            FileFormat::Mp3 => Tag::read_from_path(&track.path),
        };

        match result {
            Ok(tag) => Some(tag),
            Err(Error {
                kind: ErrorKind::NoTag, ..
            }) => {
                println!("{}", format!("No tags: {}", track).yellow());
                Some(Tag::new())
            }
            Err(err) => {
                eprintln!("{}", format!("Failed to read tags for: {}\n{}", track, err).red());
                None
            }
        }
    }

    /// Ask user to confirm action.
    /// Note: everything except `n` is a yes.
    fn confirm() -> bool {
        print!("Proceed (y/n)? ");
        io::stdout().flush().expect("Failed to flush stdout");
        let mut ans = String::new();
        io::stdin().read_line(&mut ans).expect("Failed to read line");
        ans.trim().to_lowercase() != "n"
    }

    /// Try to read artist and title from tags.
    /// Fallback to parsing them from filename if tags are empty.
    fn parse_artist_and_title(track: &&mut Track, tag: &mut Tag) -> (String, String, String) {
        let mut artist = String::new();
        let mut title = String::new();
        let mut current_tags = " - ".to_string();

        match (tag.artist(), tag.title()) {
            (Some(a), Some(t)) => {
                artist = a.to_string();
                title = t.to_string();
                current_tags = format!("{} - {}", artist, title);
            }
            (None, None) => {
                eprintln!("{}", format!("Missing tags: {}", track.path.display()).yellow());
                if let Some((a, t)) = Renamer::get_tags_from_filename(&track.name) {
                    artist = a;
                    title = t;
                }
            }
            (None, Some(t)) => {
                eprintln!("{}", format!("Missing artist tag: {}", track.path.display()).yellow());
                if let Some((a, _)) = Renamer::get_tags_from_filename(&track.name) {
                    artist = a;
                }
                title = t.to_string();
                current_tags = format!(" - {}", title);
            }
            (Some(a), None) => {
                eprintln!("{}", format!("Missing title tag: {}", track.path.display()).yellow());
                artist = a.to_string();
                if let Some((_, t)) = Renamer::get_tags_from_filename(&track.name) {
                    title = t;
                }
                current_tags = format!("{} - ", artist);
            }
        }
        (artist, title, current_tags)
    }

    /// Convert filename to artist and title tags.
    /// Expects filename to be in format 'artist - title'.
    fn get_tags_from_filename(filename: &str) -> Option<(String, String)> {
        if !filename.contains(" - ") {
            eprintln!(
                "{}",
                format!("Can't parse tag data from malformed filename: {filename}").red()
            );
            return None;
        }

        let parts: Vec<&str> = filename.splitn(2, " - ").collect();
        if parts.len() == 2 {
            let artist = parts[0].to_string();
            let title = parts[1].to_string();
            Some((artist, title))
        } else {
            None
        }
    }

    /// Print a stacked diff of the changes.
    fn show_diff(old: &str, new: &str) {
        let changeset = Changeset::new(old, new, "");
        let mut old_diff = String::new();
        let mut new_diff = String::new();

        for diff in changeset.diffs {
            match diff {
                Difference::Same(ref x) => {
                    old_diff.push_str(x);
                    new_diff.push_str(x);
                }
                Difference::Add(ref x) => {
                    if x.chars().all(char::is_whitespace) {
                        new_diff.push_str(&x.to_string().on_green().to_string());
                    } else {
                        new_diff.push_str(&x.to_string().green().to_string());
                    }
                }
                Difference::Rem(ref x) => {
                    if x.chars().all(char::is_whitespace) {
                        old_diff.push_str(&x.to_string().on_red().to_string());
                    } else {
                        old_diff.push_str(&x.to_string().red().to_string());
                    }
                }
            }
        }

        println!("{}", old_diff);
        println!("{}", new_diff);
    }

    /// Print number of changes made.
    fn print_stats(&self) {
        println!("{}", "Finished".green());
        println!("Tags:      {}", self.num_tags_fixed);
        println!("Rename:    {}", self.num_renamed);
        println!("Delete:    {}", self.num_removed);
        println!("Duplicate: {}", self.num_duplicates);
    }

    fn print_divider(text: &str) {
        println!("{}", "-".repeat(text.chars().count()));
    }
}
