use std::fmt::Display;
use std::io::{self, Cursor, Read};
use std::str::FromStr;
use std::{fmt, str};

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use byteorder::{BigEndian, ReadBytesExt};
use colored::Colorize;
use id3::Tag;

#[derive(Debug, Clone, Default)]
struct SeratoData {
    pub analysis: AnalysisVersion,
    pub beatgrid: BeatGrid,
    pub markers: Vec<Markers>,
}

#[derive(Debug, Clone, Default)]
struct AnalysisVersion {
    pub major_version: u8,
    pub minor_version: u8,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
struct BeatGrid {
    pub num_markers: u32,
    pub markers: Vec<BeatGridMarker>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum BeatGridMarker {
    Terminal { position: f32, bpm: f32 },
    NonTerminal { position: f32, beats_till_next: u32 },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Markers {
    BpmLock(BpmLock),
    Color(Color),
    Cue(Cue),
    Loop(Loop),
}

impl Markers {
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

#[derive(Debug, Clone)]
struct BpmLock {
    enabled: bool,
}

impl Display for BpmLock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BPM Lock: {}", self.enabled)
    }
}

impl Display for BeatGridMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BeatGridMarker::Terminal { position, bpm } => {
                write!(f, "{:.3}s {} BPM", position, bpm)
            }
            BeatGridMarker::NonTerminal {
                position,
                beats_till_next,
            } => {
                write!(f, "{:.3}s {} beats", position, beats_till_next)
            }
        }
    }
}

impl BpmLock {
    fn load(data: &[u8]) -> Result<BpmLock> {
        if data.len() != 1 {
            return Err(anyhow!("Invalid data length for BpmLockEntry"));
        }
        let lock = BpmLock { enabled: data[0] != 0 };
        Ok(lock)
    }
}

impl Display for AnalysisVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Version: {}.{}", self.major_version, self.minor_version)
    }
}

#[derive(Debug, Clone)]
/// RGB track colour.
struct Color {
    r: u8,
    b: u8,
    g: u8,
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

    fn load(data: &[u8]) -> Result<Color> {
        if data.len() != 4 {
            return Err(anyhow!("Invalid data length for Color"));
        }
        let color = Color {
            r: data[1],
            g: data[2],
            b: data[3],
        };
        Ok(color)
    }
}

#[derive(Debug, Clone)]
/// Cue point.
///
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
struct Cue {
    /// Cue number
    index: u8,
    /// RGB Color
    color: Color,
    /// Name
    name: String,
    /// Position in milliseconds
    position: u32,
}

impl Cue {
    fn load(data: &[u8]) -> Result<Cue> {
        if data.len() < 13 {
            return Err(anyhow!("Invalid data length for CueEntry"));
        }
        let mut cursor = Cursor::new(data);
        let _ = cursor.read_u8()?;
        let index = cursor.read_u8()?;
        let position = cursor.read_u32::<BigEndian>()?;
        let _ = cursor.read_u8()?;
        let mut color = [0; 3];
        cursor.read_exact(&mut color)?;
        let color = Color::new(color);
        let mut field6 = [0; 2];
        cursor.read_exact(&mut field6)?;
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

impl Display for Cue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let seconds = self.position as f32 * 0.001;
        let position = format!("{seconds:.3}s");
        let msg = format!("Cue {}", self.index + 1).truecolor(self.color.r, self.color.g, self.color.b);
        if self.name.is_empty() {
            write!(f, "{msg}: {position}")
        } else {
            write!(f, "{msg}: {} {}", self.name, position)
        }
    }
}

