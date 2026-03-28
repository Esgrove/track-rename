use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{Context, Result};
use rusqlite::{Connection, params};

use crate::track::TrackMetadata;
use crate::track::VERSION;
use crate::utils;

const STATE_DIR: &str = "track-rename";
#[cfg(not(test))]
const STATE_DB_NAME: &str = "state.db";
#[cfg(test)]
const STATE_DB_NAME: &str = "test_state.db";

static DB_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::data_dir()
        .expect("Failed to get data directory path")
        .join(STATE_DIR)
        .join(STATE_DB_NAME)
});

/// Maintain a database of processed tracks between program runs.
///
/// Enables skipping tracks that have already been processed with the same program version,
/// in case they have not been modified since then.
pub struct State {
    connection: Connection,
    db_path: PathBuf,
}

impl State {
    /// Open (or create) the default on-disk database and initialize the schema.
    pub fn open() -> Result<Self> {
        let db_path = DB_PATH.clone();
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create database directory: {}", parent.display()))?;
        }
        let connection =
            Connection::open(&db_path).with_context(|| format!("Failed to open database: {}", db_path.display()))?;
        Self::initialize(connection, db_path)
    }

    /// Open an in-memory database and initialize the schema.
    /// Useful for tests and as the `Default` implementation.
    pub fn open_in_memory() -> Result<Self> {
        let connection = Connection::open_in_memory().context("Failed to open in-memory database")?;
        Self::initialize(connection, PathBuf::from(":memory:"))
    }

    /// Return the database file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.db_path.as_ref()
    }

    /// Insert or update a track entry.
    ///
    /// Returns `Ok(true)` if an existing entry was updated, `Ok(false)` if a new entry was added.
    pub fn insert(&self, path: &Path, metadata: &TrackMetadata) -> Result<bool> {
        let path_str = utils::path_to_string(path);
        let modified = u64_to_i64(metadata.modified);

        let existed = self.path_exists(&path_str)?;

        self.connection
            .prepare_cached(
                "INSERT INTO tracks (path, modified, version) VALUES (?1, ?2, ?3)
                 ON CONFLICT(path) DO UPDATE SET modified = excluded.modified, version = excluded.version",
            )
            .context("Failed to prepare insert statement")?
            .execute(params![path_str, modified, &metadata.version])
            .context("Failed to insert track")?;

        Ok(existed)
    }

    /// Batch insert or update track entries within a transaction.
    ///
    /// Returns `(added_count, updated_count)`.
    pub fn batch_insert(&self, entries: &[(&Path, &TrackMetadata)]) -> Result<(usize, usize)> {
        self.connection
            .execute_batch("BEGIN")
            .context("Failed to begin batch insert transaction")?;

        let result = (|| -> Result<(usize, usize)> {
            let mut added: usize = 0;
            let mut updated: usize = 0;

            for (path, metadata) in entries {
                let path_str = utils::path_to_string(path);
                let modified = u64_to_i64(metadata.modified);
                let existed = self.path_exists(&path_str)?;

                self.connection
                    .prepare_cached(
                        "INSERT INTO tracks (path, modified, version) VALUES (?1, ?2, ?3)
                         ON CONFLICT(path) DO UPDATE SET modified = excluded.modified, version = excluded.version",
                    )
                    .context("Failed to prepare batch insert statement")?
                    .execute(params![path_str, modified, &metadata.version])
                    .context("Failed to execute batch insert")?;

                if existed {
                    updated += 1;
                } else {
                    added += 1;
                }
            }

            Ok((added, updated))
        })();

        match &result {
            Ok(_) => {
                self.connection
                    .execute_batch("COMMIT")
                    .context("Failed to commit batch insert transaction")?;
            }
            Err(_) => {
                let _ = self.connection.execute_batch("ROLLBACK");
            }
        }

        result
    }

    /// Look up a track by path.
    pub fn get(&self, path: &Path) -> Result<Option<TrackMetadata>> {
        let path_str = utils::path_to_string(path);

        let mut stmt = self
            .connection
            .prepare_cached("SELECT modified, version FROM tracks WHERE path = ?1")
            .context("Failed to prepare get statement")?;

        let result = stmt.query_row(params![path_str], |row| {
            let modified: i64 = row.get(0)?;
            let version: String = row.get(1)?;
            Ok(TrackMetadata {
                modified: i64_to_u64(modified),
                version,
            })
        });

        match result {
            Ok(metadata) => Ok(Some(metadata)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err).context("Failed to get track"),
        }
    }

    /// Return the number of tracked entries.
    pub fn len(&self) -> Result<usize> {
        let count: i64 = self
            .connection
            .prepare_cached("SELECT COUNT(*) FROM tracks")
            .context("Failed to prepare count statement")?
            .query_row([], |row| row.get(0))
            .context("Failed to count tracks")?;

        Ok(i64_to_usize(count))
    }

    /// Return `true` if the database has no tracked entries.
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Remove outdated entries from the database.
    ///
    /// Removes entries where the version does not match the current version (pure SQL)
    /// and entries whose path no longer exists on disk (filesystem check in a transaction).
    /// Returns the total number of entries removed.
    pub fn clean(&self) -> Result<usize> {
        // First: remove entries with an outdated version (pure SQL).
        let version_removed: usize = self
            .connection
            .prepare_cached("DELETE FROM tracks WHERE version != ?1")
            .context("Failed to prepare version clean statement")?
            .execute(params![VERSION])
            .context("Failed to clean outdated version entries")?;

        // Second: collect remaining paths and check the filesystem.
        let paths_to_remove: Vec<String> = {
            let mut stmt = self
                .connection
                .prepare_cached("SELECT path FROM tracks")
                .context("Failed to prepare path query for clean")?;

            stmt.query_map([], |row| row.get::<_, String>(0))
                .context("Failed to query paths for clean")?
                .filter_map(Result::ok)
                .filter(|path_str| !Path::new(path_str).exists())
                .collect()
        };

        // Remove non-existent paths in a transaction.
        let fs_removed = if paths_to_remove.is_empty() {
            0
        } else {
            self.connection
                .execute_batch("BEGIN")
                .context("Failed to begin clean transaction")?;

            let result = (|| -> Result<usize> {
                let mut removed: usize = 0;
                for path_str in &paths_to_remove {
                    removed += self
                        .connection
                        .prepare_cached("DELETE FROM tracks WHERE path = ?1")
                        .context("Failed to prepare delete statement")?
                        .execute(params![path_str])
                        .context("Failed to delete non-existent path")?;
                }
                Ok(removed)
            })();

            match &result {
                Ok(_) => {
                    self.connection
                        .execute_batch("COMMIT")
                        .context("Failed to commit clean transaction")?;
                }
                Err(_) => {
                    let _ = self.connection.execute_batch("ROLLBACK");
                }
            }

            result?
        };

        Ok(version_removed + fs_removed)
    }

    /// Initialize connection pragmas and create the schema.
    fn initialize(conn: Connection, db_path: PathBuf) -> Result<Self> {
        conn.execute_batch("PRAGMA journal_mode = WAL;")
            .context("Failed to set WAL journal mode")?;
        conn.execute_batch("PRAGMA synchronous = NORMAL;")
            .context("Failed to set synchronous mode")?;
        conn.execute_batch("PRAGMA cache_size = -2000;")
            .context("Failed to set cache size")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .context("Failed to set busy timeout")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tracks (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                modified INTEGER NOT NULL,
                version TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_tracks_path ON tracks(path);",
        )
        .context("Failed to initialize database schema")?;

        Ok(Self {
            connection: conn,
            db_path,
        })
    }

    /// Check whether a path already exists in the database.
    fn path_exists(&self, path_str: &str) -> Result<bool> {
        let mut stmt = self
            .connection
            .prepare_cached("SELECT 1 FROM tracks WHERE path = ?1")
            .context("Failed to prepare path exists statement")?;

        match stmt.query_row(params![path_str], |_| Ok(())) {
            Ok(()) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(err) => Err(err).context("Failed to check if path exists"),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::open_in_memory().expect("Failed to open in-memory state database")
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("db_path", &self.db_path)
            .finish_non_exhaustive()
    }
}

