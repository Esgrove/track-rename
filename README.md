# Track Renamer

CLI tool for formatting and renaming audio files.
Originally written in Python,
and then re-written in Rust,
which is now the primary version.

Formats artist and title tags, and renames filenames to match tags.
If tag data is missing, will try to extract artist and title from the filename.

## Rust version

Only supports ID3 tags (mp3, aif, wav) currently.

```shell
./build.sh
./install.sh
```

The convert option requires [ffmpeg](https://ffmpeg.org/) to be available in path.

See the [test data](./tests/test_formatting.rs) for formatting examples.
The formatting rules and functions are specified in [src/formatting.rs](./src/formatting.rs).

### Usage

```console
Usage: trackrename [OPTIONS] [PATH]

Arguments:
  [PATH]  Optional input directory or audio file to format

Options:
  -a, --all-tags   Resave tags for all files with ID3v2.4
  -c, --convert    Convert failed files to AIFF using ffmpeg
  -d, --debug      Enable debug prints
  -f, --force      Do not ask for confirmation
  -l, --log        Log files that can't be read
  -p, --print      Only print changes without modifying files
  -r, --rename     Rename all audio files
  -s, --sort       Sort audio files by name
  -t, --tags-only  Only fix tags without renaming files
  -v, --verbose    Verbose output
  -h, --help       Print help
  -V, --version    Print version
```

### User config

An optional user config can be put under `~/.config/track-rename.toml`.
It supports specifying track names to exclude, which will be skipped during the processing.
These can include a file extension or not, and should _not_ contain a path, just the filename.
See the [track-rename.toml](./track-rename.toml) template for more details and supported options.

### Run tests

```shell
cargo test
```

### TODO

- Refactor track renamer functions
- Support other tag / filetypes as well

## Python version

Uses [pytaglib](https://github.com/supermihi/pytaglib) for tag handling.

### Dependencies

- Python 3.11+
- [Poetry](https://github.com/python-poetry/poetry)

Install Python packages:

```shell
poetry install
```

### Usage

Run with Poetry script or directly:

```shell
poetry run rename --help
poetry run python rename/renamer.py --help
```

```console
Usage: renamer.py [OPTIONS] [DIRECTORY]

  Check and rename audio files.

  DIRECTORY: Optional input directory for audio files to format.

Options:
  -h, --help    Show this message and exit.
  -f, --force   Do not ask for confirmation
  -p, --print   Only print changes
  -r, --rename  Rename all audio files
  -s, --sort    Sort audio files by name
  -t, --tags    Only fix tags, do not rename
```

See the [test data](./tests/test_data.py) for formatting examples.

### Tests

```shell
poetry run pytest -v --cov=rename tests/
```