#[derive(Debug, Clone)]
/// Saved loop.
///
/// | Offset   |              Length | Raw Value     | Decoded   | Type                    | Description
/// | -------- | ------------------- | ------------- | --------- | ----------------------- | -----------
/// | `00`     |                `01` | `00`          |           |                         |
/// | `01`     |                `01` | `00`          | 0         | `uint8_t`               | Index
/// | `02`     |                `04` | `00 00 00 00` | 0         | `uint32_t`              | Start Position in milliseconds
/// | `06`     |                `04` | `00 00 08 26` | 2086      | `uint32_t`              | End Position in milliseconds
/// | `0a`     |                `04` | `ff ff ff ff` |           |                         |
/// | `0e`     |                `04` | `00 27 aa e1` | `#27aae1` | 4-byte ARGB value       | Color
/// | `12`     |                `03` | `00`          |           |                         |
/// | `13`     |                `01` | `00`          | False     | `uint8_t` (boolean)     | Locked
/// | `14`     | `01` <= X <= `7fec` | `00`          | ``        | UTF-8 (null-terminated) | Name
///
struct Loop {
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

impl Loop {
    fn load(data: &[u8]) -> Result<Loop> {
        if data.len() < 15 {
            return Err(anyhow!("Invalid data length for Loop"));
        }
        let mut cursor = Cursor::new(data);
        let _ = cursor.read_u8()?;
        let index = cursor.read_u8()?;
        let start_position = cursor.read_u32::<BigEndian>()?;
        let end_position = cursor.read_u32::<BigEndian>()?;
        let mut field5 = [0; 4];
        cursor.read_exact(&mut field5)?;
        let mut field6 = [0; 4];
        cursor.read_exact(&mut field6)?;
        let mut color = [0; 4];
        cursor.read_exact(&mut color)?;
        let color = Color::new_argb(color);
        let locked = cursor.read_u8()? != 0;
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

impl Display for Loop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = format!("Loop {}", self.index + 1).truecolor(self.color.r, self.color.g, self.color.b);
        write!(
            f,
            "{msg}: {} [{:.2}s - {:.2}s] {}",
            self.name,
            self.start_position as f32 * 0.001,
            self.end_position as f32 * 0.001,
            if self.locked { "locked" } else { "unlocked" }
        )
    }
}

impl Display for BeatGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.num_markers == 1 {
            write!(f, "Beatgrid {}", self.markers[0])
        } else {
            writeln!(f, "Beatgrid ({}):", self.num_markers)?;
            for marker in self.markers.iter() {
                write!(f, "  {}", marker)?;
            }
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SeratoTag {
    Analysis,
    Autotags,
    BeatGrid,
    Markers,
    Overview,
}

impl FromStr for SeratoTag {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Serato Analysis" => Ok(SeratoTag::Analysis),
            "Serato Autotags" => Ok(SeratoTag::Autotags),
            "Serato BeatGrid" => Ok(SeratoTag::BeatGrid),
            "Serato Markers2" => Ok(SeratoTag::Markers),
            "Serato Overview" => Ok(SeratoTag::Overview),
            _ => Err(anyhow!("Unknown tag description: {}", s)),
        }
    }
}

impl Display for SeratoTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SeratoTag::Analysis => {
                    "SeratoAnalysis"
                }
                SeratoTag::Autotags => {
                    "SeratoAutotags"
                }
                SeratoTag::BeatGrid => {
                    "SeratoBeatGrid"
                }
                SeratoTag::Markers => {
                    "SeratoMarkers"
                }
                SeratoTag::Overview => {
                    "SeratoOverview"
                }
            }
        )
    }
}

pub fn print_serato_tags(file_tags: &Tag) {
    println!("{}", "Serato tags:".cyan());
    let mut serato_data = SeratoData::default();
    for frame in file_tags.frames() {
        if let Some(object) = frame.content().encapsulated_object() {
            if let Ok(tag) = SeratoTag::from_str(&object.description) {
                match tag {
                    SeratoTag::Analysis => match parse_analysis_data(&object.data) {
                        Ok(data) => {
                            println!("{}", data);
                            serato_data.analysis = data;
                        }
                        Err(error) => {
                            eprintln!("{error}")
                        }
                    },
                    SeratoTag::Autotags => {}
                    SeratoTag::BeatGrid => match parse_beatgrid_data(&object.data) {
                        Ok(data) => {
                            println!("{}", data);
                            serato_data.beatgrid = data;
                        }
                        Err(error) => {
                            eprintln!("{error}")
                        }
                    },
                    SeratoTag::Markers => match parse_markers(&object.data) {
                        Ok(data) => {
                            for marker in data.iter() {
                                println!("{}", marker);
                            }

                            serato_data.markers = data;
                        }
                        Err(error) => {
                            eprintln!("{error}")
                        }
                    },
                    SeratoTag::Overview => {}
                }
            }
        }
    }
}

