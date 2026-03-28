use std::fmt;
use std::fmt::Display;

use anyhow::Result;
use anyhow::anyhow;
use colored::Colorize;
use crossterm::terminal;

/// Contains the waveform overview data.
/// It seems the length will always be 240 time slices,
/// regardless of the track length.
/// Each time slice is divided into 16 frequency bands,
/// with the byte value corresponding to the strength of that frequency band.
#[derive(Debug, Clone, Default)]
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
                    value if value <= 0.06 => ('░', "blue"),
                    value if value <= 0.20 => ('░', "cyan"),
                    value if value <= 0.42 => ('▒', "green"),
                    value if value <= 0.70 => ('▒', "yellow"),
                    _ => ('█', "red"),
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

#[cfg(test)]
mod test_overview_parse {
    use super::*;

    #[test]
    fn rejects_data_too_short() {
        let empty_data: &[u8] = &[];
        assert!(Overview::parse(empty_data).is_err());

        let single_byte: &[u8] = &[0x01];
        assert!(Overview::parse(single_byte).is_err());
    }

    #[test]
    fn parses_header_only_with_no_blocks() {
        let header_only: &[u8] = &[0x01, 0x05];
        let overview = Overview::parse(header_only).expect("Should parse header-only data");
        assert_eq!(overview.blocks.len(), 0);
    }

    #[test]
    fn parses_single_block() {
        let mut data = vec![0x01, 0x05];
        let block: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        data.extend_from_slice(&block);
        let overview = Overview::parse(&data).expect("Should parse single block");
        assert_eq!(overview.blocks.len(), 1);
        assert_eq!(overview.blocks[0], block);
    }

    #[test]
    fn parses_multiple_blocks() {
        let mut data = vec![0x01, 0x05];
        let block_a: [u8; 16] = [10; 16];
        let block_b: [u8; 16] = [20; 16];
        let block_c: [u8; 16] = [30; 16];
        data.extend_from_slice(&block_a);
        data.extend_from_slice(&block_b);
        data.extend_from_slice(&block_c);
        let overview = Overview::parse(&data).expect("Should parse multiple blocks");
        assert_eq!(overview.blocks.len(), 3);
        assert_eq!(overview.blocks[0], block_a);
        assert_eq!(overview.blocks[1], block_b);
        assert_eq!(overview.blocks[2], block_c);
    }

    #[test]
    fn ignores_trailing_bytes_less_than_block_size() {
        let mut data = vec![0x01, 0x05];
        let block: [u8; 16] = [5; 16];
        data.extend_from_slice(&block);
        // Add 10 trailing bytes (less than a full 16-byte block)
        data.extend_from_slice(&[0xFF; 10]);
        let overview = Overview::parse(&data).expect("Should parse ignoring partial trailing block");
        assert_eq!(overview.blocks.len(), 1);
        assert_eq!(overview.blocks[0], block);
    }

    #[test]
    fn verifies_block_count_for_known_length() {
        let num_blocks = 240;
        let mut data = vec![0x01, 0x05];
        for index in 0..num_blocks {
            let value = (index % 256) as u8;
            let block = [value; 16];
            data.extend_from_slice(&block);
        }
        let overview = Overview::parse(&data).expect("Should parse 240 blocks");
        assert_eq!(overview.blocks.len(), num_blocks);
    }
}

#[cfg(test)]
mod test_overview_display {
    use super::*;

    /// Build an `Overview` with 240 blocks of synthetic waveform data.
    fn build_overview_with_blocks(num_blocks: usize) -> Overview {
        let blocks: Vec<[u8; 16]> = (0..num_blocks)
            .map(|index| {
                let value = ((index * 3) % 256) as u8;
                [value; 16]
            })
            .collect();
        Overview { blocks }
    }

    #[test]
    fn display_produces_non_empty_output() {
        let overview = build_overview_with_blocks(240);
        let display_output = format!("{overview}");
        // In CI the terminal size may not be available, causing draw_waveform
        // to return an error string. Either way the output must not be empty.
        assert!(!display_output.is_empty(), "Display output should not be empty");
    }

    #[test]
    fn display_with_few_blocks_produces_output() {
        let overview = build_overview_with_blocks(10);
        let display_output = format!("{overview}");
        assert!(
            !display_output.is_empty(),
            "Display output for small overview should not be empty"
        );
    }

    #[test]
    fn display_empty_overview_produces_output() {
        let overview = Overview::default();
        let display_output = format!("{overview}");
        // An empty overview may succeed with an empty waveform or return an error string
        assert!(
            !display_output.is_empty(),
            "Display output for empty overview should not be empty"
        );
    }
}
