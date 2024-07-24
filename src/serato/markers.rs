use std::fmt::Display;
use std::io::{Cursor, Read};
use std::{fmt, io, str};

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use byteorder::{BigEndian, ReadBytesExt};
use colored::{ColoredString, Colorize};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Markers {
    BpmLock(BpmLock),
    Color(Color),
    Cue(Cue),
    Loop(Loop),
}

#[derive(Debug, Clone)]
/// Boolean for BPM lock status.
/// True means lock is enabled.
/// https://support.serato.com/hc/en-us/articles/235214887-Lock-Beatgrids
pub struct BpmLock {
    enabled: bool,
}

#[derive(Debug, Clone)]
/// RGB colour.
/// Used for track, cues, and loops.
pub struct Color {
    r: u8,
    b: u8,
    g: u8,
}

#[derive(Debug, Clone)]
/// A cue point.
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

#[derive(Debug, Clone)]
/// Saved loop.
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
    pub fn parse(data: &[u8]) -> Result<Vec<Markers>> {
        let b64data_start = 2;
        let b64data_end = data
            .iter()
            .position(|&x| x == b'\x00')
            .ok_or_else(|| anyhow!("No null terminator found"))?;
        let b64data = &data[b64data_start..b64data_end];

        // Remove linefeed characters
        let b64data: Vec<u8> = b64data.iter().cloned().filter(|&x| x != b'\n').collect();

        // Calculate padding
        let padding = match b64data.len() % 4 {
            1 => b"A==".to_vec(),
            2 => b"==".to_vec(),
            3 => b"=".to_vec(),
            _ => Vec::new(),
        };

        // Concatenate base64 data with padding
        let mut b64data_padded = b64data.clone();
        b64data_padded.extend_from_slice(&padding);

        let payload = general_purpose::STANDARD
            .decode(&b64data_padded)
            .context("Failed to decode base64 data")?;

        let mut cursor = Cursor::new(payload);
        let version = (cursor.read_u8()?, cursor.read_u8()?);
        if version != (0x01, 0x01) {
            return Err(anyhow!("Invalid payload version: {:?}", version));
        }

        let mut entries = Vec::new();
        while let Ok(entry_name_bytes) = read_bytes(&mut cursor) {
            let entry_name = String::from_utf8(entry_name_bytes)?;
            let name = entry_name.trim();
            if name.is_empty() {
                break;
            }
            let entry_len = cursor.read_u32::<BigEndian>()?;
            let mut entry_data = vec![0; entry_len as usize];
            cursor.read_exact(&mut entry_data)?;
            entries.push(Markers::load(&entry_name, &entry_data)?);
        }

        Ok(entries)
    }

    fn load(entry_name: &str, data: &[u8]) -> Result<Markers> {
        match entry_name {
            "BPMLOCK" => Ok(Markers::BpmLock(BpmLock::load(data)?)),
            "COLOR" => Ok(Markers::Color(Color::load(data)?)),
            "CUE" => Ok(Markers::Cue(Cue::load(data)?)),
            "LOOP" => Ok(Markers::Loop(Loop::load(data)?)),
            _ => Err(anyhow!("Unknown entry type: {}", entry_name)),
        }
    }
}

impl BpmLock {
    fn load(data: &[u8]) -> Result<BpmLock> {
        if data.len() != 1 {
            return Err(anyhow!("Invalid data length for BpmLock"));
        }
        Ok(BpmLock { enabled: data[0] != 0 })
    }
}

impl Color {
    /// Create a new RgbColor from an array [u8; 3].
    pub fn new(bytes: [u8; 3]) -> Self {
        Color {
            r: bytes[0],
            g: bytes[1],
            b: bytes[2],
        }
    }

    pub fn new_argb(bytes: [u8; 4]) -> Self {
        Color {
            r: bytes[1],
            g: bytes[2],
            b: bytes[3],
        }
    }

    #[inline]
    pub fn format(&self, text: &str) -> ColoredString {
        text.truecolor(self.r, self.g, self.b)
    }

    fn load(data: &[u8]) -> Result<Color> {
        if data.len() != 4 {
            return Err(anyhow!("Invalid data length for Color"));
        }
        Ok(Color {
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
    fn load(data: &[u8]) -> Result<Cue> {
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
        let name = str::from_utf8(&name_bytes)?.trim_end_matches('\x00').to_string();
        Ok(Cue {
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
    fn load(data: &[u8]) -> Result<Loop> {
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
        Ok(Loop {
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
            Markers::BpmLock(bpm_lock) => write!(f, "{}", bpm_lock),
            Markers::Color(color) => write!(f, "{}", color),
            Markers::Cue(cue) => write!(f, "{}", cue),
            Markers::Loop(loop_var) => write!(f, "{}", loop_var),
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
        let position = format!("{seconds:.3}s");
        let msg = self.color.format(format!("Cue {}", self.index + 1).as_str());
        if self.name.is_empty() {
            write!(f, "{msg}: {position}")
        } else {
            write!(f, "{msg}: {} {}", self.name, position)
        }
    }
}

impl Display for Loop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = self.color.format(format!("Loop {}", self.index + 1).as_str());
        write!(
            f,
            "{msg}: {} [{:.2}s - {:.2}s] {}",
            if self.name.is_empty() {
                super::format_duration(self.start_position)
            } else {
                self.name.clone()
            },
            self.start_position as f32 * 0.001,
            self.end_position as f32 * 0.001,
            if self.locked { "locked" } else { "unlocked" }
        )
    }
}

fn read_bytes<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    for byte in reader.bytes() {
        let byte = byte?;
        if byte == b'\x00' {
            break;
        }
        bytes.push(byte);
    }
    Ok(bytes)
}
