use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use track_rename::track::Track;
use track_rename::{serato, utils};

#[derive(Parser)]
#[command(author, version, about = "Print tag data", name = "trackprint")]
pub struct Args {
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

    let absolute_input_path = utils::resolve_input_path(args.path.as_deref())?;

    let tracks = if absolute_input_path.is_file() {
        Track::try_from_path(&absolute_input_path).map_or_else(Vec::new, |track| vec![track])
    } else {
        utils::collect_tracks(&absolute_input_path)
    };

    for track in tracks {
        println!("{}", track.to_string().bold().magenta());
        if let Some(tags) = utils::read_tags(&track, args.verbose || args.debug) {
            // Don't print empty tags
            if tags.frames().count() > 0 {
                utils::print_tag_data(&tags);
                serato::print_serato_tags(&tags);
            } else {
                println!("{}", "Empty tags".yellow());
            }
        }
    }

    Ok(())
}
