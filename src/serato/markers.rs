use std::fmt::Display;
use std::io::BufRead;
use std::io::{Cursor, Read};
use std::{fmt, io, str};

use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use byteorder::{BigEndian, ReadBytesExt};
use colored::{ColoredString, Colorize};

/// Cue points, saved loops, track color, and BPM lock status
#[derive(Debug, Clone)]
pub enum Markers {
    BpmLock(BpmLock),
    Color(Color),
    Cue(Cue),
    Loop(Loop),
}

/// Boolean for BPM lock status.
/// True means lock is enabled.
/// <https://support.serato.com/hc/en-us/articles/235214887-Lock-Beatgrids>
#[derive(Debug, Clone)]
pub struct BpmLock {
    enabled: bool,
}

/// RGB colour.
/// Used for track, cues, and loops.
#[derive(Debug, Clone)]
pub struct Color {
    r: u8,
    b: u8,
    g: u8,
}

/// A cue point.
#[derive(Debug, Clone)]
pub struct Cue {
    /// Cue number
    index: u8,
    /// Position in milliseconds
    position: u32,
    /// RGB Color
    color: Color,
    /// Name
    name: String,
}

/// Saved loop.
#[derive(Debug, Clone)]
pub struct Loop {
    /// Loop number
    index: u8,
    /// Start position in milliseconds
    start_position: u32,
    /// End position in milliseconds
    end_position: u32,
    /// RGB Color
    color: Color,
    locked: bool,
    name: String,
}

impl Markers {
    pub fn parse(data: &[u8]) -> Result<Vec<Self>> {
        let b64_data_start = 2;
        let b64_data_end = data
            .iter()
            .position(|&x| x == b'\x00')
            .ok_or_else(|| anyhow!("No null terminator found"))?;
        let b64data = &data[b64_data_start..b64_data_end];

        // Remove linefeed characters
        let mut b64_data_cleaned = Vec::with_capacity(b64data.len());
        b64_data_cleaned.extend(b64data.iter().filter(|&&b| b != b'\n'));

        match b64_data_cleaned.len() % 4 {
            1 => b64_data_cleaned.extend_from_slice(b"A=="),
            2 => b64_data_cleaned.extend_from_slice(b"=="),
            3 => b64_data_cleaned.extend_from_slice(b"="),
            _ => {}
        }

        let payload = general_purpose::STANDARD
            .decode(&b64_data_cleaned)
            .context("Failed to decode base64 data")?;

        let mut cursor = Cursor::new(payload);
        let version = (cursor.read_u8()?, cursor.read_u8()?);
        if version != (0x01, 0x01) {
            return Err(anyhow!("Invalid payload version: {version:?}"));
        }

        let mut entries = Vec::new();
        while let Ok(entry_name_bytes) = read_bytes(&mut cursor) {
            let entry_name = String::from_utf8(entry_name_bytes)?;
            let name = entry_name.trim();
            if name.is_empty() && cursor.position() as usize == cursor.get_ref().len() {
                break;
            }
            let entry_len = cursor.read_u32::<BigEndian>()?;
            let mut entry_data = vec![0; entry_len as usize];
            cursor.read_exact(&mut entry_data)?;
            entries.push(Self::load(&entry_name, &entry_data)?);
        }

        Ok(entries)
    }

    fn load(entry_name: &str, data: &[u8]) -> Result<Self> {
        match entry_name {
            "BPMLOCK" => Ok(Self::BpmLock(BpmLock::load(data)?)),
            "COLOR" => Ok(Self::Color(Color::load(data)?)),
            "CUE" => Ok(Self::Cue(Cue::load(data)?)),
            "LOOP" => Ok(Self::Loop(Loop::load(data)?)),
            _ => Err(anyhow!("Unknown entry type: {entry_name}")),
        }
    }
}

impl BpmLock {
    fn load(data: &[u8]) -> Result<Self> {
        if data.len() != 1 {
            return Err(anyhow!("Invalid data length for BpmLock"));
        }
        Ok(Self { enabled: data[0] != 0 })
    }
}

impl Color {
    /// Create a new `RgbColor` from an RGB array [u8; 3].
    pub const fn new(bytes: [u8; 3]) -> Self {
        Self {
            r: bytes[0],
            g: bytes[1],
            b: bytes[2],
        }
    }

