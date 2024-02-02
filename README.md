# Track Renamer

CLI tool for formatting and renaming audio files.
Originally made in Python,
and contains a Rust version as well.

## Python version

### Dependencies

- Python 3.11+
- [Poetry](https://github.com/python-poetry/poetry)

Install Python packages:

```shell
poetry install
```

### Usage

Run with poetry script or directly:

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

## Rust version

```shell
./build.sh
./install.sh
```

```console
Usage: track-renamer.exe [OPTIONS] <INPUT_DIR>

Arguments:
  <INPUT_DIR>  Optional input directory with audio files to format

Options:
  -f, --force      Do not ask for confirmation
  -p, --print      Only print changes
  -r, --rename     Rename all audio files
  -s, --sort       Sort audio files by name
  -t, --tags-only  Only fix tags, do not rename
  -v, --verbose    Verbose output
  -h, --help       Print help
  -V, --version    Print version
```

### Run tests

```shell
cargo test
```
