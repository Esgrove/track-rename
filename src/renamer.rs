use anyhow::{Context, Result};
use id3::{Tag, TagLike};
use colored::*;
use difference::{Changeset, Difference};
use regex::Regex;
use std::io;
use walkdir::WalkDir;

use std::fs;
use std::path::PathBuf;
use std::string::String;
use crate::formatter::Formatter;

use crate::track::Track;

pub struct Renamer {
    root: PathBuf,
    rename_files: bool,
    sort_files: bool,
    print_only: bool,
    tags_only: bool,
    verbose: bool,
    file_list: Vec<Track>,
    file_formats: [&'static str; 3],
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
            file_formats: ["mp3", "aif", "aiff"], // "flac", "m4a", "mp4"
            total_tracks: 0,
            num_tags_fixed: 0,
            num_renamed: 0,
            formatter: Formatter::new()
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

        for entry in WalkDir::new(&self.root).into_iter().filter_map(|e| e.ok()).filter(|e| {
            e.path().is_file()
                && self
                    .file_formats
                    .contains(&e.path().extension().unwrap_or_default().to_string_lossy().as_ref())
        }) {
            file_list.push(Track::new_from_path(entry.path().to_path_buf()));
        }

        if file_list.is_empty() {
            anyhow::bail!("no audio files found!");
        }

        self.total_tracks = file_list.len();

        if self.sort_files {
            file_list.sort();
        }

        self.file_list = file_list;

        Ok(())
    }

    /// Format all tracks.
    pub fn process_files(&mut self) -> Result<()> {
        println!("Formatting {} tracks...", self.total_tracks);
        let mut current_path = self.root.clone();
        for (number, file) in self.file_list.iter().enumerate() {
            if !self.sort_files {
                // Print current directory when iterating in directory order
                if current_path != file.root {
                    current_path = file.root.clone();
                    println!(
                        "{}",
                        match current_path.strip_prefix(&self.root) {
                            Ok(relative_path) => format!("{}", relative_path.display()).magenta(),
                            Err(_) => format!("{}", current_path.display()).magenta(),
                        }
                    );
                }
            }

            if self.verbose {
                println!("{:>3}: {}", number, file.name);
            }

            let mut tag = Tag::read_from_path(file.full_path())?;
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
                    eprintln!("Missing tags: {}", file.full_path().display());
                    if let Some((a, t)) = Renamer::get_tags_from_filename(&file.name) {
                        artist = a;
                        title = t;
                    }
                }
                (None, Some(t)) => {
                    eprintln!("Missing artist tag: {}", file.full_path().display());
                    if let Some((a, _)) = Renamer::get_tags_from_filename(&file.name) {
                        artist = a;
                    }
                    title = t.to_string();
                    current_tags = format!(" - {}", title);
                }
                (Some(a), None) => {
                    eprintln!("Missing title tag: {}", file.full_path().display());
                    artist = a.to_string();
                    if let Some((_, t)) = Renamer::get_tags_from_filename(&file.name) {
                        title = t;
                    }
                    current_tags = format!("{} - ", artist);
                }
            }

            let (formatted_artist, formatted_title) = self.formatter.format_track(&artist, &title);
            let new_tags = format!("{} - {}", formatted_artist, formatted_title);

            let mut tag_changed = false;
            let mut track_printed = false;
            if current_tags != new_tags {
                println!("{}/{}:", number, self.total_tracks);
                track_printed = true;
                println!("{}", "Fix tags:".blue().bold());
                Renamer::show_diff(&current_tags, &new_tags);
                self.num_tags_fixed += 1;
                if !self.print_only && Renamer::confirm() {
                    tag.set_artist(formatted_artist.clone());
                    tag.set_title(formatted_title.clone());
                    tag.write_to_path(file.full_path(), tag.version())
                        .context("Failed to write tags")?;
                    tag_changed = true;
                }
                println!("{}", "-".repeat(new_tags.len()));
            }

            if self.tags_only {
                continue;
            }

            // Check file name and rename if necessary
            let forbidden_char_regex = Regex::new("[/:\"*?<>|]+").context("Invalid regex pattern")?;
            let file_artist = forbidden_char_regex
                .replace_all(&formatted_artist, "")
                .to_string()
                .trim()
                .to_string();
            let file_title = forbidden_char_regex
                .replace_all(&formatted_title, "")
                .to_string()
                .trim()
                .to_string();

            let new_file_name = format!("{} - {}{}", file_artist, file_title, file.extension);
            let new_path = file.root.join(&new_file_name);

            if !new_path.is_file() {
                // Rename files if flag was given or if tags were not changed
                if self.rename_files || !tag_changed {
                    if !track_printed {
                        println!("{}/{}:", number, self.total_tracks);
                    }
                    println!("{}", "Rename file:".yellow().bold());
                    Renamer::show_diff(&file.filename(), &new_file_name);
                    self.num_renamed += 1;
                    if !self.print_only && Renamer::confirm() {
                        fs::rename(&file.full_path(), &new_path)?;
                    }
                    println!("{}", "-".repeat(new_file_name.len()));
                }
            }
        }

        Ok(())
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
        let mut old_string = String::new();
        let mut new_string = String::new();

        for diff in changeset.diffs {
            match diff {
                Difference::Same(ref x) => {
                    old_string.push_str(x);
                    new_string.push_str(x);
                }
                Difference::Add(ref x) => new_string.push_str(&x.green().to_string()),
                Difference::Rem(ref x) => old_string.push_str(&x.red().to_string()),
            }
        }

        println!("{}", old_string);
        println!("{}", new_string);
    }
}
