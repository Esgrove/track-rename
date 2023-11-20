use regex::Regex;
use std::path::PathBuf;

struct Renamer {
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
    fn new(path: PathBuf, rename_files: bool, sort_files: bool, verbose: bool) -> Renamer {
        Renamer {
            root: path,
            rename_files,
            sort_files,
            verbose,
            file_list: Vec::new(),
            file_formats: vec![".mp3", ".flac", ".aif", ".aiff", ".m4a", ".mp4", ".wav"],
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
                (Regex::new(r"[\]}]+").unwrap(), ")"),
                (Regex::new(r"\s+").unwrap(), " "),
                (Regex::new(r"\s{2,}").unwrap(), " "),
                (Regex::new(r"\.{2,}").unwrap(), "."),
                (Regex::new(r"\(\s*?\)").unwrap(), ""),
                (Regex::new(r"(\S)\(").unwrap(), "$1 ("),
            ],
        }
    }

    pub fn gather_files(&mut self) {
        println!("Getting audio files from {}", self.roo.display());
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
                            .to_string_lossy()
                            .into_owned(),
                    )
            })
        {
            let file_path = entry.path();
            file_list.push(Track {
                name: file_path
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                extension: file_path
                    .extension()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                path: file_path.parent().unwrap().to_owned(),
            });
        }

        if file_list.is_empty() {
            eprintln!("no audio files found!");
            process::exit(1);
        }

        self.total_tracks = file_list.len();
        println!("Found {} tracks", self.total_tracks);

        if self.sort_files {
            file_list.sort();
        }

        self.file_list = file_list;
    }
}
