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
    pub fn insert(&mut self, path: &Path, metadata: &TrackMetadata) -> Result<bool> {
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
    pub fn batch_insert(&mut self, entries: &[(&Path, &TrackMetadata)]) -> Result<(usize, usize)> {
        let tx = self
            .connection
            .transaction()
            .context("Failed to begin batch insert transaction")?;

        let mut added: usize = 0;
        let mut updated: usize = 0;

        for (path, metadata) in entries {
            let path_str = utils::path_to_string(path);
            let modified = u64_to_i64(metadata.modified);

            let existed: bool = tx
                .prepare_cached("SELECT 1 FROM tracks WHERE path = ?1")
                .context("Failed to prepare path exists statement")?
                .query_row(params![path_str], |_| Ok(()))
                .is_ok();

            tx.prepare_cached(
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

        tx.commit().context("Failed to commit batch insert transaction")?;
        Ok((added, updated))
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
    pub fn clean(&mut self) -> Result<usize> {
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
            let tx = self
                .connection
                .transaction()
                .context("Failed to begin clean transaction")?;

            let mut removed: usize = 0;
            for path_str in &paths_to_remove {
                removed += tx
                    .prepare_cached("DELETE FROM tracks WHERE path = ?1")
                    .context("Failed to prepare delete statement")?
                    .execute(params![path_str])
                    .context("Failed to delete non-existent path")?;
            }

            tx.commit().context("Failed to commit clean transaction")?;
            removed
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
            );",
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
mod test_state_insert {
    use std::path::Path;

    use super::*;

    #[test]
    fn insert_and_get_roundtrip() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let path = Path::new("/music/test.aif");
        let metadata = TrackMetadata {
            modified: 123_456_789,
            version: "1.0.0".to_string(),
        };

        state.insert(path, &metadata).expect("Failed to insert");

        let retrieved = state
            .get(path)
            .expect("Failed to get")
            .expect("Should find inserted entry");
        assert_eq!(retrieved.modified, metadata.modified);
        assert_eq!(retrieved.version, metadata.version);
    }

    #[test]
    fn returns_false_for_new_true_for_existing() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let path = Path::new("/music/test.aif");
        let meta = TrackMetadata {
            modified: 100,
            version: "1.0.0".to_string(),
        };

        let was_update = state.insert(path, &meta).expect("Failed to insert");
        assert!(!was_update, "first insert should report new entry");

        let was_update = state.insert(path, &meta).expect("Failed to insert");
        assert!(was_update, "second insert should report existing entry updated");
    }

    #[test]
    fn second_insert_updates_modified_and_version() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
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

    #[test]
    fn inserts_multiple_distinct_paths() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        let paths = [
            Path::new("/music/track_a.aif"),
            Path::new("/music/track_b.mp3"),
            Path::new("/other/dir/track_c.aif"),
        ];
        let meta = TrackMetadata {
            modified: 42,
            version: "1.0".to_string(),
        };

        for path in &paths {
            state.insert(path, &meta).expect("Failed to insert track");
        }

        assert_eq!(state.len().expect("Failed to get length"), 3);

        for path in &paths {
            let retrieved = state
                .get(path)
                .expect("Failed to get track")
                .expect("Track should exist");
            assert_eq!(retrieved.modified, 42);
        }
    }

    #[test]
    fn each_path_has_independent_metadata() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        let path_a = Path::new("/music/track_a.aif");
        let path_b = Path::new("/music/track_b.aif");

        let meta_a = TrackMetadata {
            modified: 100,
            version: "1.0".to_string(),
        };
        let meta_b = TrackMetadata {
            modified: 999,
            version: "2.5".to_string(),
        };

        state.insert(path_a, &meta_a).expect("Failed to insert track_a");
        state.insert(path_b, &meta_b).expect("Failed to insert track_b");

        let retrieved_a = state.get(path_a).expect("Failed to get").expect("Should exist");
        let retrieved_b = state.get(path_b).expect("Failed to get").expect("Should exist");

        assert_eq!(retrieved_a.modified, 100);
        assert_eq!(retrieved_a.version, "1.0");
        assert_eq!(retrieved_b.modified, 999);
        assert_eq!(retrieved_b.version, "2.5");
    }

    #[test]
    fn zero_modified_timestamp() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let path = Path::new("/music/zero_ts.aif");
        let meta = TrackMetadata {
            modified: 0,
            version: "1.0".to_string(),
        };

        state.insert(path, &meta).expect("Failed to insert");
        let retrieved = state.get(path).expect("Failed to get").expect("Should exist");
        assert_eq!(retrieved.modified, 0);
    }

    #[test]
    fn max_u64_modified_timestamp_roundtrips_via_clamping() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let path = Path::new("/music/max_ts.aif");
        let meta = TrackMetadata {
            modified: u64::MAX,
            version: "1.0".to_string(),
        };

        state.insert(path, &meta).expect("Failed to insert");
        let retrieved = state.get(path).expect("Failed to get").expect("Should exist");
        // u64::MAX is clamped to i64::MAX on write and converted back to u64
        assert_eq!(retrieved.modified, i64::MAX as u64);
    }

    #[test]
    fn empty_version_string() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let path = Path::new("/music/empty_ver.aif");
        let meta = TrackMetadata {
            modified: 50,
            version: String::new(),
        };

        state.insert(path, &meta).expect("Failed to insert");
        let retrieved = state.get(path).expect("Failed to get").expect("Should exist");
        assert_eq!(retrieved.version, "");
    }

    #[test]
    fn path_with_unicode_characters() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let path = Path::new("/音楽/トラック.aif");
        let meta = TrackMetadata {
            modified: 300,
            version: "1.0".to_string(),
        };

        state.insert(path, &meta).expect("Failed to insert unicode path");
        let retrieved = state.get(path).expect("Failed to get").expect("Should exist");
        assert_eq!(retrieved.modified, 300);
    }

    #[test]
    fn path_with_spaces_and_special_characters() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let path = Path::new("/music/Artist Name - Song Title (Remix) [2024].aif");
        let meta = TrackMetadata {
            modified: 400,
            version: "1.0".to_string(),
        };

        state.insert(path, &meta).expect("Failed to insert");
        let retrieved = state.get(path).expect("Failed to get").expect("Should exist");
        assert_eq!(retrieved.modified, 400);
    }
}

