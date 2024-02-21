mod fileformat;
mod formatter;
mod renamer;
mod track;

#[cfg(test)]
mod test_formatter;

extern crate colored;

use crate::renamer::Renamer;

use anyhow::{Context, Result};
use clap::Parser;

use std::env;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, about, version)]
pub struct RenamerArgs {
    /// Optional input directory or audio file to format
    path: Option<String>,

    /// Enable debug prints
    #[arg(short, long)]
    debug: bool,

    /// Do not ask for confirmation
    #[arg(short, long)]
    force: bool,

    /// Only print changes
    #[arg(short, long)]
    print: bool,

    /// Rename all audio files
    #[arg(short, long)]
    rename: bool,

    /// Sort audio files by name
    #[arg(short, long)]
    sort: bool,

    /// Only fix tags without renaming
    #[arg(short, long)]
    tags_only: bool,

    /// Run in test mode
    #[arg(long)]
    test: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = RenamerArgs::parse();
    let input_path = args.path.clone().unwrap_or_default().trim().to_string();
    let filepath = if input_path.is_empty() {
        env::current_dir().context("Failed to get current working directory")?
    } else {
        PathBuf::from(input_path)
    };
    if !filepath.exists() {
        anyhow::bail!(
            "Input path does not exist or is not accessible: '{}'",
            dunce::simplified(&filepath).display()
        );
    }

    let absolute_input_path = dunce::canonicalize(filepath)?;

    Renamer::new(absolute_input_path, args).run()
}
