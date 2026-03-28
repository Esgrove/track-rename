use std::fmt;
use std::fmt::Display;

use anyhow::anyhow;

#[derive(Debug, Clone, Default)]
pub struct AutoTags {
    /// Beats per minute
    pub bpm: f32,
    /// Calculated auto gain (dB)
    pub auto_gain: f32,
    /// Manually adjusted gain (dB)
    pub gain: f32,
}

impl AutoTags {
    /// Parse autotags data.
    /// Contains the BPM, auto gain, and manual gain values.
    ///
    /// | Offset | Length | Raw Value              | Decoded Value | Type                    | Description
    /// | ------ | ------ | ---------------------- | ------------- | ----------------------- | -----------
    /// |   `00` |   `02` | `01 01`                |               | *?* (2 bytes)           |
    /// |   `02` |   `07` | `31 31 35 2e 30 30 00` |      `115.00` | ASCII (zero-terminated) | BPM
    /// |   `09` |   `07` | `2d 33 2e 32 35 37 00` |      `-3.257` | ASCII (zero-terminated) | Auto Gain
    /// |   `16` |   `06` | `30 2e 30 30 30 00`    |       `0.000` | ASCII (zero-terminated) | Gain dB
    ///
    pub fn parse(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 16 {
            return Err(anyhow!("Data is too short to contain all necessary information"));
        }

        // Parse BPM
        let bpm_str: String = std::str::from_utf8(&data[2..9])
            .map_err(|_| anyhow!("Failed to parse BPM string as UTF-8"))?
            .trim_end_matches('\x00')
            .trim()
            .chars()
            .filter(|c| c.is_numeric() || *c == '.')
            .collect();

        let bpm: f32 = bpm_str
            .parse()
            .map_err(|e| anyhow!("Failed to parse BPM as f32: {e} {bpm_str:?}"))?;

        // Parse Auto Gain
        let auto_gain_str: String = std::str::from_utf8(&data[9..16])
            .map_err(|_| anyhow!("Failed to parse Auto Gain string as UTF-8"))?
            .replace('\x00', "")
            .trim()
            .chars()
            .filter(|c| c.is_numeric() || *c == '.' || *c == '-')
            .collect();

        let auto_gain: f32 = auto_gain_str
            .trim_end_matches('.')
            .parse()
            .map_err(|e| anyhow!("Failed to parse Auto Gain as f32: {e} {auto_gain_str:?}"))?;

        // Parse Gain dB (only if data is long enough)
        let gain: f32 = if data.len() >= 22 {
            let gain_str: String = std::str::from_utf8(&data[16..22])
                .map_err(|_| anyhow!("Failed to parse Gain dB string as UTF-8"))?
                .replace('\x00', "")
                .trim()
                .chars()
                .filter(|c| c.is_numeric() || *c == '.' || *c == '-')
                .collect();

            gain_str
                .trim_end_matches('.')
                .parse()
                .map_err(|e| anyhow!("Failed to parse Gain dB as f32: {e} {gain_str:?}"))?
        } else {
            0.0
        };

        Ok(Self { bpm, auto_gain, gain })
    }
}

impl Display for AutoTags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BPM: {:.3}, Auto Gain: {:.3} dB, Gain: {:.3} dB",
            self.bpm, self.auto_gain, self.gain
        )
    }
}

#[cfg(test)]
mod test_autotags_parse {
    use super::*;

    #[test]
    fn parses_full_autotags_data() {
        // Header (2 bytes) + BPM "128.00\0" (7 bytes) + Auto gain "-3.257\0" (7 bytes) + Gain "0.000\0" (6 bytes)
        let data: Vec<u8> = vec![
            0x01, 0x01, // header
            0x31, 0x32, 0x38, 0x2e, 0x30, 0x30, 0x00, // "128.00\0"
            0x2d, 0x33, 0x2e, 0x32, 0x35, 0x37, 0x00, // "-3.257\0"
            0x30, 0x2e, 0x30, 0x30, 0x30, 0x00, // "0.000\0"
        ];
        let autotags = AutoTags::parse(&data).expect("Should parse valid autotags data");
        let epsilon = 0.001;
        assert!((autotags.bpm - 128.0).abs() < epsilon, "BPM should be 128.0");
        assert!(
            (autotags.auto_gain - (-3.257)).abs() < epsilon,
            "Auto gain should be -3.257"
        );
        assert!((autotags.gain - 0.0).abs() < epsilon, "Gain should be 0.0");
    }

