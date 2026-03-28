pub mod serato_crate;

mod analysis;
mod autotags;
mod beatgrid;
mod markers;
mod overview;

use std::fmt::Display;
use std::fmt::Write as _;
use std::str::FromStr;

use anyhow::{Result, anyhow};
use colored::Colorize;
use id3::Tag;

use crate::output::print_error;
use crate::serato::analysis::AnalysisVersion;
use crate::serato::autotags::AutoTags;
use crate::serato::beatgrid::BeatGrid;
use crate::serato::markers::Markers;
use crate::serato::overview::Overview;

pub use crate::serato::serato_crate::SeratoCrate;

/// Serato tag types.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SeratoTag {
    /// Serato analysis version number
    Analysis,
    /// BPM, auto gain, and manual gain values
    Autotags,
    /// Beatgrid markers
    BeatGrid,
    /// Cue points, loops, track color, and BPM lock status
    Markers,
    /// Waveform overview data.
    Overview,
}

/// Contains all Serato custom tag data in the file.
#[derive(Debug, Clone, Default)]
pub struct SeratoData {
    pub analysis: Option<AnalysisVersion>,
    pub autotags: Option<AutoTags>,
    pub beatgrid: Option<BeatGrid>,
    pub markers: Vec<Markers>,
    pub overview: Option<Overview>,
}

impl SeratoData {
    /// Parse Serato custom tags from tag data.
    #[must_use]
    pub fn parse(file_tags: &Tag) -> Option<Self> {
        let mut serato_data = Self::default();
        let mut parsed_any = false;

        for frame in file_tags.frames() {
            if let Some(object) = frame.content().encapsulated_object()
                && let Ok(tag) = SeratoTag::from_str(&object.description)
            {
                match tag {
                    SeratoTag::Analysis => match AnalysisVersion::parse(&object.data) {
                        Ok(data) => {
                            serato_data.analysis = Some(data);
                            parsed_any = true;
                        }
                        Err(error) => print_error(&error.to_string()),
                    },
                    SeratoTag::Autotags => match AutoTags::parse(&object.data) {
                        Ok(data) => {
                            serato_data.autotags = Some(data);
                            parsed_any = true;
                        }
                        Err(error) => print_error(&error.to_string()),
                    },
                    SeratoTag::BeatGrid => match BeatGrid::parse(&object.data) {
                        Ok(data) => {
                            serato_data.beatgrid = Some(data);
                            parsed_any = true;
                        }
                        Err(error) => print_error(&error.to_string()),
                    },
                    SeratoTag::Markers => match Markers::parse(&object.data) {
                        Ok(data) => {
                            serato_data.markers = data;
                            parsed_any = true;
                        }
                        Err(error) => print_error(&error.to_string()),
                    },
                    SeratoTag::Overview => match Overview::parse(&object.data) {
                        Ok(data) => {
                            serato_data.overview = Some(data);
                            parsed_any = true;
                        }
                        Err(error) => print_error(&error.to_string()),
                    },
                }
            }
        }
        if parsed_any { Some(serato_data) } else { None }
    }
}

impl FromStr for SeratoTag {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Serato Analysis" => Ok(Self::Analysis),
            "Serato Autotags" => Ok(Self::Autotags),
            "Serato BeatGrid" => Ok(Self::BeatGrid),
            "Serato Markers2" => Ok(Self::Markers),
            "Serato Overview" => Ok(Self::Overview),
            _ => Err(anyhow!("Unknown tag description: {s}")),
        }
    }
}

impl Display for SeratoTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Analysis => "SeratoAnalysis",
                Self::Autotags => "SeratoAutotags",
                Self::BeatGrid => "SeratoBeatGrid",
                Self::Markers => "SeratoMarkers",
                Self::Overview => "SeratoOverview",
            }
        )
    }
}

impl Display for SeratoData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}", "Serato tags:".cyan().bold())?;
        if let Some(autotags) = &self.autotags {
            writeln!(f, "{}: {}", SeratoTag::Autotags, autotags)?;
        } else {
            writeln!(f, "{}: None", SeratoTag::Autotags)?;
        }
        if let Some(analysis) = &self.analysis {
            writeln!(f, "{}: {}", SeratoTag::Analysis, analysis)?;
        } else {
            writeln!(f, "{}: None", SeratoTag::Analysis)?;
        }
        if let Some(beatgrid) = &self.beatgrid {
            writeln!(f, "{}: {}", SeratoTag::BeatGrid, beatgrid)?;
        } else {
            writeln!(f, "{}: None", SeratoTag::BeatGrid)?;
        }
        if let Some(overview) = &self.overview {
            write!(f, "{}:\n{}", SeratoTag::Overview, overview)?;
        } else {
            writeln!(f, "{}: None", SeratoTag::Overview)?;
        }
        if self.markers.is_empty() {
            writeln!(f, "{}: None", SeratoTag::Markers)?;
        } else {
            writeln!(f, "{}:", SeratoTag::Markers)?;
            for marker in &self.markers {
                writeln!(f, "  {marker}")?;
            }
        }

        Ok(())
    }
}

