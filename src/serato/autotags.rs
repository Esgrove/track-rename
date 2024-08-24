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
        let bpm_str = std::str::from_utf8(&data[2..9])
            .map_err(|_| anyhow!("Failed to parse BPM string as UTF-8"))?
            .trim_end_matches('\x00')
            .trim();
        let bpm: f32 = bpm_str.parse().map_err(|_| anyhow!("Failed to parse BPM as f32"))?;

        // Parse Auto Gain
        let auto_gain_str = std::str::from_utf8(&data[9..16])
            .map_err(|_| anyhow!("Failed to parse Auto Gain string as UTF-8"))?
            .replace('\x00', "")
            .trim()
            .to_string();

        let auto_gain: f32 = auto_gain_str
            .parse()
            .map_err(|e| anyhow!("Failed to parse Auto Gain as f32: {e}"))?;

        // Parse Gain dB (only if data is long enough)
        let gain: f32 = if data.len() >= 22 {
            let gain_str = std::str::from_utf8(&data[16..22])
                .map_err(|_| anyhow!("Failed to parse Gain dB string as UTF-8"))?
                .replace('\x00', "")
                .trim()
                .to_string();

            gain_str
                .parse()
                .map_err(|e| anyhow!("Failed to parse Gain dB as f32: {e}"))?
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
