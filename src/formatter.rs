use regex::Regex;

pub struct Formatter {
    common_substitutes: Vec<(&'static str, &'static str)>,
    title_substitutes: Vec<(&'static str, &'static str)>,
    regex_substitutes: Vec<(Regex, &'static str)>,
    filename_regex_substitutes: Vec<(Regex, &'static str)>,
}

impl Formatter {
    pub fn new() -> Formatter {
        Formatter {
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
                (") - (", ""),
                (" - (", " ("),
                ("(- ", "("),
                ("( - ", "("),
                (" -)", " )"),
                (" - ) ", ")"),
                ("!!!", ""),
                ("...", " "),
                ("..", " "),
                (" ***", ""),
                (" **", ""),
                (" * ", ""),
            ],
            title_substitutes: vec![
                (" (Original Mix)", ""),
                (" DJcity", ""),
                (" DJCity", ""),
                ("(DJcity - ", "("),
                ("DJcity ", ""),
                ("DJCity ", ""),
                ("12\"", "12''"),
                ("Intro - Dirty", "Dirty Intro"),
                ("Intro - Clean", "Clean Intro"),
                ("Acap - DIY", "Acapella DIY"),
                ("(Acap)", "(Acapella)"),
                ("Acap ", "Acapella "),
                ("(Inst)", "(Instrumental)"),
                (" 12 Inch ", " 12'' "),
                ("(12 Inch ", "(12'' "),
                (" 12in ", " 12'' "),
                ("(12in ", "(12'' "),
                ("(7in ", "(7'' "),
                (" 7in ", " 7'' "),
                ("Intro/Outro", "Intro-Outro"),
                (" In/Out", " Intro-Outro"),
                ("In/Out ", "Intro-Outro "),
                ("Aca In/Aca Out", "Acapella In-Out"),
                ("Intro/Outro", "Intro"),
                ("Intro-Outro", "Intro"),
                ("In+Out", "In-Out"),
                ("In+out", "In-Out"),
            ],
            regex_substitutes: vec![
                (Regex::new(r"[\[{]+").unwrap(), "("),
                (Regex::new(r"[\]}]+").unwrap(), ")"),
                (Regex::new(r"\s+").unwrap(), " "),
                (Regex::new(r"\s{2,}").unwrap(), " "),
                (Regex::new(r"\.{2,}").unwrap(), "."),
                (Regex::new(r"\(\s*?\)").unwrap(), ""),
                (Regex::new(r"(\S)\(").unwrap(), "$1 ("),
            ],
            filename_regex_substitutes: vec![
                (Regex::new("\"").unwrap(), "''"),
                (Regex::new(r"[\\/<>|!:\*\?]+").unwrap(), "-"),
                (Regex::new(r"\s+").unwrap(), " "),
            ],
        }
    }

    /// Return formatted artist and title string.
    pub(crate) fn format_tags(&self, artist: &str, title: &str) -> (String, String) {
        let mut formatted_artist = artist.to_string();
        let mut formatted_title = title.to_string();

        for (pattern, replacement) in &self.common_substitutes {
            formatted_artist = formatted_artist.replace(pattern, replacement);
            formatted_title = formatted_title.replace(pattern, replacement);
        }

        for (pattern, replacement) in &self.title_substitutes {
            formatted_title = formatted_title.replace(pattern, replacement);
        }

        for (regex, replacement) in &self.regex_substitutes {
            formatted_artist = regex.replace_all(artist, *replacement).to_string();
            formatted_title = regex.replace_all(title, *replacement).to_string();
        }

        (formatted_artist.trim().to_string(), formatted_title.trim().to_string())
    }

    pub(crate) fn format_filename(&self, artist: &str, title: &str) -> (String, String) {
        let mut formatted_artist = artist.to_string();
        let mut formatted_title = title.to_string();

        for (regex, replacement) in &self.filename_regex_substitutes {
            formatted_artist = regex.replace_all(artist, *replacement).to_string();
            formatted_title = regex.replace_all(title, *replacement).to_string();
        }

        (formatted_artist.trim().to_string(), formatted_title.trim().to_string())
    }
}
