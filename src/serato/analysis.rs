use std::fmt;
use std::fmt::Display;

use anyhow::anyhow;
use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct AnalysisVersion {
    pub major_version: u8,
    pub minor_version: u8,
}

impl AnalysisVersion {
    /// Parse analysis tag.
    /// Contains the Serato analysis version number (*here:* 2.1).
    ///
    /// | Offset | Length | Raw Value | Decoded Value | Type            | Description
    /// | ------ | ------ | --------- | ------------- | --------------- | -----------
    /// |   `00` |   `01` |      `02` |           `2` | `unsigned char` | Major Version
    /// |   `01` |   `01` |      `01` |           `1` | `unsigned char` | Minor Version
    ///
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() >= 2 {
            let major_version = data[0];
            let minor_version = data[1];

            Ok(Self {
                major_version,
                minor_version,
            })
        } else {
            Err(anyhow!("Data is too short to contain version information"))
        }
    }
}

impl Display for AnalysisVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Version: {}.{}", self.major_version, self.minor_version)
    }
}
