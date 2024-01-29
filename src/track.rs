use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Track {
    pub name: String,
    pub extension: String,
    pub root: PathBuf,
}

impl Track {
    pub fn new(name: String, mut extension: String, path: PathBuf) -> Track {
        if !extension.starts_with('.') {
            extension.insert(0, '.');
        }

        Track {
            name,
            extension,
            root: path,
        }
    }

    pub fn new_from_path(path: PathBuf) -> Track {
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let mut extension = path.extension().unwrap().to_string_lossy().into_owned();
        let root = path.parent().unwrap().to_owned();

        if !extension.starts_with('.') {
            extension.insert(0, '.');
        }

        Track { name, extension, root }
    }

    pub fn filename(&self) -> String {
        format!("{}{}", self.name, self.extension)
    }

    pub fn full_path(&self) -> PathBuf {
        self.root.join(self.filename())
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
