# Track Renamer

CLI Tool for formatting and renaming audio files.

## Dependencies

- Python 3.11+
- [Poetry](https://github.com/python-poetry/poetry)

## Usage

```console
python3 rename/renamer.py --help

Usage: renamer.py [OPTIONS] [DIRECTORY]

  Check and rename audio files.

  DIRECTORY: Optional input directory for audio files to format.

Options:
  -h, --help    Show this message and exit.
  -p, --print   Only print changes
  -r, --rename  Rename all audio files
  -s, --sort    Sort audio files by name
  -t, --tags    Only fix tags
```

## Tests

```shell
poetry run pytest -v
```
