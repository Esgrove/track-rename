mod config;
mod statistics;
mod track_renamer;

use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use crate::track_renamer::TrackRenamer;

#[derive(Parser)]
#[command(author, about, version)]
pub struct RenamerArgs {
    /// Optional input directory or audio file to format
    path: Option<String>,

    /// Resave tags for all files with ID3v2.4
    #[arg(short, long)]
    all_tags: bool,

    /// Convert failed files to AIFF using ffmpeg
    #[arg(short, long)]
    convert: bool,

    /// Collect and save genre statistics
    #[arg(short, long)]
    genre: bool,

    /// Enable debug prints
    #[arg(short, long)]
    debug: bool,

    /// Do not ask for confirmation
    #[arg(short, long)]
    force: bool,

    /// Log files that can't be read
    #[arg(short, long)]
    log: bool,

    /// Don't skip unchanged files since last run
    #[arg(short, long)]
    no_state: bool,

    /// Only print changes without modifying files
    #[arg(short, long)]
    print: bool,

    /// Rename all audio files
    #[arg(short, long)]
    rename: bool,

    /// Sort audio files by name
    #[arg(short, long)]
    sort: bool,

    /// Only fix tags without renaming files
    #[arg(short, long)]
    tags_only: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
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

    TrackRenamer::new(absolute_input_path, args).run()
}
