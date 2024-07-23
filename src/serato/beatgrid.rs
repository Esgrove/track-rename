use anyhow::anyhow;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum BeatGridMarker {
    Terminal { position: f32, bpm: f32 },
    NonTerminal { position: f32, beats_till_next: u32 },
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
pub fn parse_beatgrid_data(data: &[u8]) -> anyhow::Result<BeatGrid> {
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

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct BeatGrid {
    pub num_markers: u32,
    pub markers: Vec<BeatGridMarker>,
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
