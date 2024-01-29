use crate::fileformat::FileFormat;
use anyhow::Context;
use std::cmp::Ordering;
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, fmt};

#[derive(Debug)]
pub struct Track {
    pub name: String,
    pub extension: FileFormat,
    pub root: PathBuf,
    pub path: PathBuf,
}

impl Track {
    #![allow(dead_code)]
    pub fn new(path: PathBuf) -> anyhow::Result<Track> {
        let name = path
            .file_stem()
            .context("Failed to get file stem")?
            .to_string_lossy()
            .into_owned();
        let extension = FileFormat::from_str(
            path.extension()
                .context("Failed to get file extension")?
                .to_string_lossy()
                .as_ref(),
        )?;
        let root = path.parent().context("Failed to get file root")?.to_owned();

        Ok(Track {
            name,
            extension,
            root,
            path,
        })
    }

    pub fn new_with_extension(path: PathBuf, extension: FileFormat) -> anyhow::Result<Track> {
        let name = path
            .file_stem()
            .context("Failed to get file stem")?
            .to_string_lossy()
            .into_owned();
        let root = path.parent().context("Failed to get file root")?.to_owned();

        Ok(Track {
            name,
            extension,
            root,
            path,
        })
    }

    pub fn filename(&self) -> String {
        format!("{}.{}", self.name, self.extension)
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

impl fmt::Display for Track {
    // Try to print full filepath relative to current working directory,
    // otherwise fallback to absolute path.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let current_dir = match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return write!(f, "{}/{}.{}", self.root.display(), self.name, self.extension),
        };
        let relative_path = match self.root.strip_prefix(&current_dir) {
            Ok(path) => path,
            Err(_) => &self.root,
        };
        write!(f, "{}/{}.{}", relative_path.display(), self.name, self.extension)
    }
}
