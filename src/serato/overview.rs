use std::fmt;
use std::fmt::Display;

use anyhow::Result;
use anyhow::anyhow;
use colored::Colorize;
use crossterm::terminal;

#[derive(Debug, Clone, Default)]
/// Contains the waveform overview data.
/// It seems the length will always be 240 time slices,
/// regardless of the track length.
/// Each time slice is divided into 16 frequency bands,
/// with the byte value corresponding to the strength of that frequency band.
pub struct Overview {
    blocks: Vec<[u8; 16]>,
}

impl Overview {
    /// Parse the waveform overview.
    /// The overview is build of 16 byte blocks that contain the frequency data for each time slice.
    ///
    /// | Offset | Length | Raw Value     | Type           | Description
    /// | ------ | ------ | ------------- | -------------- | -----------
    /// |   `00` |   `02` | `01 05`       |                |
    /// |   `02` |   `10` | `01` ... `01` | 16 * `uint8_t` | Frequency information
    /// |    ... |    ... | `01` ... `01` | 16 * `uint8_t` | Frequency information
    /// |  `ef2` |   `10` | `01` ... `01` | 16 * `uint8_t` | Frequency information
    ///
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 2 {
            return Err(anyhow!("Data too short to contain initial bytes"));
        }

        let mut frequency_info = Vec::new();
        let mut offset = 2;

        while offset + 16 <= data.len() {
            let mut freq_block = [0u8; 16];
            freq_block.copy_from_slice(&data[offset..offset + 16]);
            frequency_info.push(freq_block);
            offset += 16;
        }

        Ok(Self { blocks: frequency_info })
    }

    /// Convert waveform overview to a minimized text representation for terminal display.
    fn draw_waveform(&self) -> Result<String> {
        let (terminal_width, _) = terminal::size().map_err(|e| anyhow!("Failed to get terminal size: {e}"))?;
        let width = self.blocks.len();

        let mut waveform = String::new();

        // Calculate average for consecutive values to reduce height from original 16 to specified height
        let height = 8;
        let ratio = 16 / height;
        let mut averaged_blocks: Vec<Vec<u8>> = vec![vec![0; height]; width];

        for (x, column) in averaged_blocks.iter_mut().enumerate().take(width) {
            for (y, value) in column.iter_mut().enumerate().take(height) {
                let avg: u16 = self.blocks[x][ratio * y..ratio * y + ratio]
                    .iter()
                    .map(|&v| u16::from(v))
                    .sum::<u16>()
                    / height as u16;
                *value = avg as u8;
            }
        }

        // Adjust width if needed to fit into available terminal width
        let resampled_blocks = if terminal_width >= 240 {
            averaged_blocks
        } else if terminal_width >= 120 {
            // Downsample by two, 240 -> 120
            (0..width / 2)
                .map(|i| {
                    (0..height)
                        .map(|y| {
                            u16::midpoint(
                                u16::from(averaged_blocks[2 * i][y]),
                                u16::from(averaged_blocks[2 * i + 1][y]),
                            ) as u8
                        })
                        .collect()
                })
                .collect()
        } else {
            // Downsample by three, 240 -> 80
            (0..width / 3)
                .map(|i| {
                    (0..height)
                        .map(|y| {
                            ((u16::from(averaged_blocks[3 * i][y])
                                + u16::from(averaged_blocks[3 * i + 1][y])
                                + u16::from(averaged_blocks[3 * i + 2][y]))
                                / 3) as u8
                        })
                        .collect()
                })
                .collect()
        };

        let max_value = resampled_blocks
            .iter()
            .flat_map(|row| row.iter())
            .copied()
            .max()
            .unwrap_or(1);

        // Normalize values to range 0.0 - 1.0
        let normalized_blocks: Vec<Vec<f32>> = resampled_blocks
            .iter()
            .map(|row| {
                row.iter()
                    .map(|&value| f32::from(value) / f32::from(max_value))
                    .collect()
            })
            .collect();

        // Iterate in reverse so first values of the vertical block go to the bottom of the waveform
        for y in (0..height).rev() {
            for block in &normalized_blocks {
                let (symbol, color) = match block[y] {
                    value if value <= 0.05 => ('░', "blue"),
                    value if value <= 0.25 => ('░', "cyan"),
                    value if value <= 0.5 => ('▒', "green"),
                    value if value <= 0.75 => ('▒', "yellow"),
                    _ => ('█', "magenta"),
                };
                let formatted = symbol.to_string().color(color).to_string();
                waveform.push_str(&formatted);
            }
            waveform.push('\n');
        }

        Ok(waveform)
    }
}

impl Display for Overview {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.draw_waveform() {
            Ok(view) => {
                write!(f, "{view}")
            }
            Err(error) => {
                write!(f, "Error: {error}")
            }
        }
    }
}