#[cfg(test)]
mod test_state_batch_insert {
    use std::path::Path;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn empty_batch_returns_zero_counts() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let entries: Vec<(&Path, &TrackMetadata)> = vec![];

        let (added, updated) = state.batch_insert(&entries).expect("Failed to batch insert");
        assert_eq!(added, 0);
        assert_eq!(updated, 0);
        assert!(state.is_empty().expect("Failed to check empty"));
    }

    #[test]
    fn all_new_entries_counted_as_added() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        let path1 = Path::new("/music/new1.aif");
        let path2 = Path::new("/music/new2.aif");
        let path3 = Path::new("/music/new3.mp3");
        let meta = TrackMetadata {
            modified: 100,
            version: "1.0".to_string(),
        };

        let entries: Vec<(&Path, &TrackMetadata)> = vec![(path1, &meta), (path2, &meta), (path3, &meta)];
        let (added, updated) = state.batch_insert(&entries).expect("Failed to batch insert");

        assert_eq!(added, 3);
        assert_eq!(updated, 0);
        assert_eq!(state.len().expect("Failed to get length"), 3);
    }

    #[test]
    fn all_existing_entries_counted_as_updated() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        let path1 = Path::new("/music/existing1.aif");
        let path2 = Path::new("/music/existing2.aif");
        let meta_v1 = TrackMetadata {
            modified: 100,
            version: "1.0".to_string(),
        };
        let meta_v2 = TrackMetadata {
            modified: 200,
            version: "2.0".to_string(),
        };

        state.insert(path1, &meta_v1).expect("Failed to insert");
        state.insert(path2, &meta_v1).expect("Failed to insert");

        let entries: Vec<(&Path, &TrackMetadata)> = vec![(path1, &meta_v2), (path2, &meta_v2)];
        let (added, updated) = state.batch_insert(&entries).expect("Failed to batch insert");

        assert_eq!(added, 0);
        assert_eq!(updated, 2);
        assert_eq!(state.len().expect("Failed to get length"), 2);
    }

    #[test]
    fn mixed_new_and_existing_counts() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        let path1 = Path::new("/music/track1.aif");
        let path2 = Path::new("/music/track2.aif");
        let meta = TrackMetadata {
            modified: 100,
            version: "1.0.0".to_string(),
        };

        // Pre-insert one entry so it counts as an update in the batch.
        state.insert(path1, &meta).expect("Failed to insert");

        let entries: Vec<(&Path, &TrackMetadata)> = vec![(path1, &meta), (path2, &meta)];
        let (added, updated) = state.batch_insert(&entries).expect("Failed to batch insert");
        assert_eq!(added, 1);
        assert_eq!(updated, 1);
        assert_eq!(state.len().expect("Failed to get length"), 2);
    }

    #[test]
    fn batch_inserted_values_are_retrievable() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        let path1 = Path::new("/music/batch_verify1.aif");
        let path2 = Path::new("/music/batch_verify2.mp3");
        let meta1 = TrackMetadata {
            modified: 111,
            version: "1.1".to_string(),
        };
        let meta2 = TrackMetadata {
            modified: 222,
            version: "2.2".to_string(),
        };

        let entries: Vec<(&Path, &TrackMetadata)> = vec![(path1, &meta1), (path2, &meta2)];
        state.batch_insert(&entries).expect("Failed to batch insert");

        let retrieved1 = state.get(path1).expect("Failed to get").expect("Should exist");
        assert_eq!(retrieved1.modified, 111);
        assert_eq!(retrieved1.version, "1.1");

        let retrieved2 = state.get(path2).expect("Failed to get").expect("Should exist");
        assert_eq!(retrieved2.modified, 222);
        assert_eq!(retrieved2.version, "2.2");
    }

    #[test]
    fn batch_update_overwrites_previous_values() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        let path = Path::new("/music/batch_overwrite.aif");
        let old_meta = TrackMetadata {
            modified: 100,
            version: "old".to_string(),
        };
        state.insert(path, &old_meta).expect("Failed to insert");

        let new_meta = TrackMetadata {
            modified: 999,
            version: "new".to_string(),
        };
        let entries: Vec<(&Path, &TrackMetadata)> = vec![(path, &new_meta)];
        state.batch_insert(&entries).expect("Failed to batch insert");

        let retrieved = state.get(path).expect("Failed to get").expect("Should exist");
        assert_eq!(retrieved.modified, 999);
        assert_eq!(retrieved.version, "new");
    }

    #[test]
    fn duplicate_paths_within_single_batch_uses_last_value() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        let path = Path::new("/music/dup.aif");
        let meta_first = TrackMetadata {
            modified: 100,
            version: "1.0".to_string(),
        };
        let meta_second = TrackMetadata {
            modified: 999,
            version: "2.0".to_string(),
        };

        let entries: Vec<(&Path, &TrackMetadata)> = vec![(path, &meta_first), (path, &meta_second)];
        let (added, updated) = state.batch_insert(&entries).expect("Failed to batch insert");

        // First occurrence is new, second is an update
        assert_eq!(added, 1);
        assert_eq!(updated, 1);
        assert_eq!(state.len().expect("Failed to get length"), 1);

        let retrieved = state.get(path).expect("Failed to get").expect("Should exist");
        assert_eq!(retrieved.modified, 999, "Should have the last-written value");
        assert_eq!(retrieved.version, "2.0");
    }

    #[test]
    fn batch_insert_many_entries() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        let meta = TrackMetadata {
            modified: 500,
            version: "1.0".to_string(),
        };

        let paths: Vec<PathBuf> = (0..200)
            .map(|index| PathBuf::from(format!("/music/track_{index}.aif")))
            .collect();

        let entries: Vec<(&Path, &TrackMetadata)> = paths.iter().map(|path| (path.as_path(), &meta)).collect();

        let (added, updated) = state.batch_insert(&entries).expect("Failed to batch insert");
        assert_eq!(added, 200);
        assert_eq!(updated, 0);
        assert_eq!(state.len().expect("Failed to get length"), 200);

        // Verify a sample of entries
        let sample = state
            .get(Path::new("/music/track_99.aif"))
            .expect("Failed to get")
            .expect("Should exist");
        assert_eq!(sample.modified, 500);
    }
}

