use std::env;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(author, about = "Print tag data", version)]
pub struct Args {
    /// Optional input directory or audio file
    path: Option<String>,

    /// Convert failed files to AIFF using ffmpeg
    #[arg(short, long)]
    convert: bool,

    /// Collect and save genre statistics
    #[arg(short, long)]
    genre: bool,

    /// Enable debug prints
    #[arg(short, long)]
    debug: bool,

    /// Log files that can't be read
    #[arg(short, long)]
    log: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    let args = Args::parse();

    let absolute_input_path = track_rename::utils::resolve_input_path(&args.path)?;

    Ok(())
}
