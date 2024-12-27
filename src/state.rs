use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use dashmap::DashMap;
use rayon::prelude::*;

use crate::track::TrackMetadata;
use crate::track::VERSION;

const STATE_FILE_DIR: &str = "track-rename";
#[cfg(not(test))]
const STATE_FILE_NAME: &str = "state.json";
#[cfg(test)]
const STATE_FILE_NAME: &str = "test_state.json";

static STATE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::data_dir()
        .expect("Failed to get data directory path")
        .join(STATE_FILE_DIR)
        .join(STATE_FILE_NAME)
});

/// Maintain a map of processed tracks between program runs.
///
/// Enables skipping tracks that have already been processed with the same program version,
/// in case they have not been modified since then.
#[derive(Debug, Default)]
pub struct State {
    inner: DashMap<PathBuf, TrackMetadata>,
}

impl State {
    /// Load the state from the saved file, filtering out non-existent paths.
    #[must_use]
    pub fn load() -> Self {
        let inner: DashMap<PathBuf, TrackMetadata> = Self::read_state()
            .into_par_iter()
            .filter(|(path, _)| path.exists())
            .collect();

        Self { inner }
    }

    /// Save the current state to a file.
    pub fn save(&self) -> anyhow::Result<()> {
        let parent_dir = Self::state_path().parent().expect("Failed to get state parent path");
        fs::create_dir_all(parent_dir)?;
        let data = serde_json::to_string(&self.inner)?;
        fs::write(Self::state_path(), data)?;
        Ok(())
    }

    /// Insert a new entry into the state.
    ///
    /// Returns the old value associated with the same key if there was one.
    #[allow(clippy::must_use_candidate)]
    pub fn insert(&self, path: PathBuf, metadata: TrackMetadata) -> Option<TrackMetadata> {
        self.inner.insert(path, metadata)
    }

    #[must_use]
    pub fn get(&self, path: &PathBuf) -> Option<TrackMetadata> {
        self.inner.get(path).map(|entry| entry.clone())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove outdated entries from state.
    ///
    /// Removes entries that do not exist on disk anymore or the version does not match current version.
    /// Returns the number of elements removed.
    #[allow(clippy::must_use_candidate)]
    pub fn clean(&self) -> usize {
        let start_count = self.inner.len();

        self.inner.retain(|key, value| key.exists() && value.version == VERSION);

        let end_count = self.inner.len();

        start_count.saturating_sub(end_count)
    }

    fn read_state() -> DashMap<PathBuf, TrackMetadata> {
        Self::get_state_path().map_or_else(DashMap::new, |file_path| match fs::read_to_string(file_path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(map) => map,
                Err(err) => {
                    eprintln!("Failed to parse state file: {err}");
                    DashMap::new()
                }
            },
            Err(err) => {
                eprintln!("Failed to read state file: {err}");
                DashMap::new()
            }
        })
    }

    fn get_state_path() -> Option<&'static Path> {
        Self::state_path().exists().then(Self::state_path)
    }

    fn state_path() -> &'static Path {
        STATE_PATH.as_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_env() -> PathBuf {
        let data_dir = dirs::data_dir().expect("Failed to get data directory path");
        let state_path = data_dir.join(STATE_FILE_DIR).join(STATE_FILE_NAME);

        fs::create_dir_all(state_path.parent().unwrap()).unwrap();

        if state_path.exists() {
            fs::remove_file(&state_path).unwrap();
        }

        state_path
    }

    #[test]
    fn test_state() {
        // Everything is tested in a single test case since otherwise tests can fail as they all touch the same file.
        setup_test_env();

        let test_path: PathBuf = ["tests", "files", "basic_tags", "Basic Tags - Song - 16-44.aif"]
            .iter()
            .collect();

        let state = State::default();
        state.insert(
            test_path.clone(),
            TrackMetadata {
                modified: 123_456_789,
                version: "test_version".to_string(),
            },
        );

        state.save().expect("Failed to save state");

        let loaded_state = State::load();

        // DashMap does not have PartialEq so need to compare values manually
        assert_eq!(
            state.get(&test_path).unwrap().version,
            loaded_state.get(&test_path).unwrap().version
        );
        assert_eq!(
            state.get(&test_path).unwrap().modified,
            loaded_state.get(&test_path).unwrap().modified
        );

        setup_test_env();
        let empty_state = State::load();
        assert!(empty_state.is_empty());

        let test_data = TrackMetadata {
            modified: 1_716_068_288,
            version: "1.0.0".to_string(),
        };

        let state = State::default();
        state.insert(test_path.clone(), test_data);
        state.save().expect("Failed to save state");

        let loaded_state = State::load();
        assert_eq!(
            state.get(&test_path).unwrap().version,
            loaded_state.get(&test_path).unwrap().version
        );
        assert_eq!(
            state.get(&test_path).unwrap().modified,
            loaded_state.get(&test_path).unwrap().modified
        );
    }
}
