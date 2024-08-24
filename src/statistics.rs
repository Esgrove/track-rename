use std::fmt;

use colored::Colorize;

/// Store renaming statistics.
#[derive(Debug, Default)]
pub struct Statistics {
    pub tags: usize,
    pub tags_fixed: usize,
    pub to_rename: usize,
    pub renamed: usize,
    pub to_remove: usize,
    pub removed: usize,
    pub duplicates: usize,
    pub failed: usize,
    pub converted: usize,
}

impl Statistics {
    /// Return true if nothing needs to change
    pub const fn no_changes(&self) -> bool {
        self.tags == 0
            && self.to_rename == 0
            && self.to_remove == 0
            && self.duplicates == 0
            && self.failed == 0
            && self.converted == 0
    }
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.no_changes() {
            write!(f, "{}", "All Good".green())?;
        } else {
            writeln!(f, "{}", "Updated:".bold())?;
            writeln!(f, "Fix tags:   {} / {}", self.tags_fixed, self.tags)?;
            writeln!(f, "Renamed:    {} / {}", self.renamed, self.to_rename)?;
            if self.converted > 0 {
                writeln!(f, "Converted:  {}", self.converted)?;
            }
            if self.to_remove > 0 {
                writeln!(f, "Deleted:    {} / {}", self.removed, self.to_remove)?;
            }
            if self.duplicates > 0 {
                writeln!(f, "Duplicate:  {}", self.duplicates)?;
            }
            if self.failed > 0 {
                writeln!(f, "Failed:     {}", self.failed)?;
            }
        }
        Ok(())
    }
}
