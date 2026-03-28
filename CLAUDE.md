# Agent instructions

## Project Overview

CLI tool for formatting and renaming audio files (MP3, AIF/AIFF).
Formats artist and title ID3 tags using extensive text-normalization rules,
and renames filenames to match the formatted tags.
Falls back to parsing artist and title from the filename when tag data is missing.
Originally written in Python (still included under `rename/`), now primarily Rust.

### Binaries

- **`trackrename`** — Main binary. Formats tags, renames files, optionally converts failed files via ffmpeg.
- **`trackprint`** — Prints ID3 tag data (including Serato tags) for audio files.
- **`crateprint`** — Reads and prints Serato `.crate` files.

## Build and Test Commands

After making code changes, always run:

```shell
cargo clippy --fix --allow-dirty
cargo clippy --fix --allow-dirty --tests
cargo fmt
cargo test
```

### Other commands

```shell
# Build all binaries
cargo build

# Build a specific binary
cargo build --bin <name>

# Run a specific binary
cargo run --bin <name> -- [args]

# Format code
cargo fmt

# Run tests
cargo test

# Run tests with coverage report (text output)
cargo llvm-cov nextest
```

## Project Structure

```
src/
├── main.rs              # CLI entry point for trackrename (clap args + run)
├── lib.rs               # Library root, re-exports all public modules
├── completion.rs        # Shell completion script generation
├── config.rs            # Config struct merging CLI flags with user config file
├── file_format.rs       # FileFormat enum (Mp3 / Aif)
├── formatting.rs        # Text normalization for artist, title, and filename strings
├── genre.rs             # Genre normalization with lookup tables and regex
├── output.rs            # Colored terminal output functions and macros (print_error!, etc.)
├── state.rs             # SQLite persistence for tracking processed files
├── statistics.rs        # Statistics counters for processed/renamed/failed tracks
├── tags.rs              # TrackTags struct, ID3 tag reading/repair, print_tag_data
├── track.rs             # Track struct representing a single audio file
├── track_renamer.rs     # TrackRenamer orchestrator (gather files, process, rename)
├── utils.rs             # Helpers: file collection, path utilities, confirm, normalize_str
├── bin/
│   ├── trackprint.rs    # CLI binary for printing audio file tag data
│   └── crateprint.rs    # CLI binary for reading Serato .crate files
└── serato/
    ├── mod.rs           # SeratoData aggregate and SeratoTag enum, GEOB frame parsing
    ├── analysis.rs      # Serato Analysis tag parser
    ├── autotags.rs      # Serato Autotags parser (BPM, gain)
    ├── beatgrid.rs      # Serato BeatGrid tag parser
    ├── markers.rs       # Serato Markers2 tag parser (cues, loops, color, BPM lock)
    ├── overview.rs      # Serato Overview tag parser (waveform display)
    └── serato_crate.rs  # Serato .crate file reader/writer (UTF-16BE TLV format)
tests/
├── test_formatting.rs   # Rust integration tests for text formatting
├── test_tag_roundtrip.rs# Rust integration tests for tag read/write
├── files/               # Test audio files
rename/                  # Legacy Python version (pytaglib-based)
track-rename.toml        # Example user config (placed at ~/.config/track-rename.toml)
```

## Code organization

- All enums before structs
- Put all struct definitions before any implementations.
- Implementations only after last struct definition in the order of struct definitions.
- Functions after implementations
- In implementations, Order public methods before private methods
- In implementations, put associated functions last

## Code Style and Conventions

- Uses Rust 2024 edition
- Clippy is configured with pedantic and nursery lints enabled
- Do not use plain unwrap. Use proper error handling or `.expect()` in constants and test cases.
- Use `anyhow` for error handling with `Result<T>` return types
- Use `clap` with derive macros for CLI argument parsing
- Use `colored` crate for terminal output coloring
- Common helper functions and macros like `print_error!` and `print_warning!` are defined in `src/lib.rs`
- Use descriptive variable and function names. No single character variables.
- Prefer full names over abbreviations. For example: `directories` instead of `dirs`.
- Create docstrings for structs and functions.
- Avoid trailing comments.

## Testing

- **NEVER use nested modules inside test modules** - all test modules must be separate root-level `#[cfg(test)]` modules
- Do NOT wrap test modules in a single parent `mod tests` module

### Test module structure example

```rust
#[cfg(test)]
mod test_prefix_extraction {
    use super::test_helpers::*;
    use super::*;

    #[test]
    fn extracts_three_parts() { ... }
}

#[cfg(test)]
mod test_filtering {
    use super::*;

    #[test]
    fn removes_year() { ... }
}
```

## Git Commands

**NEVER run destructive git commands** including but not limited to:

- `git checkout -- <file>` (discards working directory changes)
- `git restore --staged <file>` (unstages changes)
- `git restore <file>` (discards changes)
- `git reset --hard`
- `git clean`
- `git stash drop`

These commands can permanently destroy uncommitted work.
If you need to undo changes, ask the user to do it manually.

## Documentation

When changing CLI arguments or adding new binaries, update the usage output in `README.md`.
Use the short `-h` flag to get concise output and replace the `.exe` suffix with the plain binary name:

```shell
cargo run --bin <name> -- -h
```

## Configuration

User configuration is read from `~/.config/track-rename.toml`.
See `track-rename.toml` in the repo root for an example.
Remember to update the example config file when adding new config options.