    #[test]
    fn parses_without_gain_field() {
        // Only 16 bytes: header + BPM + auto gain, no gain field
        let data: Vec<u8> = vec![
            0x01, 0x01, // header
            0x31, 0x32, 0x38, 0x2e, 0x30, 0x30, 0x00, // "128.00\0"
            0x2d, 0x33, 0x2e, 0x32, 0x35, 0x37, 0x00, // "-3.257\0"
        ];
        let autotags = AutoTags::parse(&data).expect("Should parse autotags without gain");
        let epsilon = 0.001;
        assert!((autotags.bpm - 128.0).abs() < epsilon, "BPM should be 128.0");
        assert!(
            (autotags.auto_gain - (-3.257)).abs() < epsilon,
            "Auto gain should be -3.257"
        );
        assert!(
            (autotags.gain - 0.0).abs() < epsilon,
            "Gain should default to 0.0 when missing"
        );
    }

    #[test]
    fn fails_on_data_too_short() {
        let short_data: Vec<u8> = vec![0x01, 0x01, 0x31, 0x32, 0x38];
        let result = AutoTags::parse(&short_data);
        assert!(result.is_err(), "Should fail when data is too short");
    }

    #[test]
    fn fails_on_empty_data() {
        let empty_data: Vec<u8> = vec![];
        let result = AutoTags::parse(&empty_data);
        assert!(result.is_err(), "Should fail on empty data");
    }

    #[test]
    fn parses_positive_auto_gain() {
        // Header + BPM "115.00\0" + Auto gain "2.500\0\0" (padded) + Gain "1.200\0"
        let data: Vec<u8> = vec![
            0x01, 0x01, // header
            0x31, 0x31, 0x35, 0x2e, 0x30, 0x30, 0x00, // "115.00\0"
            0x32, 0x2e, 0x35, 0x30, 0x30, 0x00, 0x00, // "2.500\0\0"
            0x31, 0x2e, 0x32, 0x30, 0x30, 0x00, // "1.200\0"
        ];
        let autotags = AutoTags::parse(&data).expect("Should parse positive auto gain");
        let epsilon = 0.001;
        assert!((autotags.bpm - 115.0).abs() < epsilon, "BPM should be 115.0");
        assert!((autotags.auto_gain - 2.5).abs() < epsilon, "Auto gain should be 2.5");
        assert!((autotags.gain - 1.2).abs() < epsilon, "Gain should be 1.2");
    }
}

#[cfg(test)]
mod test_autotags_display {
    use super::*;

    #[test]
    fn formats_all_fields() {
        let data: Vec<u8> = vec![
            0x01, 0x01, // header
            0x31, 0x32, 0x38, 0x2e, 0x30, 0x30, 0x00, // "128.00\0"
            0x2d, 0x33, 0x2e, 0x32, 0x35, 0x37, 0x00, // "-3.257\0"
            0x30, 0x2e, 0x30, 0x30, 0x30, 0x00, // "0.000\0"
        ];
        let autotags = AutoTags::parse(&data).expect("Should parse valid autotags data");
        let display_output = format!("{autotags}");
        assert!(
            display_output.contains("BPM: 128.000"),
            "Display should contain BPM value, got: {display_output}"
        );
        assert!(
            display_output.contains("Auto Gain: -3.257 dB"),
            "Display should contain auto gain value, got: {display_output}"
        );
        assert!(
            display_output.contains("Gain: 0.000 dB"),
            "Display should contain gain value, got: {display_output}"
        );
    }

    #[test]
    fn formats_default_gain_when_missing() {
        let data: Vec<u8> = vec![
            0x01, 0x01, // header
            0x31, 0x32, 0x38, 0x2e, 0x30, 0x30, 0x00, // "128.00\0"
            0x2d, 0x33, 0x2e, 0x32, 0x35, 0x37, 0x00, // "-3.257\0"
        ];
        let autotags = AutoTags::parse(&data).expect("Should parse autotags without gain");
        let display_output = format!("{autotags}");
        assert!(
            display_output.contains("Gain: 0.000 dB"),
            "Display should show default gain of 0.000 dB, got: {display_output}"
        );
    }
}
