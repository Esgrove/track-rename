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

use std::path::PathBuf;
use std::{env, fs};

#[derive(Parser)]
#[command(author, about, version, arg_required_else_help = true)]
struct Args {
    /// Optional input directory with audio files to format
    input_dir: Option<String>,

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

    /// Only fix tags, do not rename
    #[arg(short, long)]
    tags_only: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input_path = args.input_dir.unwrap_or_default().trim().to_string();
    let filepath = if input_path.is_empty() {
        env::current_dir().context("Failed to get current working directory")?
    } else {
        PathBuf::from(input_path)
    };
    if !filepath.is_dir() {
        anyhow::bail!(
            "Input directory does not exist or is not accessible: '{}'",
            filepath.display()
        );
    }
    let absolute_input_path = fs::canonicalize(filepath)?;

    let mut renamer = Renamer::new(
        absolute_input_path,
        args.force,
        args.rename,
        args.sort,
        args.print,
        args.tags_only,
        args.verbose,
    );
    renamer.run()
}
