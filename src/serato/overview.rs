use std::fmt;
use std::fmt::Display;

use anyhow::Result;
use anyhow::anyhow;
use colored::Colorize;
use crossterm::terminal;

/// Number of frequency bands in each overview block.
const BAND_COUNT: usize = 16;

/// Number of rows in the rendered terminal waveform.
const WAVEFORM_HEIGHT: usize = 8;

/// Contains the waveform overview data.
/// It seems the length will always be 240 time slices,
/// regardless of the track length.
/// Each time slice is divided into 16 frequency bands,
/// with the byte value corresponding to the strength of that frequency band.
#[derive(Debug, Clone, Default)]
pub struct Overview {
    blocks: Vec<[u8; BAND_COUNT]>,
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

        let mut frequency_info = Vec::with_capacity((data.len() - 2) / 16);
        let mut offset = 2;

        while offset + 16 <= data.len() {
            let mut frequency_block = [0u8; 16];
            frequency_block.copy_from_slice(&data[offset..offset + 16]);
            frequency_info.push(frequency_block);
            offset += 16;
        }

        Ok(Self { blocks: frequency_info })
    }

    /// Convert waveform overview to a minimized text representation for terminal display.
    fn draw_waveform(&self) -> Result<String> {
        let (terminal_width, _) = terminal::size().map_err(|error| anyhow!("Failed to get terminal size: {error}"))?;
        let levels = self.waveform_levels(terminal_width);

        let mut waveform = String::new();
        // Iterate in reverse so first values of the vertical block go to the bottom of the waveform
        for row in (0..WAVEFORM_HEIGHT).rev() {
            for column in &levels {
                let (symbol, color) = match column[row] {
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

    /// Average frequency bands to the waveform height and downsample columns to fit the terminal,
    /// returning levels normalized to the range 0.0 - 1.0.
    fn waveform_levels(&self, terminal_width: u16) -> Vec<[f32; WAVEFORM_HEIGHT]> {
        let bands_per_row = BAND_COUNT / WAVEFORM_HEIGHT;
        let averaged: Vec<[u16; WAVEFORM_HEIGHT]> = self
            .blocks
            .iter()
            .map(|block| {
                std::array::from_fn(|row| {
                    block[bands_per_row * row..bands_per_row * (row + 1)]
                        .iter()
                        .map(|&value| u16::from(value))
                        .sum::<u16>()
                        / bands_per_row as u16
                })
            })
            .collect();

        let columns_per_output = match terminal_width {
            240.. => 1,
            120.. => 2,
            _ => 3,
        };
        let resampled: Vec<[u16; WAVEFORM_HEIGHT]> = averaged
            .chunks_exact(columns_per_output)
            .map(|group| {
                std::array::from_fn(|row| {
                    group.iter().map(|column| column[row]).sum::<u16>() / columns_per_output as u16
                })
            })
            .collect();

        let max_value = resampled.iter().flatten().copied().max().unwrap_or(0).max(1);
        resampled
            .iter()
            .map(|column| std::array::from_fn(|row| f32::from(column[row]) / f32::from(max_value)))
            .collect()
    }
}

impl Display for Overview {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.draw_waveform() {
            Ok(view) => {
                write!(formatter, "{view}")
            }
            Err(error) => {
                write!(formatter, "Error: {error}")
            }
        }
    }
}

#[cfg(test)]
mod test_overview {
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

    #[test]
    fn waveform_levels_downsample_to_terminal_width() {
        let overview = build_overview_with_blocks(240);
        assert_eq!(overview.waveform_levels(300).len(), 240);
        assert_eq!(overview.waveform_levels(160).len(), 120);
        assert_eq!(overview.waveform_levels(100).len(), 80);
    }

    #[test]
    fn waveform_levels_average_band_pairs() {
        let mut block = [0u8; 16];
        block[0] = 10;
        block[1] = 30;
        block[14] = 40;
        block[15] = 40;
        let overview = Overview { blocks: vec![block] };
        let levels = overview.waveform_levels(300);
        assert_eq!(levels.len(), 1);
        assert!((levels[0][0] - 0.5).abs() < f32::EPSILON, "Expected (10 + 30) / 2 / 40");
        assert!(
            (levels[0][7] - 1.0).abs() < f32::EPSILON,
            "Loudest row should normalize to 1.0"
        );
        assert!(levels[0][1..7].iter().all(|&level| level == 0.0));
    }

    #[test]
    fn waveform_levels_handle_silence() {
        let overview = Overview {
            blocks: vec![[0u8; 16]; 240],
        };
        let levels = overview.waveform_levels(300);
        assert!(levels.iter().flatten().all(|&level| level == 0.0));
    }
}
