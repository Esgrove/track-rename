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

/// Print a divider line that matches the length of the reference text.
pub fn print_divider(text: &str) {
    println!("{}", "-".repeat(text.chars().count()));
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
