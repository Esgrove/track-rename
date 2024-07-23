use std::str::FromStr;

use anyhow::anyhow;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SeratoTag {
    SeratoAnalysis,
    SeratoAutotags,
    SeratoBeatGrid,
    SeratoMarkers,
    SeratoOverview,
}

impl FromStr for SeratoTag {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SeratoAnalysis" => Ok(SeratoTag::SeratoAnalysis),
            "SeratoAutotags" => Ok(SeratoTag::SeratoAutotags),
            "SeratoBeatGrid" => Ok(SeratoTag::SeratoBeatGrid),
            "SeratoMarkers2" => Ok(SeratoTag::SeratoMarkers),
            "SeratoOverview" => Ok(SeratoTag::SeratoOverview),
            _ => Err(anyhow!("Unknown tag description: {}", s)),
        }
    }
}
