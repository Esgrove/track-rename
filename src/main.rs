mod config;
mod statistics;
mod track_renamer;

use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use crate::track_renamer::TrackRenamer;

#[derive(Parser)]
#[command(author, about, version)]
pub struct RenamerArgs {
    #[command(subcommand)]
    command: Option<RenamerCommand>,

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

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

/// Subcommands for trackrename.
#[derive(Subcommand)]
enum RenamerCommand {
    /// Generate shell completion script
    #[command(name = "completion")]
    Completion {
        /// Shell to generate completion for
        #[arg(value_enum)]
        shell: Shell,

        /// Install completion script to the shell's completion directory
        #[arg(short = 'I', long)]
        install: bool,

        /// Print verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> Result<()> {
    let args = RenamerArgs::parse();
    if let Some(RenamerCommand::Completion {
        shell,
        install,
        verbose,
    }) = &args.command
    {
        return track_rename::utils::generate_shell_completion(
            *shell,
            RenamerArgs::command(),
            *install,
            *verbose,
            env!("CARGO_BIN_NAME"),
        );
    }
    TrackRenamer::new(&args)?.run()
}
