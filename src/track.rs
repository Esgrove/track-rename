use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Track {
    pub name: String,
    pub extension: String,
    pub path: PathBuf,
}

impl Track {
    pub fn new(name: String, mut extension: String, path: PathBuf) -> Track {
        if !extension.starts_with(".") {
            extension.insert(0, '.');
        }

        Track {
            name,
            extension,
            path,
        }
    }

    fn filename(&self) -> String {
        format!("{}{}", self.name, self.extension)
    }

    fn full_path(&self) -> PathBuf {
        self.path.join(&self.filename())
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Track {}

impl PartialOrd for Track {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Track {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}