    /// Create a new `RgbColor` from an ARGB array [u8; 4].
    /// Ignores the alpha channel.
    pub const fn new_argb(bytes: [u8; 4]) -> Self {
        Self {
            r: bytes[1],
            g: bytes[2],
            b: bytes[3],
        }
    }

    #[inline]
    pub fn format(&self, text: &str) -> ColoredString {
        text.truecolor(self.r, self.g, self.b)
    }

    fn load(data: &[u8]) -> Result<Self> {
        if data.len() != 4 {
            return Err(anyhow!("Invalid data length for Color"));
        }
        Ok(Self {
            r: data[1],
            g: data[2],
            b: data[3],
        })
    }
}

impl Cue {
    /// | Offset |            Length | Raw Value     | Decoded   | Type                    | Description
    /// | ------ | ----------------- | ------------- | --------- | ----------------------- | -----------
    /// | `00`   |              `01` | `00`          |           |                         |
    /// | `01`   |              `01` | `00`          | 0         | `uint8_t`               | Index
    /// | `02`   |              `04` | `00 00 00 00` | 0         | `uint32_t`              | Position in ms
    /// | `06`   |              `01` | `00`          |           |                         |
    /// | `07`   |              `03` | `cc 00 00`    | `#CC0000` | 3-byte RGB value        | Color
    /// | `0a`   |              `02` | `00 00`       |           |                         |
    /// | `0c`   | `01` <= X <= `33` | `00`          | ``        | UTF-8 (null-terminated) | Name
    ///
    fn load(data: &[u8]) -> Result<Self> {
        if data.len() < 13 {
            return Err(anyhow!("Invalid data length for CueEntry"));
        }
        let mut cursor = Cursor::new(data);
        // Skip first byte
        cursor.set_position(1);
        let index = cursor.read_u8()?;
        let position = cursor.read_u32::<BigEndian>()?;
        cursor.set_position(cursor.position() + 1);
        let mut color = [0; 3];
        cursor.read_exact(&mut color)?;
        let color = Color::new(color);
        cursor.set_position(cursor.position() + 2);
        let mut name_bytes = Vec::new();
        cursor.read_to_end(&mut name_bytes)?;
        let name = str::from_utf8(&name_bytes)?.trim_end_matches('\x00').trim();
        let name = if name.is_empty() {
            super::format_position_timestamp(position)
        } else {
            name.to_string()
        };
        Ok(Self {
            index,
            position,
            color,
            name,
        })
    }
}

impl Loop {
    /// | Offset   |              Length | Raw Value     | Decoded   | Type                    | Description
    /// | -------- | ------------------- | ------------- | --------- | ----------------------- | -----------
    /// | `00`     |                `01` | `00`          |           |                         |
    /// | `01`     |                `01` | `00`          | 0         | `uint8_t`               | Index
    /// | `02`     |                `04` | `00 00 00 00` | 0         | `uint32_t`              | Start Position in milliseconds
    /// | `06`     |                `04` | `00 00 08 26` | 2086      | `uint32_t`              | End Position in milliseconds
    /// | `0a`     |                `04` | `ff ff ff ff` |           |                         |
    /// | `0e`     |                `04` | `00 27 aa e1` | `#27aae1` | 4-byte ARGB value       | Color
    /// | `12`     |                `01` | `00`          |           |                         |
    /// | `13`     |                `01` | `00`          | False     | `uint8_t` (boolean)     | Locked
    /// | `14`     | `01` <= X <= `7fec` | `00`          | ``        | UTF-8 (null-terminated) | Name
    ///
    fn load(data: &[u8]) -> Result<Self> {
        if data.len() < 15 {
            return Err(anyhow!("Invalid data length for Loop"));
        }
        let mut cursor = Cursor::new(data);
        cursor.set_position(1);
        let index = cursor.read_u8()?;
        let start_position = cursor.read_u32::<BigEndian>()?;
        let end_position = cursor.read_u32::<BigEndian>()?;
        cursor.set_position(cursor.position() + 4);
        let mut color = [0; 4];
        cursor.read_exact(&mut color)?;
        let color = Color::new_argb(color);
        cursor.set_position(cursor.position() + 1);
        let locked = cursor.read_u8()?;
        let locked = locked == 1;
        let mut name_bytes = Vec::new();
        cursor.read_to_end(&mut name_bytes)?;
        let name = str::from_utf8(&name_bytes)?.trim_end_matches('\x00').to_string();
        Ok(Self {
            index,
            start_position,
            end_position,
            color,
            locked,
            name,
        })
    }
}

