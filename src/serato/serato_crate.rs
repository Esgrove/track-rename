use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use colored::Colorize;

/// A parsed Serato `.crate` file.
#[derive(Debug, Clone)]
pub struct SeratoCrate {
    /// The crate name (derived from filename, `%%` hierarchy decoded to ` > `).
    pub name: String,
    /// The version string from the header (e.g. `"1.0/Serato ScratchLive Crate"`).
    pub version: String,
    /// Column definitions: `(column_name, column_width)`.
    pub columns: Vec<(String, String)>,
    /// Track file paths from `ptrk` tags.
    pub tracks: Vec<PathBuf>,
}

impl SeratoCrate {
    /// Read and parse a Serato `.crate` file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let name = crate_name_from_path(path);
        let data = fs::read(path).with_context(|| format!("Failed to read crate file: {}", path.display()))?;

        ensure!(!data.is_empty(), "Crate file is empty: {}", path.display());

        let mut offset = 0;
        let mut version = String::new();
        let mut columns = Vec::new();
        let mut tracks = Vec::new();

        // Parse the version header — must be the first tag.
        if offset < data.len() {
            let (tag, value) = read_tag(&data, &mut offset)?;
            ensure!(tag == "vrsn", "Expected 'vrsn' header tag, found '{tag}'");
            version = decode_utf16be(&value).context("Failed to decode version string")?;
        }

        // Parse remaining tags.
        while offset < data.len() {
            let (tag, value) = read_tag(&data, &mut offset)?;
            match tag.as_str() {
                "ovct" => {
                    // Column definition block containing tvcn + tvcw sub-tags.
                    columns.push(parse_column_definition(&value));
                }
                "otrk" => {
                    // Track entry containing a nested ptrk tag with the file path.
                    if let Some(path) = parse_track_entry(&value)? {
                        tracks.push(path);
                    }
                }
                _ => {
                    // Unknown top-level tag (e.g. osrt) — skip gracefully.
                }
            }
        }

        Ok(Self {
            name,
            version,
            columns,
            tracks,
        })
    }

    /// Return the number of tracks in this crate.
    #[must_use]
    pub const fn track_count(&self) -> usize {
        self.tracks.len()
    }
}

impl fmt::Display for SeratoCrate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} track{}",
            self.name.bold().cyan(),
            self.tracks.len(),
            if self.tracks.len() == 1 { "" } else { "s" }
        )
    }
}

/// List all `.crate` files in the given directory, sorted alphabetically.
pub fn list_crates(dir: &Path) -> Result<Vec<PathBuf>> {
    ensure!(dir.is_dir(), "Not a directory: {}", dunce::simplified(dir).display());

    let mut crates: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "crate") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    crates.sort();
    Ok(crates)
}

/// Return the default Serato Subcrates directory path.
pub fn default_subcrates_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to determine home directory")?;
    Ok(home.join("Music/_Serato_/Subcrates"))
}

/// Derive a display name from a `.crate` file path.
///
/// Strips the `.crate` extension and replaces `%%` hierarchy separators with ` > `.
#[must_use]
pub fn crate_name_from_path(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().replace("%%", " > "))
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Read a single TLV (tag-length-value) entry from `data` at the given `offset`.
///
/// Returns `(tag_name, value_bytes)` and advances `offset` past the entry.
fn read_tag(data: &[u8], offset: &mut usize) -> Result<(String, Vec<u8>)> {
    ensure!(
        *offset + 8 <= data.len(),
        "Not enough data for tag header at offset {}",
        *offset
    );

    let tag = std::str::from_utf8(&data[*offset..*offset + 4])
        .context("Invalid tag name")?
        .to_string();

    let len = u32::from_be_bytes(
        data[*offset + 4..*offset + 8]
            .try_into()
            .context("Failed to read tag length")?,
    ) as usize;

    *offset += 8;

    ensure!(
        *offset + len <= data.len(),
        "Tag '{tag}' at offset {} declares {len} bytes but only {} remain",
        *offset - 8,
        data.len() - *offset
    );

    let value = data[*offset..*offset + len].to_vec();
    *offset += len;
    Ok((tag, value))
}

