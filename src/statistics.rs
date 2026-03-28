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
    pub overwritten: usize,
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
            && self.overwritten == 0
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
            writeln!(f, "Fix tags:    {} / {}", self.tags_fixed, self.tags)?;
            writeln!(f, "Renamed:     {} / {}", self.renamed, self.to_rename)?;
            if self.converted > 0 {
                writeln!(f, "Converted:   {}", self.converted)?;
            }
            if self.to_remove > 0 {
                writeln!(f, "Deleted:     {} / {}", self.removed, self.to_remove)?;
            }
            if self.overwritten > 0 {
                writeln!(f, "Overwritten: {}", self.overwritten)?;
            }
            if self.duplicates > 0 {
                writeln!(f, "Duplicate:   {}", self.duplicates)?;
            }
            if self.failed > 0 {
                writeln!(f, "Failed:      {}", self.failed)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_statistics_no_changes {
    use super::*;

    #[test]
    fn default_has_no_changes() {
        let stats = Statistics::default();
        assert!(stats.no_changes());
    }

    #[test]
    fn tags_means_changes() {
        let stats = Statistics {
            tags: 1,
            ..Statistics::default()
        };
        assert!(!stats.no_changes());
    }

    #[test]
    fn to_rename_means_changes() {
        let stats = Statistics {
            to_rename: 1,
            ..Statistics::default()
        };
        assert!(!stats.no_changes());
    }

    #[test]
    fn duplicates_means_changes() {
        let stats = Statistics {
            duplicates: 1,
            ..Statistics::default()
        };
        assert!(!stats.no_changes());
    }

    #[test]
    fn failed_means_changes() {
        let stats = Statistics {
            failed: 1,
            ..Statistics::default()
        };
        assert!(!stats.no_changes());
    }

    #[test]
    fn converted_means_changes() {
        let stats = Statistics {
            converted: 1,
            ..Statistics::default()
        };
        assert!(!stats.no_changes());
    }

    #[test]
    fn tags_fixed_alone_is_not_a_change() {
        let stats = Statistics {
            tags_fixed: 1,
            ..Statistics::default()
        };
        assert!(stats.no_changes());
    }
}

#[cfg(test)]
mod test_statistics_display {
    use super::*;

    #[test]
    fn default_displays_all_good() {
        let stats = Statistics::default();
        let output = format!("{stats}");
        assert!(output.contains("All Good"), "expected 'All Good' in: {output}");
    }

    #[test]
    fn displays_tags_and_renamed() {
        let stats = Statistics {
            tags: 2,
            tags_fixed: 1,
            to_rename: 3,
            renamed: 2,
            ..Statistics::default()
        };
        let output = format!("{stats}");
        assert!(output.contains("Fix tags:"), "expected 'Fix tags:' in: {output}");
        assert!(output.contains("1 / 2"), "expected 'Fix tags: 1 / 2' in: {output}");
        assert!(output.contains("Renamed:"), "expected 'Renamed:' in: {output}");
        assert!(output.contains("2 / 3"), "expected 'Renamed: 2 / 3' in: {output}");
    }

    #[test]
    fn displays_converted() {
        let stats = Statistics {
            converted: 1,
            ..Statistics::default()
        };
        let output = format!("{stats}");
        assert!(output.contains("Converted:"), "expected 'Converted:' in: {output}");
    }

    #[test]
    fn displays_failed() {
        let stats = Statistics {
            failed: 1,
            ..Statistics::default()
        };
        let output = format!("{stats}");
        assert!(output.contains("Failed:"), "expected 'Failed:' in: {output}");
    }

    #[test]
    fn displays_duplicates() {
        let stats = Statistics {
            duplicates: 1,
            ..Statistics::default()
        };
        let output = format!("{stats}");
        assert!(output.contains("Duplicate:"), "expected 'Duplicate:' in: {output}");
    }
}

#[cfg(test)]
mod test_statistics_no_changes_remaining_fields {
    use super::*;

    #[test]
    fn to_remove_means_changes() {
        let stats = Statistics {
            to_remove: 1,
            ..Statistics::default()
        };
        assert!(!stats.no_changes());
    }

    #[test]
    fn overwritten_means_changes() {
        let stats = Statistics {
            overwritten: 1,
            ..Statistics::default()
        };
        assert!(!stats.no_changes());
    }

    #[test]
    fn renamed_alone_is_not_a_change() {
        let stats = Statistics {
            renamed: 1,
            ..Statistics::default()
        };
        assert!(
            stats.no_changes(),
            "renamed alone should not count as a change since it is not checked in no_changes"
        );
    }
}

#[cfg(test)]
mod test_statistics_display_remaining_paths {
    use super::*;

    #[test]
    fn displays_deleted_when_to_remove_is_set() {
        let stats = Statistics {
            tags: 1,
            to_remove: 3,
            removed: 2,
            ..Statistics::default()
        };
        let output = format!("{stats}");
        assert!(output.contains("Deleted:"), "expected 'Deleted:' in: {output}");
        assert!(output.contains("2 / 3"), "expected '2 / 3' in: {output}");
    }

    #[test]
    fn displays_overwritten_when_set() {
        let stats = Statistics {
            tags: 1,
            overwritten: 4,
            ..Statistics::default()
        };
        let output = format!("{stats}");
        assert!(output.contains("Overwritten:"), "expected 'Overwritten:' in: {output}");
        assert!(output.contains('4'), "expected overwritten count '4' in: {output}");
    }

    #[test]
    fn displays_all_labels_when_all_fields_set() {
        let stats = Statistics {
            tags: 5,
            tags_fixed: 3,
            to_rename: 4,
            renamed: 2,
            to_remove: 3,
            removed: 1,
            duplicates: 2,
            overwritten: 1,
            failed: 1,
            converted: 1,
        };
        let output = format!("{stats}");
        assert!(output.contains("Updated:"), "expected 'Updated:' in: {output}");
        assert!(output.contains("Fix tags:"), "expected 'Fix tags:' in: {output}");
        assert!(output.contains("Renamed:"), "expected 'Renamed:' in: {output}");
        assert!(output.contains("Converted:"), "expected 'Converted:' in: {output}");
        assert!(output.contains("Deleted:"), "expected 'Deleted:' in: {output}");
        assert!(output.contains("Overwritten:"), "expected 'Overwritten:' in: {output}");
        assert!(output.contains("Duplicate:"), "expected 'Duplicate:' in: {output}");
        assert!(output.contains("Failed:"), "expected 'Failed:' in: {output}");
    }
}
