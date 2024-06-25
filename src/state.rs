use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

const STATE_FILE_DIR: &str = "track-rename";
#[cfg(not(test))]
const STATE_FILE_NAME: &str = "state.json";
#[cfg(test)]
const STATE_FILE_NAME: &str = "test_state.json";

#[derive(Debug, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TrackMetadata {
    pub modified: u64,
    pub hash: String,
    pub version: String,
}

pub type State = HashMap<PathBuf, TrackMetadata>;

pub fn load_state() -> State {
    read_state().into_par_iter().filter(|(path, _)| path.exists()).collect()
}

pub fn save_state(state: &State) -> anyhow::Result<()> {
    let path = state_path()?;
    fs::create_dir_all(path.parent().unwrap())?;
    let data = serde_json::to_string(state)?;
    fs::write(path, data)?;
    Ok(())
}

fn get_state_path() -> anyhow::Result<PathBuf> {
    let state_path = state_path()?;
    match state_path.exists() {
        true => Ok(state_path),
        false => Err(anyhow!("State file not found: {}", state_path.display())),
    }
}

fn state_path() -> anyhow::Result<PathBuf> {
    let data_dir = dirs::data_dir().context("Failed to get data directory path")?;
    Ok(data_dir.join(STATE_FILE_DIR).join(STATE_FILE_NAME))
}

fn read_state() -> State {
    get_state_path()
        .and_then(|file_path| fs::read_to_string(file_path).map_err(anyhow::Error::from))
        .and_then(|contents| serde_json::from_str(&contents).map_err(anyhow::Error::from))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_env() -> PathBuf {
        let data_dir = dirs::data_dir().expect("Failed to get data directory path");
        let state_path = data_dir.join(STATE_FILE_DIR).join(STATE_FILE_NAME);

        // Ensure the directory exists
        fs::create_dir_all(state_path.parent().unwrap()).unwrap();

        // Clean up any existing test file
        if state_path.exists() {
            fs::remove_file(&state_path).unwrap();
        }

        state_path
    }

    #[test]
    fn test_state() {
        // Everything is tested in a single test case since otherwise tests can fail as they all touch the same file.
        let state_path = setup_test_env();
        let mut state = State::default();
        let test_path: PathBuf = ["tests", "files", "basic_tags", "Basic Tags - Song - 16-44.aif"]
            .iter()
            .collect();
        state.insert(
            test_path,
            TrackMetadata {
                modified: 123456789,
                hash: "test_hash".to_string(),
                version: "test_version".to_string(),
            },
        );

        // Save the state
        save_state(&state).expect("Failed to save state");

        // Verify the file contents
        let saved_data = fs::read_to_string(state_path).expect("Failed to read state file");
        let loaded_state: State = serde_json::from_str(&saved_data).expect("Failed to deserialize state");

        assert_eq!(state, loaded_state);

        setup_test_env();
        let state = load_state();
        assert!(state.is_empty());

        let mut state = State::default();
        let test_path: PathBuf = ["tests", "files", "basic_tags", "Basic Tags - Song - 16-44.aif"]
            .iter()
            .collect();
        let test_data = TrackMetadata {
            modified: 123456789,
            hash: "test_hash".to_string(),
            version: "test_version".to_string(),
        };
        state.insert(test_path.clone(), test_data.clone());
        save_state(&state).expect("Failed to save state");

        let loaded_state = load_state();
        assert_eq!(state, loaded_state);
    }
}
