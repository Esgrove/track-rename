use std::fmt;

use colored::Colorize;

#[derive(Debug, Default)]
pub struct Statistics {
    pub num_tags: usize,
    pub num_tags_fixed: usize,
    pub num_to_rename: usize,
    pub num_renamed: usize,
    pub num_to_remove: usize,
    pub num_removed: usize,
    pub num_duplicates: usize,
    pub num_failed: usize,
    pub num_converted: usize,
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", "Statistics:".bold())?;
        writeln!(f, "Fix tags:   {} / {}", self.num_tags_fixed, self.num_tags)?;
        writeln!(f, "Renamed:    {} / {}", self.num_renamed, self.num_to_rename)?;
        if self.num_converted > 0 {
            writeln!(f, "Converted:  {}", self.num_converted)?;
        }
        if self.num_to_remove > 0 {
            writeln!(f, "Deleted:    {} / {}", self.num_removed, self.num_to_remove)?;
        }
        if self.num_duplicates > 0 {
            writeln!(f, "Duplicate:  {}", self.num_duplicates)?;
        }
        if self.num_failed > 0 {
            writeln!(f, "Failed:     {}", self.num_failed)?;
        }
        Ok(())
    }
}