/// Safely convert a `u64` to `i64` for `SQLite` storage.
/// Values that overflow `i64::MAX` are clamped.
fn u64_to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// Safely convert an `i64` from `SQLite` back to `u64`.
/// Negative values are mapped to `0`.
fn i64_to_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

/// Safely convert an `i64` count to `usize`.
/// Negative values are mapped to `0`.
fn i64_to_usize(value: i64) -> usize {
    usize::try_from(value).unwrap_or(0)
}

#[cfg(test)]
mod test_state_database {
    use super::*;

    #[test]
    fn test_insert_and_get_roundtrip() {
        let state = State::open_in_memory().unwrap();
        let path = Path::new("/music/test.aif");
        let metadata = TrackMetadata {
            modified: 123_456_789,
            version: "1.0.0".to_string(),
        };

        state.insert(path, &metadata).unwrap();

        let retrieved = state.get(path).unwrap().expect("should find inserted entry");
        assert_eq!(retrieved.modified, metadata.modified);
        assert_eq!(retrieved.version, metadata.version);
    }

    #[test]
    fn test_batch_insert_counts() {
        let state = State::open_in_memory().unwrap();

        let path1 = Path::new("/music/track1.aif");
        let path2 = Path::new("/music/track2.aif");
        let meta = TrackMetadata {
            modified: 100,
            version: "1.0.0".to_string(),
        };

        // Pre-insert one entry so it counts as an update in the batch.
        state.insert(path1, &meta).unwrap();

        let entries: Vec<(&Path, &TrackMetadata)> = vec![(path1, &meta), (path2, &meta)];
        let (added, updated) = state.batch_insert(&entries).unwrap();
        assert_eq!(added, 1);
        assert_eq!(updated, 1);
        assert_eq!(state.len().unwrap(), 2);
    }

