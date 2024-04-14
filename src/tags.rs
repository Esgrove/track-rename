use colored::Colorize;

use crate::utils;

#[derive(Debug, Default, Clone)]
pub struct Tags {
    pub current_artist: String,
    pub current_title: String,
    pub current_album: String,
    pub current_genre: String,
    pub formatted_artist: String,
    pub formatted_title: String,
    pub formatted_album: String,
    pub formatted_genre: String,
}

impl Tags {
    pub fn new(artist: String, title: String, album: String, genre: String) -> Tags {
        Tags {
            current_artist: artist,
            current_title: title,
            current_album: album,
            current_genre: genre,
            ..Default::default()
        }
    }

    pub fn current_name(&self) -> String {
        format!("{} - {}", self.current_artist, self.current_title)
    }

    pub fn formatted_name(&self) -> String {
        format!("{} - {}", self.formatted_artist, self.formatted_title)
    }

    /// Returns true if any of the formatted tag fields differ from their current value.
    pub fn changed(&self) -> bool {
        self.current_artist != self.formatted_artist
            || self.current_title != self.formatted_title
            || self.current_album != self.formatted_album
            || self.current_genre != self.formatted_genre
    }

    pub fn print_diff(&self) {
        utils::show_diff(&self.current_name(), &self.formatted_name());
        if self.current_album != self.formatted_album {
            println!("{}: {} -> {}", "album".bold(), self.current_album, self.formatted_album);
        }
        if self.current_genre != self.formatted_genre {
            println!("{}: {} -> {}", "genre".bold(), self.current_genre, self.formatted_genre);
        }
    }
}
