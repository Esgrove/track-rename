use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use dashmap::DashMap;
use rayon::prelude::*;

use crate::track::TrackMetadata;
use crate::track::VERSION;

const STATE_FILE_DIR: &str = "track-rename";
#[cfg(not(test))]
const STATE_FILE_NAME: &str = "state.json";
#[cfg(test)]
const STATE_FILE_NAME: &str = "test_state.json";

/// Maintain a map of processed tracks.
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
        let path = Self::state_path()?;
        fs::create_dir_all(path.parent().expect("Failed to get state parent path"))?;
        let data = serde_json::to_string(&self.inner)?;
        fs::write(path, data)?;
        Ok(())
    }

    /// Insert a new entry into the state.
    ///
    /// Returns the old value associated with the same key if there was one.
    #[allow(clippy::must_use_candidate)]
    pub fn insert(&self, path: PathBuf, metadata: TrackMetadata) -> Option<TrackMetadata> {
        self.inner.insert(path, metadata)
    }

    /// Get an entry in the state.
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

    #[must_use]
    pub fn clean(&self) -> usize {
        let start_count = self.inner.len();

        self.inner.retain(|key, value| value.version == VERSION || key.exists());

        let end_count = self.inner.len();

        start_count.saturating_sub(end_count)
    }

    /// Private helper to get the path to the state file.
    fn state_path() -> anyhow::Result<PathBuf> {
        let data_dir = dirs::data_dir().context("Failed to get data directory path")?;
        Ok(data_dir.join(STATE_FILE_DIR).join(STATE_FILE_NAME))
    }

    fn read_state() -> DashMap<PathBuf, TrackMetadata> {
        Self::get_state_path()
            .and_then(|file_path| fs::read_to_string(file_path).map_err(anyhow::Error::from))
            .and_then(|contents| serde_json::from_str(&contents).map_err(anyhow::Error::from))
            .unwrap_or_default()
    }

    fn get_state_path() -> anyhow::Result<PathBuf> {
        let state_path = Self::state_path()?;
        if state_path.exists() {
            Ok(state_path)
        } else {
            Err(anyhow!("State file not found: {}", state_path.display()))
        }
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
        let state_path = setup_test_env();

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
