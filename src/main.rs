mod renamer;
mod track;

extern crate colored;

use crate::renamer::Renamer;

use anyhow::Result;
use clap::Parser;

use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(author, about, version, arg_required_else_help = true)]
struct Args {
    /// Input directory with audio files to rename
    input_dir: String,

    /// Only print changes
    #[arg(short, long)]
    print: bool,

    /// Rename audio files
    #[arg(short, long)]
    rename: bool,

    /// Sort audio files by name
    #[arg(short, long)]
    sort: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input_path = args.input_dir.trim();
    if input_path.is_empty() {
        anyhow::bail!("empty input path");
    }
    let filepath = Path::new(input_path);
    if !filepath.is_dir() {
        anyhow::bail!(
            "Input directory does not exist or is not accessible: '{}'",
            filepath.display()
        );
    }
    let absolute_input_path = fs::canonicalize(filepath)?;

    let mut renamer = Renamer::new(absolute_input_path, args.rename, args.sort, args.print, args.verbose);
    renamer.run()
}