#[cfg(test)]
mod test_state_clean {
    use std::path::Path;

    use super::*;

    #[test]
    fn clean_on_empty_database_returns_zero() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let removed = state.clean().expect("Failed to clean");
        assert_eq!(removed, 0);
        assert!(state.is_empty().expect("Failed to check empty"));
    }

    #[test]
    fn removes_wrong_version() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let path = Path::new("/music/test.aif");
        let old_meta = TrackMetadata {
            modified: 100,
            version: "old_version".to_string(),
        };

        state.insert(path, &old_meta).expect("Failed to insert");
        assert_eq!(state.len().expect("Failed to get length"), 1);

        let removed = state.clean().expect("Failed to clean");
        assert_eq!(removed, 1);
        assert!(state.is_empty().expect("Failed to check empty"));
    }

    #[test]
    fn removes_entries_with_nonexistent_filesystem_paths() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

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

    #[test]
    fn retains_entries_with_current_version_and_existing_path() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let meta = TrackMetadata {
            modified: 100,
            version: VERSION.to_string(),
        };

        state.insert(&path, &meta).expect("Failed to insert");
        assert_eq!(state.len().expect("Failed to get length"), 1);

        let removed = state.clean().expect("Failed to clean");
        assert_eq!(
            removed, 0,
            "Entry with current version and existing path should be retained"
        );
        assert_eq!(state.len().expect("Failed to get length"), 1);
    }

    #[test]
    fn removes_wrong_version_and_nonexistent_paths_together() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        // Entry 1: wrong version (will be removed by version check)
        let wrong_version_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let wrong_version_meta = TrackMetadata {
            modified: 100,
            version: "ancient_version".to_string(),
        };
        state
            .insert(&wrong_version_path, &wrong_version_meta)
            .expect("Failed to insert");

        // Entry 2: correct version but nonexistent path (will be removed by fs check)
        let nonexistent_path = Path::new("/does/not/exist/track.aif");
        let current_version_meta = TrackMetadata {
            modified: 200,
            version: VERSION.to_string(),
        };
        state
            .insert(nonexistent_path, &current_version_meta)
            .expect("Failed to insert");

        // Entry 3: correct version and existing path (should be retained)
        let valid_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let valid_meta = TrackMetadata {
            modified: 300,
            version: VERSION.to_string(),
        };
        state.insert(&valid_path, &valid_meta).expect("Failed to insert");

        // Entry 1 and entry 3 have the same path so only 2 entries exist.
        // Entry 1 was overwritten by entry 3 (same path, upsert).
        // Entry 2 is the nonexistent path.
        assert_eq!(state.len().expect("Failed to get length"), 2);

        let removed = state.clean().expect("Failed to clean");
        // Only the nonexistent path should be removed (entry 3 replaced entry 1)
        assert_eq!(removed, 1, "Should remove only the nonexistent-path entry");
        assert_eq!(state.len().expect("Failed to get length"), 1);

        // The surviving entry should be the valid one
        let surviving = state.get(&valid_path).expect("Failed to get").expect("Should exist");
        assert_eq!(surviving.modified, 300);
        assert_eq!(surviving.version, VERSION);
    }

    #[test]
    fn removes_both_wrong_version_and_nonexistent_in_separate_passes() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        // Entry with wrong version (will be removed in first pass)
        let path_wrong_ver = Path::new("/music/wrong_version.aif");
        let meta_wrong_ver = TrackMetadata {
            modified: 100,
            version: "0.0.1".to_string(),
        };
        state.insert(path_wrong_ver, &meta_wrong_ver).expect("Failed to insert");

        // Entry with correct version but nonexistent path (removed in second pass)
        let path_nonexistent = Path::new("/nonexistent/correct_version.aif");
        let meta_correct_ver = TrackMetadata {
            modified: 200,
            version: VERSION.to_string(),
        };
        state
            .insert(path_nonexistent, &meta_correct_ver)
            .expect("Failed to insert");

        assert_eq!(state.len().expect("Failed to get length"), 2);

        let removed = state.clean().expect("Failed to clean");
        assert_eq!(removed, 2, "Both entries should be removed");
        assert!(state.is_empty().expect("Failed to check empty"));
    }

    #[test]
    fn return_value_matches_actual_removals() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        // Insert 3 entries with wrong version
        for index in 0..3 {
            let path_string = format!("/music/wrong_v_{index}.aif");
            let path = Path::new(&path_string);
            let meta = TrackMetadata {
                modified: 100,
                version: "wrong".to_string(),
            };
            state.insert(path, &meta).expect("Failed to insert");
        }

        // Insert 2 entries with correct version but nonexistent paths
        for index in 0..2 {
            let path_string = format!("/nonexistent/correct_v_{index}.aif");
            let path = Path::new(&path_string);
            let meta = TrackMetadata {
                modified: 200,
                version: VERSION.to_string(),
            };
            state.insert(path, &meta).expect("Failed to insert");
        }

        assert_eq!(state.len().expect("Failed to get length"), 5);

        let removed = state.clean().expect("Failed to clean");
        assert_eq!(removed, 5, "All 5 entries should be removed");
        assert_eq!(state.len().expect("Failed to get length"), 0);
    }
}

