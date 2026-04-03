//! Library support for formatting, renaming, and inspecting audio track metadata.

#![warn(missing_docs)]

#[macro_use]
/// Colored output helpers and printing macros.
pub mod output;

/// Shell completion generation utilities.
pub mod completion;
/// Audio file format detection and display.
pub mod file_format;
/// Artist, title, album, and filename formatting helpers.
pub mod formatting;
/// Genre normalization utilities and lookup tables.
pub mod genre;
/// Serato metadata parsing and crate file support.
pub mod serato;
/// Persistent state storage for processed tracks.
pub mod state;
/// Tag reading, writing, and display helpers.
pub mod tags;
/// Track model and file operations.
pub mod track;
/// Miscellaneous filesystem and string helpers.
pub mod utils;