    #[test]
    fn test_get_returns_none_for_missing() {
        let state = State::open_in_memory().unwrap();
        let result = state.get(Path::new("/nonexistent/path.aif")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_len_and_is_empty() {
        let state = State::open_in_memory().unwrap();
        assert!(state.is_empty().unwrap());
        assert_eq!(state.len().unwrap(), 0);

        let path = Path::new("/music/test.aif");
        let meta = TrackMetadata {
            modified: 100,
            version: "1.0.0".to_string(),
        };
        state.insert(path, &meta).unwrap();

        assert!(!state.is_empty().unwrap());
        assert_eq!(state.len().unwrap(), 1);
    }

    #[test]
    fn test_clean_removes_wrong_version() {
        let state = State::open_in_memory().unwrap();
        let path = Path::new("/music/test.aif");
        let old_meta = TrackMetadata {
            modified: 100,
            version: "old_version".to_string(),
        };

        state.insert(path, &old_meta).unwrap();
        assert_eq!(state.len().unwrap(), 1);

        let removed = state.clean().unwrap();
        assert_eq!(removed, 1);
        assert!(state.is_empty().unwrap());
    }

    #[test]
    fn test_insert_returns_false_for_new_true_for_existing() {
        let state = State::open_in_memory().unwrap();
        let path = Path::new("/music/test.aif");
        let meta = TrackMetadata {
            modified: 100,
            version: "1.0.0".to_string(),
        };

        let was_update = state.insert(path, &meta).unwrap();
        assert!(!was_update, "first insert should report new entry");

        let was_update = state.insert(path, &meta).unwrap();
        assert!(was_update, "second insert should report existing entry updated");
    }

    #[test]
    fn test_debug_impl() {
        let state = State::open_in_memory().unwrap();
        let debug_str = format!("{state:?}");
        assert!(debug_str.contains("State"));
        assert!(debug_str.contains(":memory:"));
    }

    #[test]
    fn test_default_impl() {
        let state = State::default();
        assert!(state.is_empty().unwrap());
    }
}

#[cfg(test)]
mod test_conversion_functions {
    use super::*;

    #[test]
    fn u64_to_i64_zero_returns_zero() {
        assert_eq!(u64_to_i64(0), 0);
    }

    #[test]
    fn u64_to_i64_small_value_returns_same() {
        assert_eq!(u64_to_i64(100), 100);
    }

    #[test]
    fn u64_to_i64_max_i64_returns_max_i64() {
        assert_eq!(u64_to_i64(i64::MAX as u64), i64::MAX);
    }

    #[test]
    fn u64_to_i64_u64_max_clamps_to_i64_max() {
        assert_eq!(u64_to_i64(u64::MAX), i64::MAX);
    }

    #[test]
    fn u64_to_i64_overflow_clamps_to_i64_max() {
        assert_eq!(u64_to_i64(i64::MAX as u64 + 1), i64::MAX);
    }

    #[test]
    fn i64_to_u64_zero_returns_zero() {
        assert_eq!(i64_to_u64(0), 0);
    }

    #[test]
    fn i64_to_u64_small_value_returns_same() {
        assert_eq!(i64_to_u64(100), 100);
    }

    #[test]
    fn i64_to_u64_negative_returns_zero() {
        assert_eq!(i64_to_u64(-1), 0);
    }

    #[test]
    fn i64_to_u64_min_returns_zero() {
        assert_eq!(i64_to_u64(i64::MIN), 0);
    }

    #[test]
    fn i64_to_usize_zero_returns_zero() {
        assert_eq!(i64_to_usize(0), 0);
    }

    #[test]
    fn i64_to_usize_small_value_returns_same() {
        assert_eq!(i64_to_usize(100), 100);
    }

    #[test]
    fn i64_to_usize_negative_returns_zero() {
        assert_eq!(i64_to_usize(-1), 0);
    }

    #[test]
    fn i64_to_usize_min_returns_zero() {
        assert_eq!(i64_to_usize(i64::MIN), 0);
    }
}

#[cfg(test)]
mod test_state_path {
    use super::*;

    #[test]
    fn in_memory_state_path_returns_memory_string() {
        let state = State::open_in_memory().expect("Failed to open in-memory state");
        assert_eq!(state.path().to_string_lossy(), ":memory:");
    }
}

#[cfg(test)]
mod test_state_clean_nonexistent_paths {
    use std::path::Path;

    use super::*;

    #[test]
    fn clean_removes_entries_with_nonexistent_filesystem_paths() {
        let state = State::open_in_memory().expect("Failed to open in-memory state");

        let nonexistent_path_one = Path::new("/nonexistent/track_clean_test_one.aif");
        let nonexistent_path_two = Path::new("/nonexistent/track_clean_test_two.aif");

        let metadata_one = TrackMetadata {
            modified: 500,
            version: VERSION.to_string(),
        };
        let metadata_two = TrackMetadata {
            modified: 600,
            version: VERSION.to_string(),
        };

        state
            .insert(nonexistent_path_one, &metadata_one)
            .expect("Failed to insert first track");
        state
            .insert(nonexistent_path_two, &metadata_two)
            .expect("Failed to insert second track");

        assert_eq!(state.len().expect("Failed to get length"), 2);

        let removed_count = state.clean().expect("Failed to clean state");
        assert_eq!(removed_count, 2);
        assert!(state.is_empty().expect("Failed to check if state is empty"));
    }
}

#[cfg(test)]
mod test_state_insert_update_modifies_values {
    use std::path::Path;

    use super::*;

    #[test]
    fn second_insert_updates_modified_and_version() {
        let state = State::open_in_memory().expect("Failed to open in-memory state");
        let track_path = Path::new("/music/update_test_track.aif");

        let original_metadata = TrackMetadata {
            modified: 100,
            version: "1.0".to_string(),
        };
        state
            .insert(track_path, &original_metadata)
            .expect("Failed to insert original track");

        let updated_metadata = TrackMetadata {
            modified: 200,
            version: "2.0".to_string(),
        };
        state
            .insert(track_path, &updated_metadata)
            .expect("Failed to insert updated track");

        let retrieved = state
            .get(track_path)
            .expect("Failed to get track")
            .expect("Track should exist after insert");

        assert_eq!(retrieved.modified, 200);
        assert_eq!(retrieved.version, "2.0");
    }
}