/// Parse and print Serato tag data if any is present.
pub fn print_serato_tags(file_tags: &Tag) {
    if let Some(serato_data) = SeratoData::parse(file_tags) {
        print!("{serato_data}");
    }
}

/// Format duration in milliseconds as `MM:SS:T` to match Serato.
fn format_position_timestamp(position_in_ms: u32) -> String {
    let minutes = position_in_ms / 60000;
    let seconds = (position_in_ms % 60000) / 1000;
    let tenths = (f64::from(position_in_ms % 1000) / 100.0).round();

    format!("{minutes:02}:{seconds:02}.{tenths}")
}

/// Debug function to print bytes as hexadecimal
#[allow(dead_code)]
fn format_as_byte_string(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Debug function to print formatted hexdump
#[allow(dead_code)]
fn hexdump(buffer: &[u8], ascii: bool) -> String {
    let mut offset = 0;
    let mut result = String::new();
    while offset < buffer.len() {
        let end = std::cmp::min(buffer.len(), offset + 16);
        let line = &buffer[offset..end];

        // Format the offset
        let _ = write!(result, "    {offset:08x}  ");

        // Format the hexadecimal values
        for byte in line {
            let _ = write!(result, "{byte:02x} ");
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

#[cfg(test)]
mod test_serato_tag_from_str {
    use super::*;

    #[test]
    fn parses_analysis() {
        let tag = SeratoTag::from_str("Serato Analysis").expect("Should parse 'Serato Analysis'");
        assert_eq!(tag, SeratoTag::Analysis);
    }

    #[test]
    fn parses_autotags() {
        let tag = SeratoTag::from_str("Serato Autotags").expect("Should parse 'Serato Autotags'");
        assert_eq!(tag, SeratoTag::Autotags);
    }

    #[test]
    fn parses_beatgrid() {
        let tag = SeratoTag::from_str("Serato BeatGrid").expect("Should parse 'Serato BeatGrid'");
        assert_eq!(tag, SeratoTag::BeatGrid);
    }

    #[test]
    fn parses_markers() {
        let tag = SeratoTag::from_str("Serato Markers2").expect("Should parse 'Serato Markers2'");
        assert_eq!(tag, SeratoTag::Markers);
    }

    #[test]
    fn parses_overview() {
        let tag = SeratoTag::from_str("Serato Overview").expect("Should parse 'Serato Overview'");
        assert_eq!(tag, SeratoTag::Overview);
    }

    #[test]
    fn rejects_unknown_tag() {
        let result = SeratoTag::from_str("Unknown Tag");
        assert!(result.is_err(), "Should reject unknown tag description");
    }

    #[test]
    fn rejects_empty_string() {
        let result = SeratoTag::from_str("");
        assert!(result.is_err(), "Should reject empty string");
    }
}

#[cfg(test)]
mod test_serato_tag_display {
    use super::*;

    #[test]
    fn displays_analysis() {
        assert_eq!(SeratoTag::Analysis.to_string(), "SeratoAnalysis");
    }

    #[test]
    fn displays_autotags() {
        assert_eq!(SeratoTag::Autotags.to_string(), "SeratoAutotags");
    }

    #[test]
    fn displays_beatgrid() {
        assert_eq!(SeratoTag::BeatGrid.to_string(), "SeratoBeatGrid");
    }

    #[test]
    fn displays_markers() {
        assert_eq!(SeratoTag::Markers.to_string(), "SeratoMarkers");
    }

    #[test]
    fn displays_overview() {
        assert_eq!(SeratoTag::Overview.to_string(), "SeratoOverview");
    }
}

#[cfg(test)]
mod test_format_position_timestamp {
    use super::*;

    #[test]
    fn formats_zero_milliseconds() {
        assert_eq!(format_position_timestamp(0), "00:00.0");
    }

    #[test]
    fn formats_one_second() {
        assert_eq!(format_position_timestamp(1000), "00:01.0");
    }

    #[test]
    fn formats_one_minute() {
        assert_eq!(format_position_timestamp(60000), "01:00.0");
    }

    #[test]
    fn formats_one_minute_one_second_five_tenths() {
        assert_eq!(format_position_timestamp(61500), "01:01.5");
    }

    #[test]
    fn formats_two_minutes_five_seconds_three_tenths() {
        assert_eq!(format_position_timestamp(125_300), "02:05.3");
    }

    #[test]
    fn formats_large_value() {
        // 10 minutes exactly
        assert_eq!(format_position_timestamp(600_000), "10:00.0");
    }
}

#[cfg(test)]
mod test_serato_data_parse {
    use super::*;
    use std::path::Path;

    /// Return the path to the extended tags MP3 test file.
    fn extended_tags_mp3_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/extended_tags/Extended Tags - Song - 16-44.mp3")
    }

    #[test]
    fn parses_serato_data_from_real_file() {
        let test_path = extended_tags_mp3_path();
        if !test_path.exists() {
            eprintln!("Test file not found, skipping: {}", test_path.display());
            return;
        }
        let tag = id3::Tag::read_from_path(&test_path).expect("Failed to read ID3 tags from test file");
        let serato_data = SeratoData::parse(&tag);
        assert!(
            serato_data.is_some(),
            "SeratoData::parse should return Some for file with Serato tags"
        );

        let serato_data = serato_data.expect("SeratoData should be present");
        assert!(serato_data.analysis.is_some(), "Analysis should be present");
        assert!(serato_data.autotags.is_some(), "Autotags should be present");

        let autotags = serato_data.autotags.as_ref().expect("Autotags should be present");
        assert!(
            autotags.bpm > 0.0,
            "BPM should be greater than zero, got: {}",
            autotags.bpm
        );

        assert!(serato_data.beatgrid.is_some(), "BeatGrid should be present");
        assert!(serato_data.overview.is_some(), "Overview should be present");
        assert!(!serato_data.markers.is_empty(), "Markers should not be empty");
    }
}

#[cfg(test)]
mod test_serato_data_parse_no_serato {
    use super::*;
    use std::path::Path;

    #[test]
    fn returns_none_for_file_without_serato_data() {
        let test_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/basic_tags/Basic Tags - Song - 16-44.mp3");
        if !test_path.exists() {
            eprintln!("Test file not found, skipping: {}", test_path.display());
            return;
        }
        let tag = id3::Tag::read_from_path(&test_path).expect("Failed to read ID3 tags from test file");
        let serato_data = SeratoData::parse(&tag);
        assert!(
            serato_data.is_none(),
            "SeratoData::parse should return None for file without Serato tags"
        );
    }
}

#[cfg(test)]
mod test_serato_data_display {
    use super::*;
    use std::path::Path;

    /// Return the path to the extended tags MP3 test file.
    fn extended_tags_mp3_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/files/extended_tags/Extended Tags - Song - 16-44.mp3")
    }

    #[test]
    fn display_contains_all_tag_sections() {
        let test_path = extended_tags_mp3_path();
        if !test_path.exists() {
            eprintln!("Test file not found, skipping: {}", test_path.display());
            return;
        }
        let tag = id3::Tag::read_from_path(&test_path).expect("Failed to read ID3 tags from test file");
        let serato_data = SeratoData::parse(&tag).expect("SeratoData should be present in test file");

        let display_output = format!("{serato_data}");
        assert!(
            display_output.contains("Serato tags:"),
            "Display should contain 'Serato tags:', got: {display_output}"
        );
        assert!(
            display_output.contains("SeratoAutotags"),
            "Display should contain 'SeratoAutotags', got: {display_output}"
        );
        assert!(
            display_output.contains("SeratoAnalysis"),
            "Display should contain 'SeratoAnalysis', got: {display_output}"
        );
        assert!(
            display_output.contains("SeratoBeatGrid"),
            "Display should contain 'SeratoBeatGrid', got: {display_output}"
        );
        assert!(
            display_output.contains("SeratoOverview"),
            "Display should contain 'SeratoOverview', got: {display_output}"
        );
        assert!(
            display_output.contains("SeratoMarkers"),
            "Display should contain 'SeratoMarkers', got: {display_output}"
        );
        assert!(
            display_output.contains("BPM"),
            "Display should contain 'BPM', got: {display_output}"
        );
    }
}

#[cfg(test)]
mod test_serato_data_display_empty {
    use super::*;

    #[test]
    fn display_shows_none_for_all_fields() {
        let serato_data = SeratoData::default();
        let display_output = format!("{serato_data}");
        assert!(
            display_output.contains("None"),
            "Display of empty SeratoData should contain 'None', got: {display_output}"
        );
    }
}

#[cfg(test)]
mod test_format_as_byte_string {
    use super::*;

    #[test]
    fn formats_three_bytes() {
        let result = format_as_byte_string(&[0x00, 0xff, 0xab]);
        assert_eq!(result, "00 ff ab");
    }

    #[test]
    fn formats_empty_slice() {
        let result = format_as_byte_string(&[]);
        assert_eq!(result, "");
    }
}

#[cfg(test)]
mod test_hexdump {
    use super::*;

    #[test]
    fn contains_hex_values_without_ascii() {
        let data = [0x48, 0x65, 0x6c, 0x6c, 0x6f];
        let result = hexdump(&data, false);
        assert!(
            result.contains("48"),
            "Hexdump should contain hex value '48', got: {result}"
        );
        assert!(
            result.contains("65"),
            "Hexdump should contain hex value '65', got: {result}"
        );
        assert!(
            result.contains("6c"),
            "Hexdump should contain hex value '6c', got: {result}"
        );
        assert!(
            result.contains("6f"),
            "Hexdump should contain hex value '6f', got: {result}"
        );
    }

    #[test]
    fn contains_ascii_delimiters_when_enabled() {
        let data = [0x48, 0x65, 0x6c, 0x6c, 0x6f];
        let result = hexdump(&data, true);
        assert!(
            result.contains('|'),
            "Hexdump with ASCII should contain '|' delimiters, got: {result}"
        );
    }

    #[test]
    fn returns_empty_for_empty_slice() {
        let result = hexdump(&[], false);
        assert_eq!(result, "");

        let result_ascii = hexdump(&[], true);
        assert_eq!(result_ascii, "");
    }
}
