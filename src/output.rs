use std::cmp::Ordering;

use colored::{ColoredString, Colorize};
use difference::{Changeset, Difference};

/// Format bool value as a coloured string.
#[must_use]
pub fn colorize_bool(value: bool) -> ColoredString {
    if value { "true".green() } else { "false".yellow() }
}

/// Create a coloured diff for the given strings.
pub fn color_diff(old: &str, new: &str, stacked: bool) -> (String, String) {
    let changeset = Changeset::new(old, new, "");
    let mut old_diff = String::new();
    let mut new_diff = String::new();

    if stacked {
        // Find the starting index of the first matching sequence for a nicer visual alignment.
        // For example:
        //   Constantine - Onde As Satisfaction (Club Tool).aif
        //        Darude - Onde As Satisfaction (Constantine Club Tool).aif
        // Instead of:
        //   Constantine - Onde As Satisfaction (Club Tool).aif
        //   Darude - Onde As Satisfaction (Constantine Club Tool).aif
        for diff in &changeset.diffs {
            if let Difference::Same(text) = diff {
                if text.chars().all(char::is_whitespace) || text.chars().count() < 2 {
                    continue;
                }

                let old_first_match_index = old.find(text);
                let new_first_match_index = new.find(text);

                // Add leading whitespace so that the first matching sequence lines up.
                if let (Some(old_index), Some(new_index)) = (old_first_match_index, new_first_match_index) {
                    match old_index.cmp(&new_index) {
                        Ordering::Greater => {
                            new_diff = " ".repeat(old_index.saturating_sub(new_index));
                        }
                        Ordering::Less => {
                            old_diff = " ".repeat(new_index.saturating_sub(old_index));
                        }
                        Ordering::Equal => {}
                    }
                    break;
                }
            }
        }
    }

    for diff in changeset.diffs {
        match diff {
            Difference::Same(ref text) => {
                old_diff.push_str(text);
                new_diff.push_str(text);
            }
            Difference::Add(ref text) => {
                if text.chars().all(char::is_whitespace) {
                    new_diff.push_str(&text.on_green().to_string());
                } else {
                    new_diff.push_str(&text.green().to_string());
                }
            }
            Difference::Rem(ref text) => {
                if text.chars().all(char::is_whitespace) {
                    old_diff.push_str(&text.on_red().to_string());
                } else {
                    old_diff.push_str(&text.red().to_string());
                }
            }
        }
    }

    (old_diff, new_diff)
}

/// Print a single line diff of the changes.
pub fn print_diff(old: &str, new: &str) {
    let (old_diff, new_diff) = color_diff(old, new, false);
    println!("{old_diff} -> {new_diff}");
}

/// Print a stacked diff of the changes.
pub fn print_stacked_diff(old: &str, new: &str) {
    let (old_diff, new_diff) = color_diff(old, new, true);
    println!("{old_diff}");
    println!("{new_diff}");
}

/// Print a divider line that matches the length of the reference text
/// plus an optional prefix width.
pub fn print_divider(text: &str, prefix_width: usize) {
    println!("{}", "-".repeat(prefix_width + text.chars().count()));
}

/// Print error message with red color to stderr.
#[inline]
pub fn print_error(message: &str) {
    eprintln!("{}", format!("Error: {message}").red());
}

/// Print formatted error message with red color to stderr.
#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        $crate::output::print_error(&format!($($arg)*))
    };
}

/// Print warning message with yellow color to stderr.
#[inline]
pub fn print_yellow(message: &str) {
    eprintln!("{}", message.yellow());
}

/// Print formatted warning message with yellow color to stderr.
#[macro_export]
macro_rules! print_yellow {
    ($($arg:tt)*) => {
        $crate::output::print_yellow(&format!($($arg)*))
    };
}

/// Print message with green color.
#[inline]
pub fn print_green(message: &str) {
    println!("{}", message.green());
}

/// Print formatted message with green color.
#[macro_export]
macro_rules! print_green {
    ($($arg:tt)*) => {
        $crate::output::print_green(&format!($($arg)*))
    };
}

/// Print message with magenta color.
#[inline]
pub fn print_magenta(message: &str) {
    println!("{}", message.magenta());
}

/// Print formatted message with magenta color.
#[macro_export]
macro_rules! print_magenta {
    ($($arg:tt)*) => {
        $crate::output::print_magenta(&format!($($arg)*))
    };
}

/// Print message with bold magenta color.
#[inline]
pub fn print_magenta_bold(message: &str) {
    println!("{}", message.magenta().bold());
}

/// Print formatted message with bold magenta color.
#[macro_export]
macro_rules! print_magenta_bold {
    ($($arg:tt)*) => {
        $crate::output::print_magenta_bold(&format!($($arg)*))
    };
}

