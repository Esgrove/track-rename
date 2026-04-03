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

    /// Optional input directories or audio files
    #[arg(value_hint = clap::ValueHint::AnyPath)]
    paths: Vec<PathBuf>,

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

    let tracks = collect_input_tracks(&args.paths)?;

    for (index, track) in tracks.into_iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!("{}", track.to_string().bold().magenta());
        if let Some(file_tags) = track.read_tags(args.verbose || args.debug) {
            // Don't print empty tags
            if file_tags.is_empty() {
                println!("{}", "Empty tags".yellow());
            } else {
                tags::print_tag_data(&file_tags);
                if let Some(id3_tag) = file_tags.get_id3() {
                    serato::print_serato_tags(id3_tag);
                }
            }
        }
    }

    Ok(())
}

fn collect_input_tracks(paths: &[PathBuf]) -> Result<Vec<Track>> {
    let resolved_paths = if paths.is_empty() {
        vec![utils::resolve_input_path(None)?]
    } else {
        paths
            .iter()
            .map(|path| utils::resolve_input_path(Some(path.as_path())))
            .collect::<Result<Vec<_>>>()?
    };

    let mut tracks = Vec::new();
    for path in resolved_paths {
        if path.is_file() {
            if let Some(track) = Track::try_from_path(&path) {
                tracks.push(track);
            }
        } else {
            tracks.extend(utils::collect_tracks(&path));
        }
    }
    tracks.sort();
    Ok(tracks)
}
