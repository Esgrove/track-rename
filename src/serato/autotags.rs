use std::fmt;
use std::fmt::Display;

use anyhow::anyhow;

#[derive(Debug, Clone, Default)]
pub struct AutoTags {
    /// Beats per minute
    pub bpm: f32,
    /// Calculated auto gain
    pub auto_gain: f32,
    /// Manually adjusted gain
    pub gain: f32,
}

impl AutoTags {
    /// Parse autotags data.
    /// Contains the BPM, autogain, and manual gain values.
    ///
    /// | Offset | Length | Raw Value              | Decoded Value | Type                    | Description
    /// | ------ | ------ | ---------------------- | ------------- | ----------------------- | -----------
    /// |   `00` |   `02` | `01 01`                |               | *?* (2 bytes)           |
    /// |   `02` |   `07` | `31 31 35 2e 30 30 00` |      `115.00` | ASCII (zero-terminated) | BPM
    /// |   `09` |   `07` | `2d 33 2e 32 35 37 00` |      `-3.257` | ASCII (zero-terminated) | Auto Gain
    /// |   `16` |   `06` | `30 2e 30 30 30 00`    |       `0.000` | ASCII (zero-terminated) | Gain dB
    ///
    pub fn parse(data: &[u8]) -> anyhow::Result<AutoTags> {
        if data.len() < 16 {
            return Err(anyhow!("Data is too short to contain all necessary information"));
        }

        // Parse BPM
        let bpm_str = std::str::from_utf8(&data[2..9])
            .map_err(|_| anyhow!("Failed to parse BPM string as UTF-8"))?
            .trim_end_matches(char::from(0));
        let bpm: f32 = bpm_str.parse().map_err(|_| anyhow!("Failed to parse BPM as f32"))?;

        // Parse Auto Gain
        let auto_gain_str = std::str::from_utf8(&data[9..16])
            .map_err(|_| anyhow!("Failed to parse Auto Gain string as UTF-8"))?
            .trim_end_matches(char::from(0));
        let auto_gain: f32 = auto_gain_str
            .parse()
            .map_err(|_| anyhow!("Failed to parse Auto Gain as f32"))?;

        // Parse Gain dB
        let gain_str = std::str::from_utf8(&data[16..22])
            .map_err(|_| anyhow!("Failed to parse Gain dB string as UTF-8"))?
            .trim_end_matches(char::from(0));
        let gain: f32 = gain_str
            .parse()
            .map_err(|_| anyhow!("Failed to parse Gain dB as f32"))?;

        Ok(AutoTags { bpm, auto_gain, gain })
    }
}

impl Display for AutoTags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BPM: {:.2}, Auto Gain: {:.3}, Gain: {:.3}",
            self.bpm, self.auto_gain, self.gain
        )
    }
}