/// Print message with cyan color.
#[inline]
pub fn print_cyan(message: &str) {
    println!("{}", message.cyan());
}

/// Print formatted message with cyan color.
#[macro_export]
macro_rules! print_cyan {
    ($($arg:tt)*) => {
        $crate::output::print_cyan(&format!($($arg)*))
    };
}

/// Print message with bold style.
#[inline]
pub fn print_bold(message: &str) {
    println!("{}", message.bold());
}

/// Print formatted message with bold style.
#[macro_export]
macro_rules! print_bold {
    ($($arg:tt)*) => {
        $crate::output::print_bold(&format!($($arg)*))
    };
}

/// Print message with dimmed style.
#[inline]
pub fn print_dimmed(message: &str) {
    println!("{}", message.dimmed());
}

/// Print formatted message with dimmed style.
#[macro_export]
macro_rules! print_dimmed {
    ($($arg:tt)*) => {
        $crate::output::print_dimmed(&format!($($arg)*))
    };
}

#[cfg(test)]
mod test_color_diff {
    use super::*;

    /// Strip ANSI escape sequences from a string so plain text can be checked.
    fn strip_ansi(text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        while let Some(character) = chars.next() {
            if character == '\x1b' {
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next == 'm' {
                        break;
                    }
                }
            } else {
                result.push(character);
            }
        }
        result
    }

    #[test]
    fn identical_strings_contain_input_text() {
        let (old_diff, new_diff) = color_diff("hello", "hello", false);
        assert!(
            old_diff.contains("hello"),
            "Expected old_diff to contain 'hello', got: {old_diff}"
        );
        assert!(
            new_diff.contains("hello"),
            "Expected new_diff to contain 'hello', got: {new_diff}"
        );
    }

    #[test]
    fn completely_different_strings_contain_respective_text() {
        let (old_diff, new_diff) = color_diff("alpha", "beta", false);
        assert!(
            !old_diff.is_empty(),
            "Expected old_diff to be non-empty for removed text"
        );
        assert!(!new_diff.is_empty(), "Expected new_diff to be non-empty for added text");
        assert_ne!(
            old_diff, new_diff,
            "Expected old_diff and new_diff to differ for completely different strings"
        );
        let old_plain = strip_ansi(&old_diff);
        let new_plain = strip_ansi(&new_diff);
        assert!(
            old_plain.contains("alpha"),
            "Expected old_diff to contain 'alpha', got plain text: {old_plain}"
        );
        assert!(
            new_plain.contains("beta"),
            "Expected new_diff to contain 'beta', got plain text: {new_plain}"
        );
    }

    #[test]
    fn partial_difference_contains_common_text() {
        let (old_diff, new_diff) = color_diff("hello world", "hello earth", false);
        // The common prefix "hello " appears as plain text in both diffs
        assert!(
            old_diff.contains("hello "),
            "Expected old_diff to contain common prefix 'hello ', got: {old_diff}"
        );
        assert!(
            new_diff.contains("hello "),
            "Expected new_diff to contain common prefix 'hello ', got: {new_diff}"
        );
        // The differing suffixes get ANSI-wrapped per character, so they won't appear
        // as contiguous plain substrings. Verify the diffs are longer than just the
        // common prefix and differ from each other.
        assert!(
            old_diff.len() > "hello ".len(),
            "Expected old_diff to contain more than just the common prefix, got: {old_diff}"
        );
        assert!(
            new_diff.len() > "hello ".len(),
            "Expected new_diff to contain more than just the common prefix, got: {new_diff}"
        );
        assert_ne!(
            old_diff, new_diff,
            "Expected old_diff and new_diff to differ for changed suffixes"
        );
    }

    #[test]
    fn stacked_mode_alignment() {
        // "ABC - Shared" is shorter before the common " - Shared" than "XYZW - Shared",
        // so old_diff should get leading spaces to align the common text.
        let (old_diff, new_diff) = color_diff("ABC - Shared", "XYZW - Shared", true);
        assert!(
            old_diff.contains(" - Shared"),
            "Expected old_diff to contain common text ' - Shared', got: {old_diff}"
        );
        assert!(
            new_diff.contains(" - Shared"),
            "Expected new_diff to contain common text ' - Shared', got: {new_diff}"
        );
        // The shorter old string should be padded with leading spaces for alignment.
        assert!(
            old_diff.starts_with(' '),
            "Expected old_diff to start with padding spaces for alignment, got: {old_diff}"
        );
        assert!(
            !new_diff.starts_with(' '),
            "Expected new_diff to have no leading padding, got: {new_diff}"
        );
    }
}
