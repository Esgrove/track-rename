use std::fmt;
use std::fmt::Display;

use anyhow::Result;
use anyhow::anyhow;

#[derive(Debug, Clone)]
pub enum BeatGridMarker {
    Terminal { position: f32, bpm: f32 },
    NonTerminal { position: f32, beats_till_next: u32 },
}

#[derive(Debug, Clone, Default)]
pub struct BeatGrid {
    pub num_markers: u32,
    pub markers: Vec<BeatGridMarker>,
}

impl BeatGrid {
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
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 6 {
            return Err(anyhow!("Data is too short to contain valid beatgrid information"));
        }

        let num_markers_bytes = [data[2], data[3], data[4], data[5]];
        let num_markers = u32::from_be_bytes(num_markers_bytes);
        if num_markers == 0 {
            return Ok(Self::default());
        }

        if data.len() < 11 {
            return Err(anyhow!("Data is too short to contain valid beatgrid information"));
        }

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

        Ok(Self { num_markers, markers })
    }
}

impl Display for BeatGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.num_markers == 0 {
            write!(f, "Empty")
        } else if self.num_markers == 1 {
            write!(f, "Beatgrid {}", self.markers[0])
        } else {
            writeln!(f, "Beatgrid ({}):", self.num_markers)?;
            for marker in &self.markers {
                write!(f, "  {marker}")?;
            }
            Ok(())
        }
    }
}

impl Display for BeatGridMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Terminal { position, bpm } => {
                write!(f, "{position:.3}s {bpm:.3} BPM")
            }
            Self::NonTerminal {
                position,
                beats_till_next,
            } => {
                writeln!(f, "{position:.3}s {beats_till_next} beats")
            }
        }
    }
}

#[cfg(test)]
mod test_beatgrid_parse {
    use super::*;

    #[test]
    fn data_too_short_returns_error() {
        let short_data: Vec<u8> = vec![0x01, 0x00, 0x00, 0x00, 0x00];
        assert!(BeatGrid::parse(&short_data).is_err());
    }

    #[test]
    fn empty_data_returns_error() {
        let empty_data: Vec<u8> = vec![];
        assert!(BeatGrid::parse(&empty_data).is_err());
    }

    #[test]
    fn zero_markers_returns_empty_beatgrid() {
        // Header (2 bytes) + num_markers = 0 (4 bytes)
        let data: Vec<u8> = vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let beatgrid = BeatGrid::parse(&data).expect("should parse zero-marker beatgrid");
        assert_eq!(beatgrid.num_markers, 0);
        assert!(beatgrid.markers.is_empty());
    }

    #[test]
    fn single_terminal_marker() {
        let position: f32 = 0.0;
        let bpm: f32 = 128.0;
        let position_bytes = position.to_be_bytes();
        let bpm_bytes = bpm.to_be_bytes();

        let mut data: Vec<u8> = vec![
            0x01, 0x00, // header
            0x00, 0x00, 0x00, 0x01, // num_markers = 1
        ];
        data.extend_from_slice(&position_bytes);
        data.extend_from_slice(&bpm_bytes);
        data.push(0x00); // footer byte

        let beatgrid = BeatGrid::parse(&data).expect("should parse single terminal marker");
        assert_eq!(beatgrid.num_markers, 1);
        assert_eq!(beatgrid.markers.len(), 1);

        match &beatgrid.markers[0] {
            BeatGridMarker::Terminal {
                position: parsed_position,
                bpm: parsed_bpm,
            } => {
                assert!(
                    (*parsed_position - position).abs() < f32::EPSILON,
                    "expected position {position}, got {parsed_position}"
                );
                assert!(
                    (*parsed_bpm - bpm).abs() < f32::EPSILON,
                    "expected bpm {bpm}, got {parsed_bpm}"
                );
            }
            BeatGridMarker::NonTerminal { .. } => {
                panic!("expected terminal marker, got non-terminal");
            }
        }
    }

