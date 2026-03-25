use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;

use track_rename::serato::crate_file::{self, SeratoCrate};

#[derive(Parser)]
#[command(author, version, about = "Print Serato crate contents", name = "crateprint")]
pub struct Args {
    /// Optional path to a .crate file or Serato Subcrates directory.
    /// Defaults to ~/Music/_Serato_/Subcrates
    #[arg(value_hint = clap::ValueHint::AnyPath)]
    path: Option<PathBuf>,

    /// Show track file paths
    #[arg(short, long)]
    tracks: bool,

    /// Verbose output (show columns and version)
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input_path = if let Some(p) = &args.path {
        dunce::canonicalize(p)
            .with_context(|| format!("Input path does not exist or is not accessible: '{}'", p.display()))?
    } else {
        let default = crate_file::default_subcrates_dir()?;
        if !default.exists() {
            anyhow::bail!(
                "Default Serato Subcrates directory not found: '{}'\nProvide a path as an argument.",
                default.display()
            );
        }
        default
    };

    if input_path.is_file() {
        let serato_crate = SeratoCrate::from_file(&input_path)?;
        print_crate(&serato_crate, args.tracks, args.verbose);
    } else if input_path.is_dir() {
        let crate_paths = crate_file::list_crates(&input_path)?;
        if crate_paths.is_empty() {
            println!("{}", "No .crate files found.".yellow());
            return Ok(());
        }
        println!(
            "Found {} crate file{} in {}\n",
            crate_paths.len().to_string().bold(),
            if crate_paths.len() == 1 { "" } else { "s" },
            dunce::simplified(&input_path).display().to_string().cyan(),
        );
        for path in &crate_paths {
            match SeratoCrate::from_file(path) {
                Ok(serato_crate) => {
                    print_crate(&serato_crate, args.tracks, args.verbose);
                }
                Err(e) => {
                    eprintln!("{} Failed to parse {}: {e}", "Error:".red().bold(), path.display());
                }
            }
        }
    } else {
        anyhow::bail!("Path is not a file or directory: '{}'", input_path.display());
    }

    Ok(())
}

/// Print a single parsed crate.
fn print_crate(serato_crate: &SeratoCrate, show_tracks: bool, verbose: bool) {
    println!("{serato_crate}");

    if verbose {
        println!("  {}: {}", "Version".dimmed(), serato_crate.version);
        if !serato_crate.columns.is_empty() {
            println!("  {}:", "Columns".dimmed());
            for (name, width) in &serato_crate.columns {
                println!("    {name} (width: {width})");
            }
        }
    }

    if show_tracks {
        if serato_crate.tracks.is_empty() {
            println!("  {}", "(no tracks)".dimmed());
        } else {
            for track in &serato_crate.tracks {
                println!("  {}", track.display());
            }
        }
    }
}