impl Display for Markers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BpmLock(bpm_lock) => write!(f, "{bpm_lock}"),
            Self::Color(color) => write!(f, "{color}"),
            Self::Cue(cue) => write!(f, "{cue}"),
            Self::Loop(loop_var) => write!(f, "{loop_var}"),
        }
    }
}

impl Display for BpmLock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BPM Lock: {}", self.enabled)
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Color: {}",
            format!("[{},{},{}]", self.r, self.g, self.b).truecolor(self.r, self.g, self.b)
        )
    }
}

impl Display for Cue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let seconds = self.position as f32 * 0.001;
        let position = format!("{seconds:>7.3}s");
        let title = format!("Cue {}", self.index + 1);
        let text = self.color.format(&self.name);
        write!(f, "{title}: {text:<12} {position}")
    }
}

impl Display for Loop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = self.color.format(format!("Loop {}", self.index + 1).as_str());
        write!(
            f,
            "{msg}: {} [{:.2}s - {:.2}s] {}",
            if self.name.is_empty() {
                super::format_position_timestamp(self.start_position)
            } else {
                self.name.clone()
            },
            self.start_position as f32 * 0.001,
            self.end_position as f32 * 0.001,
            if self.locked { "locked" } else { "unlocked" }
        )
    }
}

/// Read bytes until null byte (0x00), excluding the terminator.
fn read_bytes<R: BufRead>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    reader.read_until(b'\x00', &mut buffer)?;
    if buffer.last() == Some(&b'\x00') {
        buffer.pop();
    }
    Ok(buffer)
}

#[cfg(test)]
mod test_color {
    use super::*;

