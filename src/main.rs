mod config;
mod statistics;
mod track_renamer;

use std::path::PathBuf;

use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::Shell;

use crate::track_renamer::TrackRenamer;

#[derive(Parser)]
#[command(author, about, version)]
pub struct RenamerArgs {
    /// Optional input directory or audio file to format
    #[arg(value_hint = clap::ValueHint::AnyPath)]
    path: Option<PathBuf>,

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

    /// Overwrite existing files when renaming
    #[arg(short, long)]
    overwrite: bool,

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

    /// Generate shell completion
    #[arg(short = 'e', long, value_name = "SHELL")]
    completion: Option<Shell>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = RenamerArgs::parse();
    let absolute_input_path = track_rename::utils::resolve_input_path(args.path.as_deref())?;
    args.completion.as_ref().map_or_else(
        || TrackRenamer::new(absolute_input_path, &args).run(),
        |shell| {
            track_rename::utils::generate_shell_completion(*shell, RenamerArgs::command(), true, env!("CARGO_BIN_NAME"))
        },
    )
}
