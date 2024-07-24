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
        let (term_width, _) = terminal::size().map_err(|e| anyhow!("Failed to get terminal size: {}", e))?;
        let width = self.blocks.len();

        let mut waveform = String::new();

        // Calculate average for each consecutive two values to reduce height to 8
        let new_height = 4;
        let mut averaged_blocks: Vec<Vec<u8>> = vec![vec![0; new_height]; width];

        for x in 0..width {
            for y in 0..new_height {
                let avg = self.blocks[x][4 * y] as u16
                    + self.blocks[x][4 * y + 1] as u16
                    + self.blocks[x][4 * y + 2] as u16
                    + self.blocks[x][4 * y + 3] as u16 / new_height as u16;
                averaged_blocks[x][y] = avg as u8;
            }
        }

        let resampled_blocks = if width > term_width as usize {
            // Resample to fit terminal width
            let ratio = width as f32 / term_width as f32;
            (0..term_width as usize)
                .map(|x| {
                    let src_start = (x as f32 * ratio).floor() as usize;
                    let src_end = ((x as f32 * ratio).ceil() as usize).min(width);
                    let count = (src_end - src_start).max(1);
                    let mut block = vec![0; new_height];
                    for i in src_start..src_end {
                        for y in 0..new_height {
                            block[y] += averaged_blocks[i][y];
                        }
                    }
                    for y in 0..new_height {
                        block[y] /= count as u8;
                    }
                    block
                })
                .collect::<Vec<_>>()
        } else {
            averaged_blocks
        };

        for y in (0..new_height).rev() {
            for x in 0..(term_width as usize) {
                let value = resampled_blocks[x][y];
                let symbol = match value {
                    0..=34 => '░',
                    35..=78 => '▒',
                    _ => '█',
                };
                waveform += format!("{}", symbol).as_str();
            }
            waveform.push('\n');
        }
        Ok(waveform)
    }
}

impl Display for Overview {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Overview ({}):", self.blocks.len())?;
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