#[cfg(test)]
mod test_state_queries {
    use std::path::Path;

    use super::*;

    #[test]
    fn get_returns_none_for_missing() {
        let state = State::open_in_memory().expect("Failed to open in-memory state");
        let result = state.get(Path::new("/nonexistent/path.aif")).expect("Failed to get");
        assert!(result.is_none());
    }

    #[test]
    fn repeated_gets_return_same_result() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let path = Path::new("/music/stable.aif");
        let meta = TrackMetadata {
            modified: 42,
            version: "1.0".to_string(),
        };
        state.insert(path, &meta).expect("Failed to insert");

        let first = state.get(path).expect("Failed to get").expect("Should exist");
        let second = state.get(path).expect("Failed to get").expect("Should exist");
        let third = state.get(path).expect("Failed to get").expect("Should exist");

        assert_eq!(first.modified, second.modified);
        assert_eq!(second.modified, third.modified);
        assert_eq!(first.version, second.version);
    }

    #[test]
    fn get_missing_path_does_not_affect_existing_entries() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        let existing_path = Path::new("/music/exists.aif");
        let missing_path = Path::new("/music/missing.aif");
        let meta = TrackMetadata {
            modified: 42,
            version: "1.0".to_string(),
        };
        state.insert(existing_path, &meta).expect("Failed to insert");

        let result = state.get(missing_path).expect("Failed to get");
        assert!(result.is_none());

        let existing = state.get(existing_path).expect("Failed to get").expect("Should exist");
        assert_eq!(existing.modified, 42);
        assert_eq!(state.len().expect("Failed to get length"), 1);
    }

    #[test]
    fn len_and_is_empty() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        assert!(state.is_empty().expect("Failed to check empty"));
        assert_eq!(state.len().expect("Failed to get length"), 0);

        let path = Path::new("/music/test.aif");
        let meta = TrackMetadata {
            modified: 100,
            version: "1.0.0".to_string(),
        };
        state.insert(path, &meta).expect("Failed to insert");

        assert!(!state.is_empty().expect("Failed to check empty"));
        assert_eq!(state.len().expect("Failed to get length"), 1);
    }

    #[test]
    fn len_tracks_inserts_upserts_and_cleans() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");

        assert_eq!(state.len().expect("Failed to get length"), 0);

        let meta = TrackMetadata {
            modified: 100,
            version: "old".to_string(),
        };

        for index in 0..5 {
            let path_string = format!("/music/track_{index}.aif");
            let path = Path::new(&path_string);
            state.insert(path, &meta).expect("Failed to insert");
        }
        assert_eq!(state.len().expect("Failed to get length"), 5);

        // Upsert one existing entry (len should stay the same)
        let path_zero = Path::new("/music/track_0.aif");
        state.insert(path_zero, &meta).expect("Failed to insert");
        assert_eq!(state.len().expect("Failed to get length"), 5);

        // Clean removes all (wrong version + nonexistent paths)
        let removed = state.clean().expect("Failed to clean");
        assert_eq!(removed, 5);
        assert_eq!(state.len().expect("Failed to get length"), 0);
    }

    #[test]
    fn is_empty_reflects_state_through_lifecycle() {
        let mut state = State::open_in_memory().expect("Failed to open in-memory state");
        assert!(state.is_empty().expect("Failed to check empty"));

        let path = Path::new("/nonexistent/track.aif");
        let meta = TrackMetadata {
            modified: 100,
            version: "old".to_string(),
        };
        state.insert(path, &meta).expect("Failed to insert");
        assert!(!state.is_empty().expect("Failed to check empty"));

        state.clean().expect("Failed to clean");
        assert!(state.is_empty().expect("Failed to check empty"));
    }

    #[test]
    fn in_memory_state_path_returns_memory_string() {
        let state = State::open_in_memory().expect("Failed to open in-memory state");
        assert_eq!(state.path().to_string_lossy(), ":memory:");
    }

    #[test]
    fn debug_impl_contains_struct_name_and_path() {
        let state = State::open_in_memory().expect("Failed to open in-memory state");
        let debug_str = format!("{state:?}");
        assert!(debug_str.contains("State"));
        assert!(debug_str.contains(":memory:"));
    }

    #[test]
    fn default_impl_creates_empty_state() {
        let state = State::default();
        assert!(state.is_empty().expect("Failed to check empty"));
    }

    #[test]
    fn schema_creation_is_idempotent() {
        let conn = Connection::open_in_memory().expect("Failed to open connection");
        let state = State::initialize(conn, PathBuf::from(":memory:")).expect("First init should succeed");

        // Simulate a second schema creation
        state
            .connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS tracks (
                    id INTEGER PRIMARY KEY,
                    path TEXT NOT NULL UNIQUE,
                    modified INTEGER NOT NULL,
                    version TEXT NOT NULL
                );",
            )
            .expect("Re-creating schema should not fail");

        // Verify the database still works after the redundant schema creation
        state
            .connection
            .execute(
                "INSERT INTO tracks (path, modified, version) VALUES (?1, ?2, ?3)",
                params!["/music/test.aif", 100i64, "1.0"],
            )
            .expect("Insert should work after double init");

        let count: i64 = state
            .connection
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
            .expect("Count query should work");
        assert_eq!(count, 1);
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
