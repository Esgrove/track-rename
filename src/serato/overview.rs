use std::fmt;
use std::fmt::Display;

use anyhow::anyhow;
use anyhow::Result;

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
}

impl Display for Overview {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Overview ({}):", self.blocks.len())?;
        for block in self.blocks.iter() {
            write!(f, "  {:?}", block)?;
        }
        Ok(())
    }
}
