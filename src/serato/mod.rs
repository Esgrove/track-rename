mod analysis;
mod autotags;
mod beatgrid;
mod markers;
mod overview;

use std::fmt::Display;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use colored::Colorize;
use id3::Tag;

use crate::serato::analysis::AnalysisVersion;
use crate::serato::autotags::AutoTags;
use crate::serato::beatgrid::BeatGrid;
use crate::serato::markers::Markers;
use crate::serato::overview::Overview;
use crate::utils;

/// Contains all Serato custom tag data in the file.
#[derive(Debug, Clone, Default)]
pub struct SeratoData {
    pub analysis: Option<AnalysisVersion>,
    pub autotags: Option<AutoTags>,
    pub beatgrid: Option<BeatGrid>,
    pub markers: Vec<Markers>,
    pub overview: Option<Overview>,
}

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

impl SeratoData {
    /// Parse Serato custom tags from tag data.
    #[must_use]
    pub fn parse(file_tags: &Tag) -> Option<Self> {
        let mut serato_data = Self::default();
        let mut parsed_any = false;

        for frame in file_tags.frames() {
            if let Some(object) = frame.content().encapsulated_object() {
                if let Ok(tag) = SeratoTag::from_str(&object.description) {
                    match tag {
                        SeratoTag::Analysis => match AnalysisVersion::parse(&object.data) {
                            Ok(data) => {
                                serato_data.analysis = Some(data);
                                parsed_any = true;
                            }
                            Err(error) => utils::print_error(&error.to_string()),
                        },
                        SeratoTag::Autotags => match AutoTags::parse(&object.data) {
                            Ok(data) => {
                                serato_data.autotags = Some(data);
                                parsed_any = true;
                            }
                            Err(error) => utils::print_error(&error.to_string()),
                        },
                        SeratoTag::BeatGrid => match BeatGrid::parse(&object.data) {
                            Ok(data) => {
                                serato_data.beatgrid = Some(data);
                                parsed_any = true;
                            }
                            Err(error) => utils::print_error(&error.to_string()),
                        },
                        SeratoTag::Markers => match Markers::parse(&object.data) {
                            Ok(data) => {
                                serato_data.markers = data;
                                parsed_any = true;
                            }
                            Err(error) => utils::print_error(&error.to_string()),
                        },
                        SeratoTag::Overview => match Overview::parse(&object.data) {
                            Ok(data) => {
                                serato_data.overview = Some(data);
                                parsed_any = true;
                            }
                            Err(error) => utils::print_error(&error.to_string()),
                        },
                    }
                }
            }
        }
        if parsed_any {
            Some(serato_data)
        } else {
            None
        }
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
                Self::Analysis => {
                    "SeratoAnalysis"
                }
                Self::Autotags => {
                    "SeratoAutotags"
                }
                Self::BeatGrid => {
                    "SeratoBeatGrid"
                }
                Self::Markers => {
                    "SeratoMarkers"
                }
                Self::Overview => {
                    "SeratoOverview"
                }
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

#[allow(dead_code)]
/// Debug function to print bytes as hexadecimal
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
        result.push_str(&format!("    {offset:08x}  "));

        // Format the hexadecimal values
        for byte in line {
            result.push_str(&format!("{byte:02x} "));
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
