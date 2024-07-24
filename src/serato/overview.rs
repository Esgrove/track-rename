use std::fmt;
use std::fmt::Display;

use anyhow::anyhow;
use anyhow::Result;
use crossterm::terminal;

#[derive(Debug, Clone, Default)]
/// Contains the waveform overview data.
/// The overview is build of 16 byte blocks that contain the frequency data for each time slice.
///
/// ![Serato Overview](serato-overview-hexdump.png)
///
/// | Offset | Length | Raw Value     | Type           | Description
/// | ------ | ------ | ------------- | -------------- | -----------
/// |   `00` |   `02` | `01 05`       |                |
/// |   `02` |   `10` | `01` ... `01` | 16 * `uint8_t` | Frequency information
/// |    ... |    ... | `01` ... `01` | 16 * `uint8_t` | Frequency information
/// |  `ef2` |   `10` | `01` ... `01` | 16 * `uint8_t` | Frequency information
///
pub struct Overview {
    blocks: Vec<[u8; 16]>,
}

impl Overview {
    pub fn parse(data: &[u8]) -> Result<Overview> {
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

        Ok(Overview { blocks: frequency_info })
    }

    fn draw_waveform(&self) -> Result<String> {
        let (terminal_width, _) = terminal::size().map_err(|e| anyhow!("Failed to get terminal size: {}", e))?;
        let width = self.blocks.len();

        let mut waveform = String::new();

        // Calculate average for each consecutive four values to reduce height to 4
        let new_height = 4;
        let mut averaged_blocks: Vec<Vec<u8>> = vec![vec![0; new_height]; width];

        for (x, column) in averaged_blocks.iter_mut().enumerate().take(width) {
            for (y, value) in column.iter_mut().enumerate().take(new_height) {
                let avg: u16 =
                    self.blocks[x][4 * y..4 * y + 4].iter().map(|&v| v as u16).sum::<u16>() / new_height as u16;
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
                    (0..new_height)
                        .map(|y| ((averaged_blocks[2 * i][y] as u16 + averaged_blocks[2 * i + 1][y] as u16) / 2) as u8)
                        .collect()
                })
                .collect()
        } else {
            // Downsample by three, 240 -> 80
            (0..width / 3)
                .map(|i| {
                    (0..new_height)
                        .map(|y| {
                            ((averaged_blocks[3 * i][y] as u16
                                + averaged_blocks[3 * i + 1][y] as u16
                                + averaged_blocks[3 * i + 2][y] as u16)
                                / 3) as u8
                        })
                        .collect()
                })
                .collect()
        };

        // Iterate in reverse so first values of the vertical block go to the bottom of the waveform
        for y in (0..new_height).rev() {
            for block in &resampled_blocks {
                let symbol = match block[y] {
                    0..=24 => '░',
                    25..=50 => '▒',
                    _ => '█',
                };
                waveform.push(symbol);
            }
            waveform.push('\n');
        }

        Ok(waveform)
    }
}

impl Display for Overview {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Overview:")?;
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
