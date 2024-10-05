use std::env;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use track_rename::track::Track;
use track_rename::{serato, utils};

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

    let absolute_input_path = utils::resolve_input_path(&args.path)?;

    let tracks = if absolute_input_path.is_file() {
        Track::try_from_path(&absolute_input_path).map_or_else(Vec::new, |track| vec![track])
    } else {
        utils::collect_tracks(&absolute_input_path)
    };

    for track in tracks {
        println!("{}", track.to_string().bold().magenta());
        match utils::read_tags(&track, args.verbose || args.debug) {
            None => {}
            Some(tags) => {
                if tags.frames().count() > 0 {
                    utils::print_tag_data(&tags);
                    serato::print_serato_tags(&tags);
                }
            }
        }
    }

    Ok(())
}