/// Parse analysis tag.
/// Contains the Serato analysis version number (*here:* 2.1).
///
/// | Offset | Length | Raw Value | Decoded Value | Type            | Description
/// | ------ | ------ | --------- | ------------- | --------------- | -----------
/// |   `00` |   `01` |      `02` |           `2` | `unsigned char` | Major Version
/// |   `01` |   `01` |      `01` |           `1` | `unsigned char` | Minor Version
///
fn parse_analysis_data(data: &[u8]) -> anyhow::Result<AnalysisVersion> {
    if data.len() >= 2 {
        let major_version = data[0];
        let minor_version = data[1];

        Ok(AnalysisVersion {
            major_version,
            minor_version,
        })
    } else {
        Err(anyhow!("Data is too short to contain version information"))
    }
}

/// Parse beatgrid tag.
/// The tag data consists of a header followed by zero or more beatgrid markers and a single footer byte.
///
/// ## Header
///
/// | Offset | Length | Raw Value     | Value | Type       | Description
/// | ------ | ------ | ------------- | ----- | ---------- | -----------
/// |   `00` |   `02` | `01 00`       |       |            |
/// |   `02` |   `04` | `00 00 00 01` |     1 | `uint32_t` | Number of Markers
///
/// ## Beatgrid Markers
///
/// There are two different types of beatgrid markers: terminal and non-terminal.
///
/// ### Terminal
///
/// The last beatgrid marker always has to be a terminal one.
/// This is also the case if the tag only contains a single beatgrid marker.
///
/// | Offset | Length | Raw Value      | Type               | Description
/// | ------ | ------ | -------------- | ------------------ | -----------
/// |   `00` |   `04` | `3e 9c 28 38`  | `float` (binary32) | Position
/// |   `04` |   `04` | `42 e6 00 00`  | `float` (binary32) | BPM
///
/// ### Non-terminal
///
/// All beatgrid markers before the last one are non-terminal beatgrid markers.
/// Instead of a floating point BPM value, they contain the number of beats till the next marker as an integer.
///
/// | Offset | Length | Raw Value     | Type               | Description
/// | ------ | ------ | ------------- | ------------------ | -----------
/// |   `00` |   `04` |               | `float` (binary32) | Position
/// |   `04` |   `04` | `00 00 00 04` | `uint32_t`         | Beats till next marker
///
fn parse_beatgrid_data(data: &[u8]) -> anyhow::Result<BeatGrid> {
    if data.len() < 11 {
        return Err(anyhow!("Data is too short to contain valid beatgrid information"));
    }

    let num_markers_bytes = [data[2], data[3], data[4], data[5]];
    let num_markers = u32::from_be_bytes(num_markers_bytes);

    let mut markers = Vec::new();
    let mut offset = 6;

    for _ in 0..num_markers {
        if offset + 8 > data.len() {
            return Err(anyhow!("Data is too short to contain all beatgrid markers"));
        }

        let position_bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
        let position = f32::from_be_bytes(position_bytes);
        let next_data = &data[offset + 4..offset + 8];
        let marker_bytes = [next_data[0], next_data[1], next_data[2], next_data[3]];

        let marker = if offset + 8 == data.len() - 1 {
            let bpm = f32::from_be_bytes(marker_bytes);
            BeatGridMarker::Terminal { position, bpm }
        } else {
            let beats_till_next = u32::from_be_bytes(marker_bytes);
            BeatGridMarker::NonTerminal {
                position,
                beats_till_next,
            }
        };

        markers.push(marker);
        offset += 8;
    }

    Ok(BeatGrid { num_markers, markers })
}

fn parse_markers(data: &[u8]) -> Result<Vec<Markers>> {
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

#[allow(dead_code)]
fn format_as_byte_string(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<String>>()
        .join(" ")
}

#[allow(dead_code)]
fn hexdump(buffer: &[u8], ascii: bool) -> String {
    let mut offset = 0;
    let mut result = String::new();
    while offset < buffer.len() {
        let end = std::cmp::min(buffer.len(), offset + 16);
        let line = &buffer[offset..end];

        // Format the offset
        result.push_str(&format!("    {:08x}  ", offset));

        // Format the hexadecimal values
        for byte in line {
            result.push_str(&format!("{:02x} ", byte));
        }

        // Add padding if the line is less than 16 bytes
        if line.len() < 16 {
            for _ in 0..(16 - line.len()) {
                result.push_str("   ");
            }
        }

        if ascii {
            // Format the ASCII representation
            result.push_str(" |");
            for &byte in line {
                if byte.is_ascii_graphic() || byte == b' ' {
                    result.push(byte as char);
                } else {
                    result.push('.');
                }
            }
            result.push('|');
        }
        result.push('\n');

        offset += 16;
    }
    result
}
