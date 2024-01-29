use anyhow::{Context, Result};
use colored::*;
use difference::{Changeset, Difference};
use id3::{Error, ErrorKind, Tag, TagLike};

use std::collections::HashMap;
use std::io;
use walkdir::WalkDir;

use crate::fileformat::FileFormat;
use crate::formatter::Formatter;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::String;

use crate::track::Track;

pub struct Renamer {
    root: PathBuf,
    rename_files: bool,
    sort_files: bool,
    print_only: bool,
    tags_only: bool,
    verbose: bool,
    file_list: Vec<Track>,
    total_tracks: usize,
    num_tags_fixed: usize,
    num_renamed: usize,
    formatter: Formatter,
}

impl Renamer {
    pub fn new(
        path: PathBuf,
        rename_files: bool,
        sort_files: bool,
        print_only: bool,
        tags_only: bool,
        verbose: bool,
    ) -> Renamer {
        Renamer {
            root: path,
            rename_files,
            sort_files,
            print_only,
            tags_only,
            verbose,
            file_list: Vec::new(),
            total_tracks: 0,
            num_tags_fixed: 0,
            num_renamed: 0,
            formatter: Formatter::new(),
        }
    }

    /// Gather and process audio files.
    pub fn run(&mut self) -> Result<()> {
        self.gather_files()?;
        self.process_files()
    }

    /// Get all audio files recursively from the root path.
    pub fn gather_files(&mut self) -> Result<()> {
        println!("Getting audio files from {}", format!("{}", self.root.display()).cyan());
        let mut file_list: Vec<Track> = Vec::new();

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let path = entry.path();
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default()
                .to_lowercase();

            match FileFormat::from_str(&extension) {
                Ok(format) => {
                    if let Ok(track) = Track::new_with_extension(path.to_path_buf(), format) {
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
                    } else {
                        eprintln!("{}", e);
                    }
                }
            }
        }

        if file_list.is_empty() {
            anyhow::bail!("no audio files found!");
        }

        self.total_tracks = file_list.len();

        if self.sort_files {
            file_list.sort();
        }

        self.file_list = file_list;

        if self.verbose && self.file_list.len() < 100 {
            for track in &self.file_list {
                println!("{}", track);
            }

            self.print_extension_counts();
        }

        Ok(())
    }

    /// Format all tracks.
    pub fn process_files(&mut self) -> Result<()> {
        println!("{}", format!("Processing {} tracks...", self.total_tracks).bold());
        if self.print_only {
            println!("{}", "Running in print-only mode".yellow().bold())
        }
        let mut current_path = self.root.clone();
        for (number, track) in self.file_list.iter().enumerate() {
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
                    eprintln!("Missing tags: {}", track.full_path().display());
                    if let Some((a, t)) = Renamer::get_tags_from_filename(&track.name) {
                        artist = a;
                        title = t;
                    }
                }
                (None, Some(t)) => {
                    eprintln!("Missing artist tag: {}", track.full_path().display());
                    if let Some((a, _)) = Renamer::get_tags_from_filename(&track.name) {
                        artist = a;
                    }
                    title = t.to_string();
                    current_tags = format!(" - {}", title);
                }
                (Some(a), None) => {
                    eprintln!("Missing title tag: {}", track.full_path().display());
                    artist = a.to_string();
                    if let Some((_, t)) = Renamer::get_tags_from_filename(&track.name) {
                        title = t;
                    }
                    current_tags = format!("{} - ", artist);
                }
            }

            let (formatted_artist, formatted_title) = self.formatter.format_tags(&artist, &title);
            let formatted_tags = format!("{} - {}", formatted_artist, formatted_title);

            let mut tag_changed = false;
            let mut track_printed = false;
            if current_tags != formatted_tags {
                println!("{}/{}:", number + 1, self.total_tracks);
                track_printed = true;
                println!("{}", "Fix tags:".blue().bold());
                Renamer::show_diff(&current_tags, &formatted_tags);
                self.num_tags_fixed += 1;
                if !self.print_only && Renamer::confirm() {
                    tag.set_artist(formatted_artist.clone());
                    tag.set_title(formatted_title.clone());
                    tag.write_to_path(&track.path, tag.version())
                        .context("Failed to write tags")?;
                    tag_changed = true;
                }
                println!("{}", "-".repeat(formatted_tags.chars().count()));
            }

            if self.tags_only {
                continue;
            }

            // Check file name and rename if necessary

            let (file_artist, file_title) = self.formatter.format_filename(&formatted_artist, &formatted_title);

            let new_file_name = format!("{} - {}.{}", file_artist, file_title, track.extension);
            let new_path = track.root.join(&new_file_name);

            if !new_path.is_file() {
                // Rename files if flag was given or if tags were not changed
                if self.rename_files || !tag_changed {
                    if !track_printed {
                        println!("{}/{}:", number + 1, self.total_tracks);
                    }
                    println!("{}", "Rename file:".yellow().bold());
                    Renamer::show_diff(&track.filename(), &new_file_name);
                    self.num_renamed += 1;
                    if !self.print_only && Renamer::confirm() {
                        fs::rename(&track.full_path(), &new_path)?;
                    }
                    println!("{}", "-".repeat(new_file_name.chars().count()));
                }
            } else if new_path != track.path {
                if !track_printed {
                    println!("{}/{}:", number + 1, self.total_tracks);
                }
                println!("{}", "Duplicate:".red().bold());
                println!("{}", track.path.display());
                println!("{}", new_path.display());
                println!("{}", "-".repeat(new_file_name.chars().count()));
            }
        }

        Ok(())
    }

    /// Count and print the total number of each file extension in the file list.
    fn print_extension_counts(&self) {
        let mut file_format_counts: HashMap<String, usize> = HashMap::new();

        for track in &self.file_list {
            *file_format_counts.entry(track.extension.to_string()).or_insert(0) += 1;
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
        let result = match track.extension {
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
                println!("{}", format!("Failed to read tags for: {}\n{}", track, err).red());
                None
            }
        }
    }

    /// Ask user to confirm action.
    /// Note: everything except `n` is a yes.
    fn confirm() -> bool {
        println!("Proceed (y/n)? ");
        let mut ans = String::new();
        io::stdin().read_line(&mut ans).expect("Failed to read line");
        ans.trim().to_lowercase() != "n"
    }

    fn get_tags_from_filename(filename: &str) -> Option<(String, String)> {
        if !filename.contains(" - ") {
            eprintln!("Can't parse tag data from malformed filename: {filename}");
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
}
