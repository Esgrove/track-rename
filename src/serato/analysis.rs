use std::fmt;
use std::fmt::Display;

use anyhow::Result;
use anyhow::anyhow;

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
        write!(f, "Version {}.{}", self.major_version, self.minor_version)
    }
}

#[cfg(test)]
mod test_analysis_version_parse {
    use super::*;

    #[test]
    fn parses_valid_two_byte_data() {
        let data = [1u8, 5u8];
        let version = AnalysisVersion::parse(&data).expect("Should parse valid 2-byte data");
        assert_eq!(version.major_version, 1);
        assert_eq!(version.minor_version, 5);
    }

    #[test]
    fn parses_version_two_one() {
        let data = [2u8, 1u8];
        let version = AnalysisVersion::parse(&data).expect("Should parse version 2.1");
        assert_eq!(version.major_version, 2);
        assert_eq!(version.minor_version, 1);
    }

    #[test]
    fn parses_data_longer_than_two_bytes() {
        let data = [3u8, 7u8, 0xFFu8, 0xABu8];
        let version = AnalysisVersion::parse(&data).expect("Should parse when data has extra bytes");
        assert_eq!(version.major_version, 3);
        assert_eq!(version.minor_version, 7);
    }

    #[test]
    fn fails_on_single_byte_data() {
        let data = [1u8];
        let result = AnalysisVersion::parse(&data);
        assert!(result.is_err(), "Should fail when data is only 1 byte");
    }

    #[test]
    fn fails_on_empty_data() {
        let data: [u8; 0] = [];
        let result = AnalysisVersion::parse(&data);
        assert!(result.is_err(), "Should fail when data is empty");
    }
}

#[cfg(test)]
mod test_analysis_version_display {
    use super::*;

    #[test]
    fn displays_version_one_five() {
        let data = [1u8, 5u8];
        let version = AnalysisVersion::parse(&data).expect("Should parse valid data");
        assert_eq!(format!("{version}"), "Version 1.5");
    }

    #[test]
    fn displays_version_two_one() {
        let data = [2u8, 1u8];
        let version = AnalysisVersion::parse(&data).expect("Should parse valid data");
        assert_eq!(format!("{version}"), "Version 2.1");
    }

    #[test]
    fn displays_version_zero_zero() {
        let data = [0u8, 0u8];
        let version = AnalysisVersion::parse(&data).expect("Should parse valid data");
        assert_eq!(format!("{version}"), "Version 0.0");
    }
}
