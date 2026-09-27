# Track Renamer

CLI tool for formatting and renaming audio files.

Formats artist and title tags, and renames filenames to match tags.
If tag data is missing, will try to extract artist and title from the filename.

Supports MP3 and AIFF via ID3 tags, and FLAC via Vorbis comments.

```shell
./build.sh
./install.sh
```

The convert option requires [ffmpeg](https://ffmpeg.org/) to be available in path.

See the [test data](./tests/test_formatting.rs) for formatting examples.
The formatting rules and functions are specified in [src/formatting.rs](./src/formatting.rs).

## Usage

```console
Usage: trackrename [OPTIONS] [PATH] [COMMAND]

Commands:
  completion  Generate shell completion script
  help        Print this message or the help of the given subcommand(s)

Arguments:
  [PATH]  Optional input directory or audio file to format

Options:
  -a, --all-tags   Resave tags for all files with ID3v2.4
  -c, --convert    Convert failed MP3 files to AIFF using ffmpeg
  -g, --genre      Collect and save genre statistics
  -d, --debug      Enable debug prints
  -f, --force      Do not ask for confirmation
  -l, --log        Log files that can't be read
  -n, --no-state   Don't skip unchanged files since last run
  -o, --overwrite  Overwrite existing files when renaming
  -p, --print      Only print changes without modifying files
  -r, --rename     Rename all audio files
  -S, --silent     Suppress running index and directory output
  -s, --sort       Sort audio files by name
  -t, --tags-only  Only fix tags without renaming files
  -v, --verbose    Verbose output
  -h, --help       Print help
  -V, --version    Print version
```

## User config

An optional user config can be put under `~/.config/track-rename.toml`.
It supports specifying track names to exclude, which will be skipped during the processing.
These can include a file extension or not, and should _not_ contain a path, just the filename.
See the [track-rename.toml](./track-rename.toml) template for more details and supported options.

## Tests

```shell
cargo test
```

## Benchmarks

Performance-critical code paths are benchmarked with [Criterion](https://crates.io/crates/criterion).

```shell
cargo bench

# Compare against a saved baseline
cargo bench -- --save-baseline before
cargo bench -- --baseline before
```

## Code Coverage

Code coverage is generated using [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
with [cargo-nextest](https://nexte.st/).

```shell
# Run tests with coverage (text output)
cargo llvm-cov nextest

# Generate HTML coverage report
cargo llvm-cov nextest --html

# Generate and open HTML report in browser
cargo llvm-cov nextest --html --open
```

The HTML report is generated in `target/llvm-cov/html/`.

## Required Tools

```shell
cargo install --locked cargo-nextest
cargo install --locked cargo-llvm-cov
```

## TODO

- Refactor track renamer functions
- Support other tag / filetypes as well
