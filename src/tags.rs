use colored::Colorize;
use id3::{Tag, TagLike};

use crate::track::Track;
use crate::utils;

#[derive(Debug, Default, Clone)]
pub struct TrackTags {
    pub current_artist: String,
    pub current_title: String,
    pub current_album: String,
    pub current_genre: String,
    pub current_name: String,
    pub formatted_name: String,
    pub formatted_artist: String,
    pub formatted_title: String,
    pub formatted_album: String,
    pub formatted_genre: String,
    pub update_needed: bool,
}

impl TrackTags {
    #[must_use]
    pub fn new(name: String, artist: String, title: String, album: String, genre: String) -> Self {
        Self {
            current_name: name,
            current_artist: artist,
            current_title: title,
            current_album: album,
            current_genre: genre,
            ..Default::default()
        }
    }

    /// Try to read tags such as artist and title from tags.
    ///
    /// Fallback to parsing them from filename if tags are empty.
    #[must_use]
    pub fn parse_tag_data(track: &Track, tag: &Tag) -> Self {
        let mut artist = String::new();
        let mut title = String::new();

        // Tags might be formatted correctly but a missing field needs to be written.
        // Store formatted name before parsing missing fields from filename.
        let current_name: String;

        match (tag.artist(), tag.title()) {
            (Some(a), Some(t)) => {
                artist = utils::normalize_str(a);
                title = utils::normalize_str(t);
                current_name = format!("{artist} - {title}");
            }
            (None, None) => {
                eprintln!("\n{}", format!("Missing tags: {}", track.path.display()).yellow());
                current_name = format!("{artist} - {title}");
                if let Some((a, t)) = utils::get_tags_from_filename(&track.name) {
                    artist = a;
                    title = t;
                }
            }
            (None, Some(t)) => {
                eprintln!("\n{}", format!("Missing artist tag: {}", track.path.display()).yellow());
                title = utils::normalize_str(t);
                current_name = format!("{artist} - {title}");
                if let Some((a, _)) = utils::get_tags_from_filename(&track.name) {
                    artist = a;
                }
            }
            (Some(a), None) => {
                eprintln!("\n{}", format!("Missing title tag: {}", track.path.display()).yellow());
                artist = utils::normalize_str(a);
                current_name = format!("{artist} - {title}");
                if let Some((_, t)) = utils::get_tags_from_filename(&track.name) {
                    title = t;
                }
            }
        }
        let album = utils::normalize_str(tag.album().unwrap_or_default());
        let genre = utils::normalize_str(tag.genre_parsed().unwrap_or_default().as_ref());
        Self::new(current_name, artist, title, album, genre)
    }

    /// Returns true if any of the formatted tag fields differ from their current value,
    /// or artist and/or title tag is missing.
    #[must_use]
    pub fn changed(&self) -> bool {
        self.current_name != self.formatted_name
            || self.current_artist != self.formatted_artist
            || self.current_title != self.formatted_title
            || self.current_album != self.formatted_album
            || self.current_genre != self.formatted_genre
    }

    /// Print coloured diff for changes in tags.
    ///
    /// Prints nothing if there are no changes.
    pub fn show_diff(&self) {
        if self.current_name != self.formatted_name {
            utils::print_stacked_diff(&self.current_name, &self.formatted_name);
        }
        if self.current_album != self.formatted_album {
            print!("{}: ", "Album".bold());
            utils::print_diff(&self.current_album, &self.formatted_album);
        }
        if self.current_genre != self.formatted_genre {
            print!("{}: ", "Genre".bold());
            utils::print_diff(&self.current_genre, &self.formatted_genre);
        }
    }
}