/// Decode a byte slice as UTF-16 big-endian.
fn decode_utf16be(data: &[u8]) -> Result<String> {
    ensure!(
        data.len().is_multiple_of(2),
        "UTF-16BE data has odd length: {}",
        data.len()
    );
    let u16s: Vec<u16> = data.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
    String::from_utf16(&u16s).context("Invalid UTF-16BE data")
}

/// Parse a column definition (`ovct`) block.
///
/// Expected sub-tags: `tvcn` (column name) and `tvcw` (column width).
fn parse_column_definition(data: &[u8]) -> (String, String) {
    let mut offset = 0;
    let mut col_name: Option<String> = None;
    let mut col_width: Option<String> = None;

    while offset + 8 <= data.len() {
        if let Ok((tag, value)) = read_tag(data, &mut offset) {
            match tag.as_str() {
                "tvcn" => {
                    col_name = decode_utf16be(&value).ok();
                }
                "tvcw" => {
                    col_width = decode_utf16be(&value).ok();
                }
                _ => {}
            }
        } else {
            break;
        }
    }

    (col_name.unwrap_or_default(), col_width.unwrap_or_default())
}

/// Parse a track entry (`otrk`) block.
///
/// Expected sub-tag: `ptrk` whose value is a UTF-16BE file path.
fn parse_track_entry(data: &[u8]) -> Result<Option<PathBuf>> {
    let mut offset = 0;

    while offset + 8 <= data.len() {
        let (tag, value) = read_tag(data, &mut offset)?;
        if tag == "ptrk" {
            let path_str = decode_utf16be(&value).context("Failed to decode track path")?;
            // Paths in crate files are relative to the volume root.
            // On macOS, prefix with `/` to get an absolute path.
            let path = if cfg!(target_os = "macos") && !path_str.starts_with('/') {
                PathBuf::from(format!("/{path_str}"))
            } else {
                PathBuf::from(&path_str)
            };
            return Ok(Some(path));
        }
    }

    bail!("otrk block does not contain a ptrk sub-tag");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a TLV tag from its name and raw value bytes.
    fn make_tag(name: [u8; 4], value: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&name);
        buf.extend_from_slice(&(value.len() as u32).to_be_bytes());
        buf.extend_from_slice(value);
        buf
    }

    /// Encode a string as UTF-16BE bytes.
    fn encode_utf16be(s: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        for code_unit in s.encode_utf16() {
            buf.extend_from_slice(&code_unit.to_be_bytes());
        }
        buf
    }

    #[test]
    fn test_decode_utf16be_ascii() {
        let encoded = encode_utf16be("hello");
        let decoded = decode_utf16be(&encoded).unwrap();
        assert_eq!(decoded, "hello");
    }

    #[test]
    fn test_decode_utf16be_unicode() {
        let encoded = encode_utf16be("café ☕");
        let decoded = decode_utf16be(&encoded).unwrap();
        assert_eq!(decoded, "café ☕");
    }

    #[test]
    fn test_decode_utf16be_empty() {
        let decoded = decode_utf16be(&[]).unwrap();
        assert_eq!(decoded, "");
    }

    #[test]
    fn test_decode_utf16be_odd_length() {
        let result = decode_utf16be(&[0x00, 0x41, 0x00]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_tag() {
        let data = make_tag(*b"vrsn", &[0x00, 0x41, 0x00, 0x42]);
        let mut offset = 0;
        let (tag, value) = read_tag(&data, &mut offset).unwrap();
        assert_eq!(tag, "vrsn");
        assert_eq!(value, &[0x00, 0x41, 0x00, 0x42]);
        assert_eq!(offset, data.len());
    }

    #[test]
    fn test_read_tag_not_enough_data() {
        let data = [0x76, 0x72, 0x73]; // only 3 bytes
        let mut offset = 0;
        assert!(read_tag(&data, &mut offset).is_err());
    }

    #[test]
    fn test_read_tag_length_exceeds_data() {
        // Tag claims 100 bytes but only 2 follow.
        let mut data = Vec::new();
        data.extend_from_slice(b"test");
        data.extend_from_slice(&100u32.to_be_bytes());
        data.extend_from_slice(&[0x00, 0x01]);
        let mut offset = 0;
        assert!(read_tag(&data, &mut offset).is_err());
    }

    #[test]
    fn test_crate_name_from_path_simple() {
        let path = PathBuf::from("/some/dir/TEST.crate");
        assert_eq!(crate_name_from_path(&path), "TEST");
    }

    #[test]
    fn test_crate_name_from_path_hierarchy() {
        let path = PathBuf::from("/some/dir/X-BATTLE%%ALLSTYLES.crate");
        assert_eq!(crate_name_from_path(&path), "X-BATTLE > ALLSTYLES");
    }

    #[test]
    fn test_crate_name_from_path_deep_hierarchy() {
        let path = PathBuf::from("/dir/A%%B%%C.crate");
        assert_eq!(crate_name_from_path(&path), "A > B > C");
    }

    #[test]
    fn test_parse_column_definition() {
        let col_name = encode_utf16be("song");
        let col_width = encode_utf16be("250");
        let mut block = make_tag(*b"tvcn", &col_name);
        block.extend(make_tag(*b"tvcw", &col_width));

        let result = parse_column_definition(&block);
        assert_eq!(result, ("song".to_string(), "250".to_string()));
    }

    #[test]
    fn test_parse_track_entry() {
        let path_str = "Users/esgrove/Music/test.mp3";
        let encoded_path = encode_utf16be(path_str);
        let ptrk_block = make_tag(*b"ptrk", &encoded_path);

        let result = parse_track_entry(&ptrk_block).unwrap();
        assert!(result.is_some());
        let track_path = result.unwrap();

        if cfg!(target_os = "macos") {
            assert_eq!(track_path, PathBuf::from("/Users/esgrove/Music/test.mp3"));
        } else {
            assert_eq!(track_path, PathBuf::from("Users/esgrove/Music/test.mp3"));
        }
    }

    #[test]
    fn test_parse_minimal_crate_data() {
        // Build a minimal crate file in memory: vrsn header + one otrk.
        let version_str = encode_utf16be("1.0/Serato ScratchLive Crate");
        let mut data = make_tag(*b"vrsn", &version_str);

        let path_str = "Users/esgrove/Music/track.mp3";
        let encoded_path = encode_utf16be(path_str);
        let ptrk = make_tag(*b"ptrk", &encoded_path);
        let otrk = make_tag(*b"otrk", &ptrk);
        data.extend(otrk);

        // Parse directly from bytes (replicating from_file logic without filesystem).
        let mut offset = 0;
        let (tag, value) = read_tag(&data, &mut offset).unwrap();
        assert_eq!(tag, "vrsn");
        let version = decode_utf16be(&value).unwrap();
        assert_eq!(version, "1.0/Serato ScratchLive Crate");

        let (tag, value) = read_tag(&data, &mut offset).unwrap();
        assert_eq!(tag, "otrk");
        let track = parse_track_entry(&value).unwrap();
        assert!(track.is_some());
    }

    #[test]
    fn test_parse_crate_with_columns() {
        let version_str = encode_utf16be("1.0/Serato ScratchLive Crate");
        let mut data = make_tag(*b"vrsn", &version_str);

        // Add an ovct block.
        let col_name = encode_utf16be("song");
        let col_width = encode_utf16be("250");
        let mut ovct_content = make_tag(*b"tvcn", &col_name);
        ovct_content.extend(make_tag(*b"tvcw", &col_width));
        data.extend(make_tag(*b"ovct", &ovct_content));

        // Add one track.
        let path_str = "Users/test/Music/file.mp3";
        let encoded_path = encode_utf16be(path_str);
        let ptrk = make_tag(*b"ptrk", &encoded_path);
        data.extend(make_tag(*b"otrk", &ptrk));

        // Write to a temp file and parse.
        let dir = std::env::temp_dir();
        let crate_path = dir.join("TEST%%SUB.crate");
        fs::write(&crate_path, &data).unwrap();

        let parsed = SeratoCrate::from_file(&crate_path).unwrap();
        assert_eq!(parsed.name, "TEST > SUB");
        assert_eq!(parsed.version, "1.0/Serato ScratchLive Crate");
        assert_eq!(parsed.columns.len(), 1);
        assert_eq!(parsed.columns[0], ("song".to_string(), "250".to_string()));
        assert_eq!(parsed.track_count(), 1);

        let _ = fs::remove_file(&crate_path);
    }

    #[test]
    fn test_display() {
        let c = SeratoCrate {
            name: "TEST".to_string(),
            version: "1.0/Serato ScratchLive Crate".to_string(),
            columns: vec![],
            tracks: vec![PathBuf::from("/a.mp3"), PathBuf::from("/b.mp3")],
        };
        let display = format!("{c}");
        assert!(display.contains("TEST"));
        assert!(display.contains("2 tracks"));
    }

    #[test]
    fn test_display_single_track() {
        let c = SeratoCrate {
            name: "SOLO".to_string(),
            version: String::new(),
            columns: vec![],
            tracks: vec![PathBuf::from("/a.mp3")],
        };
        let display = format!("{c}");
        assert!(display.contains("1 track"));
        // Should NOT say "tracks" (plural).
        assert!(!display.contains("tracks"));
    }

    #[test]
    fn test_list_crates_nonexistent_dir() {
        let result = list_crates(Path::new("/nonexistent/dir/abc123"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_real_crate_file() {
        let crate_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/TEST.crate");
        if !crate_path.exists() {
            eprintln!("Test crate file not found, skipping: {}", crate_path.display());
            return;
        }

        let parsed = SeratoCrate::from_file(&crate_path).expect("Failed to parse TEST.crate");

        assert_eq!(parsed.name, "TEST");
        assert_eq!(parsed.version, "1.0/Serato ScratchLive Crate");

        // The file has 7 column definitions: song, artist, bpm, key, album, length, comment
        assert_eq!(parsed.columns.len(), 7);
        assert_eq!(parsed.columns[0].0, "song");
        assert_eq!(parsed.columns[1].0, "artist");
        assert_eq!(parsed.columns[2].0, "bpm");
        assert_eq!(parsed.columns[3].0, "key");
        assert_eq!(parsed.columns[4].0, "album");
        assert_eq!(parsed.columns[5].0, "length");
        assert_eq!(parsed.columns[6].0, "comment");

        // The file contains 5 tracks
        assert_eq!(parsed.track_count(), 5);

        let track_filenames: Vec<String> = parsed
            .tracks
            .iter()
            .filter_map(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
            .collect();

        assert_eq!(
            track_filenames,
            vec![
                "The Cardigans X Destiny's Child - My Favourite Name.mp3",
                "SZA feat. Doja Cat - Kill Bill (Nick Bike Remix) (Clean).mp3",
                "SZA feat. Doja Cat - Kill Bill (Nick Bike Remix).mp3",
                "SYLVESTER - Mighty Real (Nick Bike Edit).mp3",
                "SUGARLOAF GANGSTERS - Samba Swat.mp3",
            ]
        );

        // All paths should be absolute on macOS
        if cfg!(target_os = "macos") {
            for track in &parsed.tracks {
                assert!(track.is_absolute(), "Expected absolute path, got: {}", track.display());
            }
        }
    }
}
