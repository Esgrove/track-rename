use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use colored::Colorize;

use track_rename::track::Track;
use track_rename::{completion, serato, tags, utils};

/// Subcommands for trackprint.
#[derive(Subcommand)]
enum TrackprintCommand {
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

#[derive(Parser)]
#[command(author, version, about = "Print tag data", name = "trackprint")]
pub struct Args {
    #[command(subcommand)]
    command: Option<TrackprintCommand>,

    /// Optional input directory or audio file
    #[arg(value_hint = clap::ValueHint::AnyPath)]
    path: Option<PathBuf>,

    /// Enable debug prints
    #[arg(short, long)]
    debug: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(TrackprintCommand::Completion {
        shell,
        install,
        verbose,
    }) = &args.command
    {
        return completion::generate_shell_completion(
            *shell,
            Args::command(),
            *install,
            *verbose,
            env!("CARGO_BIN_NAME"),
        );
    }

    let absolute_input_path = utils::resolve_input_path(args.path.as_deref())?;

    let tracks = if absolute_input_path.is_file() {
        Track::try_from_path(&absolute_input_path).map_or_else(Vec::new, |track| vec![track])
    } else {
        utils::collect_tracks(&absolute_input_path)
    };

    for track in tracks {
        println!("{}", track.to_string().bold().magenta());
        if let Some(file_tags) = tags::read_tags(&track, args.verbose || args.debug) {
            // Don't print empty tags
            if file_tags.frames().next().is_some() {
                tags::print_tag_data(&file_tags);
                serato::print_serato_tags(&file_tags);
            } else {
                println!("{}", "Empty tags".yellow());
            }
        }
    }

    Ok(())
}
