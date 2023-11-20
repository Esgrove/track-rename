use regex::Regex;
use std::path::PathBuf;
use anyhow::{Result, Context};
use walkdir::WalkDir;
use colored::*;
use difference::{Changeset, Difference};
use std::string::String;

use crate::track::Track;

pub struct Renamer {
    root: PathBuf,
    rename_files: bool,
    sort_files: bool,
    verbose: bool,
    file_list: Vec<Track>,
    file_formats: Vec<&'static str>,
    total_tracks: usize,
    common_substitutes: Vec<(&'static str, &'static str)>,
    title_substitutes: Vec<(&'static str, &'static str)>,
    regex_substitutes: Vec<(Regex, &'static str)>,
}

impl Renamer {
    pub fn new(path: PathBuf, rename_files: bool, sort_files: bool, verbose: bool) -> Renamer {
        Renamer {
            root: path,
            rename_files,
            sort_files,
            verbose,
            file_list: Vec::new(),
            file_formats: vec!["mp3", "flac", "aif", "aiff", "m4a", "mp4", "wav"],
            total_tracks: 0,
            common_substitutes: vec![
                (" feat ", " feat. "),
                (" ft. ", " feat. "),
                (" Feat ", " feat. "),
                (" featuring ", " feat. "),
                (" Featuring ", " feat. "),
                ("(feat ", "(feat. "),
                ("(ft. ", "(feat. "),
                ("(Feat ", "(feat. "),
                ("(featuring ", "(feat. "),
                ("(Featuring ", "(feat. "),
                ("!!!", ""),
                ("...", " "),
            ],
            title_substitutes: vec![
                (" (Original Mix)", ""),
                ("DJcity ", ""),
                (" DJcity", ""),
                ("DJCity ", ""),
                (" DJCity", ""),
                ("12\"", "12''"),
                ("Intro - Dirty", "Dirty Intro"),
                ("Intro - Clean", "Clean Intro"),
            ],
            regex_substitutes: vec![
                (Regex::new(r"[\[{]+").unwrap(), "("),
                (Regex::new(r"[]}]+").unwrap(), ")"),
                (Regex::new(r"\s+").unwrap(), " "),
                (Regex::new(r"\s{2,}").unwrap(), " "),
                (Regex::new(r"\.{2,}").unwrap(), "."),
                (Regex::new(r"\(\s*?\)").unwrap(), ""),
                (Regex::new(r"(\S)\(").unwrap(), "$1 ("),
            ],
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.gather_files()?;
        self.process_files()
    }

    pub fn gather_files(&mut self) -> Result<()> {
        println!("Getting audio files from {}", format!("{}", self.root.display()).magenta());
        let mut file_list: Vec<Track> = Vec::new();

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().is_file()
                    && self.file_formats.contains(
                        &e.path()
                            .extension()
                            .unwrap_or_default()
                            .to_string_lossy().as_ref()
                    )
            })
        {
            let file_path = entry.path();
            file_list.push(Track::new(
                file_path
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                file_path
                    .extension()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                file_path.parent().unwrap().to_owned(),
            ));
        }

        if file_list.is_empty() {
            anyhow::bail!("no audio files found!");
        }

        self.total_tracks = file_list.len();
        println!("Found {} tracks", self.total_tracks);

        if self.sort_files {
            file_list.sort();
        }

        self.file_list = file_list;

        Ok(())
    }

    pub fn process_files(&mut self) -> Result<()> {
        println!("Renaming tracks...");
        let mut current_path = self.root.clone();
        for (number, file) in self.file_list.iter().enumerate() {
            if !self.sort_files {
                // Print current directory when iterating in directory order
                if current_path != file.path {
                    current_path = file.path.clone();
                    println!("{}", match current_path.strip_prefix(&self.root) {
                        Ok(relative_path) => format!("{}",relative_path.display()).magenta(),
                        Err(_) => format!("{}",current_path.display()).magenta(),
                    });
                }
            }

            if self.verbose {
                println!("{:>3}: {}", number, file.name);
            }

            // Placeholder for tag processing
            let (artist, title) = self.get_tags(file)?;
            let current_tags = format!("{} - {}", artist, title);
            let (formatted_artist, formatted_title) = self.format_track(&artist, &title);
            let new_tags = format!("{} - {}", formatted_artist, formatted_title);

            let mut tag_changed = false;
            if current_tags != new_tags {
                // Placeholder for check_print, show_diff, confirm, and tag saving
                tag_changed = true;
            }

            // Check file name and rename if necessary
            let forbidden_char_regex = Regex::new("[/:\"*?<>|]+").context("Invalid regex pattern")?;
            let file_artist = forbidden_char_regex.replace_all(&formatted_artist, "").to_string().trim().to_string();
            let file_title = forbidden_char_regex.replace_all(&formatted_title, "").to_string().trim().to_string();
            let new_file_name = format!("{} - {}{}", file_artist, file_title, file.extension);
            let new_path = file.path.join(&new_file_name);

            if !new_path.is_file() {
                // Rename files if flag was given or if tags were not changed
                if self.rename_files || !tag_changed {
                    // Placeholder for check_print, show_diff, confirm, and file renaming
                }
            }
        }

        Ok(())
    }

    // Placeholder implementations for helper methods
    fn get_tags(&self, file: &Track) -> Result<(String, String)> {
        // Placeholder for tag reading logic
        Ok(("Artist".to_string(), "Title".to_string()))
    }

    fn format_track(&self, artist: &str, title: &str) -> (String, String) {
        // Placeholder for format logic
        let mut formatted_artist = artist.to_string();
        let mut formatted_title = title.to_string();

        for (pattern, replacement) in &self.common_substitutes {
            formatted_artist = formatted_artist.replace(pattern, replacement);
        }

        for (pattern, replacement) in &self.common_substitutes {
            formatted_title = formatted_title.replace(pattern, replacement);
        }

        for (pattern, replacement) in &self.title_substitutes {
            formatted_title = formatted_title.replace(pattern, replacement);
        }

        for (regex, replacement) in &self.regex_substitutes {
            formatted_artist = regex.replace_all(&artist, *replacement).to_string();
        }

        for (regex, replacement) in &self.regex_substitutes {
            formatted_title = regex.replace_all(&title, *replacement).to_string();
        }

        (formatted_artist.to_string(), formatted_title.to_string())
    }

    fn show_diff(old: &str, new: &str) {
        let changeset = Changeset::new(old, new, "");

        for diff in changeset.diffs {
            match diff {
                Difference::Same(ref x) => print!("{}", x),
                Difference::Add(ref x) => print!("{}", x.green()),
                Difference::Rem(ref x) => print!("{}", x.red()),
            }
        }

        println!();
    }
}