    #[test]
    fn creates_color_from_rgb_array() {
        let color = Color::new([255, 128, 0]);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn creates_color_from_all_zeros() {
        let color = Color::new([0, 0, 0]);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn creates_color_from_all_max() {
        let color = Color::new([255, 255, 255]);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 255);
    }

    #[test]
    fn creates_color_ignoring_alpha_channel() {
        let color = Color::new_argb([0, 255, 128, 0]);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn ignores_nonzero_alpha() {
        let color = Color::new_argb([200, 10, 20, 30]);
        assert_eq!(color.r, 10);
        assert_eq!(color.g, 20);
        assert_eq!(color.b, 30);
    }

    #[test]
    fn creates_color_with_max_alpha() {
        let color = Color::new_argb([255, 100, 150, 200]);
        assert_eq!(color.r, 100);
        assert_eq!(color.g, 150);
        assert_eq!(color.b, 200);
    }

    #[test]
    fn displays_rgb_values() {
        let color = Color::new([255, 128, 0]);
        let display_output = color.to_string();
        assert!(
            display_output.contains("[255,128,0]"),
            "Expected display to contain '[255,128,0]', got: {display_output}"
        );
    }

    #[test]
    fn displays_color_prefix() {
        let color = Color::new([0, 0, 0]);
        let display_output = color.to_string();
        assert!(
            display_output.contains("Color:"),
            "Expected display to contain 'Color:', got: {display_output}"
        );
    }

    #[test]
    fn displays_all_zeros() {
        let color = Color::new([0, 0, 0]);
        let display_output = color.to_string();
        assert!(
            display_output.contains("[0,0,0]"),
            "Expected display to contain '[0,0,0]', got: {display_output}"
        );
    }

    #[test]
    fn displays_argb_constructed_color() {
        let color = Color::new_argb([128, 10, 20, 30]);
        let display_output = color.to_string();
        assert!(
            display_output.contains("[10,20,30]"),
            "Expected display to contain '[10,20,30]', got: {display_output}"
        );
    }

    #[test]
    fn formats_text_with_color() {
        let color = Color::new([255, 0, 0]);
        let colored_text = color.format("test");
        let text_string = colored_text.to_string();
        assert!(
            text_string.contains("test"),
            "Formatted text should contain 'test', got: {text_string}"
        );
    }

    #[test]
    fn loads_color_from_four_bytes() {
        let color = Color::load(&[0x00, 0xFF, 0x80, 0x40]).expect("Should parse color");
        assert_eq!(color.r, 0xFF);
        assert_eq!(color.g, 0x80);
        assert_eq!(color.b, 0x40);
    }

    #[test]
    fn rejects_too_short_data() {
        let result = Color::load(&[0x00, 0xFF, 0x80]);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_too_long_data() {
        let result = Color::load(&[0x00, 0xFF, 0x80, 0x40, 0x00]);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_cue_and_loop {
    use super::*;

    #[test]
    fn loads_cue_with_empty_name() {
        let cue_data: Vec<u8> = vec![
            0x00, // padding
            0x00, // index = 0
            0x00, 0x00, 0x03, 0xe8, // position = 1000 ms
            0x00, // padding
            0xCC, 0x00, 0x00, // RGB color
            0x00, 0x00, // padding
            0x00, // null-terminated empty name
        ];
        let cue = Cue::load(&cue_data).expect("Should parse valid cue data with empty name");
        assert_eq!(cue.index, 0);
        assert_eq!(cue.position, 1000);
        assert_eq!(cue.color.r, 0xCC);
        assert_eq!(cue.color.g, 0x00);
        assert_eq!(cue.color.b, 0x00);
        // Empty name should be replaced with a position timestamp
        assert!(
            !cue.name.is_empty(),
            "Name should not be empty (should be a position timestamp)"
        );
    }

    #[test]
    fn loads_cue_with_name() {
        let mut cue_data: Vec<u8> = vec![
            0x00, // padding
            0x02, // index = 2
            0x00, 0x00, 0x07, 0xD0, // position = 2000 ms
            0x00, // padding
            0x00, 0xFF, 0x00, // RGB color (green)
            0x00, 0x00, // padding
        ];
        // Add name "Drop" + null terminator
        cue_data.extend_from_slice(b"Drop\x00");

        let cue = Cue::load(&cue_data).expect("Should parse cue with named cue point");
        assert_eq!(cue.index, 2);
        assert_eq!(cue.position, 2000);
        assert_eq!(cue.color.r, 0x00);
        assert_eq!(cue.color.g, 0xFF);
        assert_eq!(cue.color.b, 0x00);
        assert_eq!(cue.name, "Drop");
    }

    #[test]
    fn rejects_too_short_data() {
        let short_data: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00];
        let result = Cue::load(&short_data);
        assert!(result.is_err(), "Should reject data shorter than 13 bytes");
    }

    #[test]
    fn loads_loop_with_empty_name() {
        let loop_data: Vec<u8> = vec![
            0x00, // padding
            0x00, // index = 0
            0x00, 0x00, 0x01, 0xF4, // start_position = 500 ms
            0x00, 0x00, 0x07, 0xD0, // end_position = 2000 ms
            0xFF, 0xFF, 0xFF, 0xFF, // padding
            0x00, 0x27, 0xAA, 0xE1, // ARGB color
            0x00, // padding
            0x01, // locked = true
            0x00, // null-terminated empty name
        ];
        let loop_entry = Loop::load(&loop_data).expect("Should parse valid loop data");
        assert_eq!(loop_entry.index, 0);
        assert_eq!(loop_entry.start_position, 500);
        assert_eq!(loop_entry.end_position, 2000);
        assert_eq!(loop_entry.color.r, 0x27);
        assert_eq!(loop_entry.color.g, 0xAA);
        assert_eq!(loop_entry.color.b, 0xE1);
        assert!(loop_entry.locked, "Loop should be locked");
    }

    #[test]
    fn rejects_too_short_loop_data() {
        let short_data: Vec<u8> = vec![0x00; 10];
        let result = Loop::load(&short_data);
        assert!(result.is_err(), "Should reject data shorter than 15 bytes");
    }

    #[test]
    fn displays_cue_marker() {
        let cue_data: Vec<u8> = vec![
            0x00, // padding
            0x00, // index = 0
            0x00, 0x00, 0x03, 0xe8, // position = 1000 ms
            0x00, // padding
            0xCC, 0x00, 0x00, // RGB color
            0x00, 0x00, // padding
            0x00, // null-terminated empty name
        ];
        let cue = Cue::load(&cue_data).expect("Should parse valid cue data");
        let marker = Markers::Cue(cue);
        let display_output = format!("{marker}");
        assert!(
            display_output.contains("Cue 1"),
            "Display should contain 'Cue 1', got: {display_output}"
        );
    }

    /// Helper to build a valid loop byte array with the given lock state.
    fn build_loop_data(locked: bool) -> Vec<u8> {
        vec![
            0x00, // padding
            0x00, // index = 0
            0x00,
            0x00,
            0x01,
            0xF4, // start_position = 500 ms
            0x00,
            0x00,
            0x07,
            0xD0, // end_position = 2000 ms
            0xFF,
            0xFF,
            0xFF,
            0xFF, // padding
            0x00,
            0x27,
            0xAA,
            0xE1, // ARGB color
            0x00, // padding
            u8::from(locked),
            0x00, // null-terminated empty name
        ]
    }

    #[test]
    fn displays_locked_loop() {
        let loop_data = build_loop_data(true);
        let loop_entry = Loop::load(&loop_data).expect("Should parse valid locked loop data");
        let marker = Markers::Loop(loop_entry);
        let display_output = format!("{marker}");
        assert!(
            display_output.contains("Loop 1"),
            "Display should contain 'Loop 1', got: {display_output}"
        );
        assert!(
            display_output.contains("locked"),
            "Display should contain 'locked', got: {display_output}"
        );
    }

    #[test]
    fn displays_unlocked_loop() {
        let loop_data = build_loop_data(false);
        let loop_entry = Loop::load(&loop_data).expect("Should parse valid unlocked loop data");
        let marker = Markers::Loop(loop_entry);
        let display_output = format!("{marker}");
        assert!(
            display_output.contains("Loop 1"),
            "Display should contain 'Loop 1', got: {display_output}"
        );
        assert!(
            display_output.contains("unlocked"),
            "Display should contain 'unlocked', got: {display_output}"
        );
    }
}

#[cfg(test)]
mod test_bpmlock {
    use super::*;

    #[test]
    fn displays_enabled_bpmlock() {
        let bpmlock = BpmLock { enabled: true };
        let marker = Markers::BpmLock(bpmlock);
        assert_eq!(marker.to_string(), "BPM Lock: true");
    }

    #[test]
    fn displays_disabled_bpmlock() {
        let bpmlock = BpmLock { enabled: false };
        let marker = Markers::BpmLock(bpmlock);
        assert_eq!(marker.to_string(), "BPM Lock: false");
    }

    #[test]
    fn loads_enabled_bpmlock() {
        let result = BpmLock::load(&[1]).expect("Should parse enabled BpmLock");
        assert!(result.enabled);
    }

    #[test]
    fn loads_disabled_bpmlock() {
        let result = BpmLock::load(&[0]).expect("Should parse disabled BpmLock");
        assert!(!result.enabled);
    }

    #[test]
    fn rejects_empty_data() {
        let result = BpmLock::load(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_too_long_data() {
        let result = BpmLock::load(&[1, 0]);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_markers_parsing {
    use super::*;
    use std::io::Cursor;
    use std::path::Path;

    #[test]
    fn parses_markers_from_extended_tags_file() {
        let test_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/extended_tags/Extended Tags - Song - 16-44.mp3");
        if !test_path.exists() {
            eprintln!("Test file not found, skipping: {}", test_path.display());
            return;
        }
        let tag = id3::Tag::read_from_path(&test_path).expect("Failed to read ID3 tags from test file");

        let mut found_markers = false;
        for frame in tag.frames() {
            if let Some(object) = frame.content().encapsulated_object()
                && object.description == "Serato Markers2"
            {
                let markers = Markers::parse(&object.data).expect("Should parse Serato Markers2 data");
                assert!(!markers.is_empty(), "Markers should not be empty");

                // Check that at least one known marker variant is present
                let has_known_variant = markers.iter().any(|marker| {
                    matches!(
                        marker,
                        Markers::BpmLock(_) | Markers::Color(_) | Markers::Cue(_) | Markers::Loop(_)
                    )
                });
                assert!(has_known_variant, "Should have at least one known marker variant");
                found_markers = true;
            }
        }
        assert!(
            found_markers,
            "Should have found Serato Markers2 GEOB frame in test file"
        );
    }

    #[test]
    fn reads_bytes_until_null() {
        let data = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0xFF];
        let mut cursor = Cursor::new(data);
        let result = read_bytes(&mut cursor).expect("Should read bytes until null terminator");
        assert_eq!(result, vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]);
    }

    #[test]
    fn reads_all_bytes_without_null() {
        let data = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];
        let mut cursor = Cursor::new(data);
        let result = read_bytes(&mut cursor).expect("Should read all bytes when no null is present");
        assert_eq!(result, vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]);
    }

    #[test]
    fn reads_empty_data() {
        let data: Vec<u8> = vec![];
        let mut cursor = Cursor::new(data);
        let result = read_bytes(&mut cursor).expect("Should handle empty data");
        assert!(result.is_empty(), "Should return empty vec for empty data");
    }
}