    #[test]
    fn two_markers_non_terminal_then_terminal() {
        let first_position: f32 = 0.5;
        let beats_till_next: u32 = 4;
        let second_position: f32 = 10.0;
        let terminal_bpm: f32 = 120.0;

        let mut data: Vec<u8> = vec![
            0x01, 0x00, // header
            0x00, 0x00, 0x00, 0x02, // num_markers = 2
        ];
        data.extend_from_slice(&first_position.to_be_bytes());
        data.extend_from_slice(&beats_till_next.to_be_bytes());
        data.extend_from_slice(&second_position.to_be_bytes());
        data.extend_from_slice(&terminal_bpm.to_be_bytes());
        data.push(0x00); // footer byte

        let beatgrid = BeatGrid::parse(&data).expect("should parse two markers");
        assert_eq!(beatgrid.num_markers, 2);
        assert_eq!(beatgrid.markers.len(), 2);

        match &beatgrid.markers[0] {
            BeatGridMarker::NonTerminal {
                position,
                beats_till_next: parsed_beats,
            } => {
                assert!(
                    (*position - first_position).abs() < f32::EPSILON,
                    "expected position {first_position}, got {position}"
                );
                assert_eq!(*parsed_beats, beats_till_next);
            }
            BeatGridMarker::Terminal { .. } => {
                panic!("expected non-terminal marker for first entry");
            }
        }

        match &beatgrid.markers[1] {
            BeatGridMarker::Terminal { position, bpm } => {
                assert!(
                    (*position - second_position).abs() < f32::EPSILON,
                    "expected position {second_position}, got {position}"
                );
                assert!(
                    (*bpm - terminal_bpm).abs() < f32::EPSILON,
                    "expected bpm {terminal_bpm}, got {bpm}"
                );
            }
            BeatGridMarker::NonTerminal { .. } => {
                panic!("expected terminal marker for last entry");
            }
        }
    }

    #[test]
    fn marker_data_truncated_returns_error() {
        // Header says 1 marker but data is too short to contain it
        let data: Vec<u8> = vec![
            0x01, 0x00, // header
            0x00, 0x00, 0x00, 0x01, // num_markers = 1
            0x00, 0x00, // only 2 bytes of marker data (need 8 + footer)
        ];
        assert!(BeatGrid::parse(&data).is_err());
    }
}

#[cfg(test)]
mod test_beatgrid_display {
    use super::*;

    #[test]
    fn empty_beatgrid_displays_empty() {
        let beatgrid = BeatGrid::default();
        let display_output = format!("{beatgrid}");
        assert_eq!(display_output, "Empty");
    }

    #[test]
    fn single_terminal_marker_displays_bpm() {
        let position: f32 = 0.0;
        let bpm: f32 = 128.0;
        let position_bytes = position.to_be_bytes();
        let bpm_bytes = bpm.to_be_bytes();

        let mut data: Vec<u8> = vec![
            0x01, 0x00, // header
            0x00, 0x00, 0x00, 0x01, // num_markers = 1
        ];
        data.extend_from_slice(&position_bytes);
        data.extend_from_slice(&bpm_bytes);
        data.push(0x00); // footer

        let beatgrid = BeatGrid::parse(&data).expect("should parse single marker");
        let display_output = format!("{beatgrid}");
        assert_eq!(display_output, "Beatgrid 0.000s 128.000 BPM");
    }

    #[test]
    fn multiple_markers_display_with_count() {
        let first_position: f32 = 0.5;
        let beats_till_next: u32 = 4;
        let second_position: f32 = 10.0;
        let terminal_bpm: f32 = 120.0;

        let mut data: Vec<u8> = vec![
            0x01, 0x00, // header
            0x00, 0x00, 0x00, 0x02, // num_markers = 2
        ];
        data.extend_from_slice(&first_position.to_be_bytes());
        data.extend_from_slice(&beats_till_next.to_be_bytes());
        data.extend_from_slice(&second_position.to_be_bytes());
        data.extend_from_slice(&terminal_bpm.to_be_bytes());
        data.push(0x00); // footer

        let beatgrid = BeatGrid::parse(&data).expect("should parse two markers");
        let display_output = format!("{beatgrid}");
        assert!(
            display_output.contains("Beatgrid (2):"),
            "expected marker count in output, got: {display_output}"
        );
        assert!(
            display_output.contains("0.500s 4 beats"),
            "expected non-terminal marker in output, got: {display_output}"
        );
        assert!(
            display_output.contains("10.000s 120.000 BPM"),
            "expected terminal marker in output, got: {display_output}"
        );
    }

    #[test]
    fn terminal_marker_format() {
        let marker = BeatGridMarker::Terminal {
            position: 1.234,
            bpm: 140.0,
        };
        let display_output = format!("{marker}");
        assert_eq!(display_output, "1.234s 140.000 BPM");
    }

    #[test]
    fn non_terminal_marker_format() {
        let marker = BeatGridMarker::NonTerminal {
            position: 0.500,
            beats_till_next: 8,
        };
        let display_output = format!("{marker}");
        assert_eq!(display_output, "0.500s 8 beats\n");
    }
}
