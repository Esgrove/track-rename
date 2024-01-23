#!/usr/bin/env python3

import difflib
import os
import sys
from pathlib import Path

import click
import colorama

try:
    from colorprint import Color, get_color, print_bold, print_error, print_warn, print_yellow
    from track import Track
    from formatter import Formatter
except ModuleNotFoundError:
    # poetry run needs the full import path
    from rename.colorprint import Color, get_color, print_bold, print_error, print_warn, print_yellow
    from rename.track import Track
    from rename.formatter import Formatter

try:
    # Workaround to be able to run tests on Apple Silicon while pytaglib is broken
    # https://github.com/supermihi/pytaglib/issues/114
    import taglib
except ImportError:
    taglib = None


class Renamer:
    """Audio track tag and filename formatting."""

    def __init__(
        self, path: Path, rename_files: bool, sort_files: bool, print_only: bool, tags_only: bool, force: bool
    ):
        self.root: Path = path
        self.rename_files: bool = rename_files
        self.sort_files: bool = sort_files
        self.print_only: bool = print_only
        self.tags_only: bool = tags_only
        self.force: bool = force

        self.file_list: list[Track] = []
        self.file_formats = (".mp3", ".flac", ".aif", ".aiff", ".m4a")
        self.total_tracks = 0
        self.num_renamed = 0
        self.num_tags_fixed = 0

        self.formatter = Formatter()

    def run(self):
        """Gather and process audio files."""
        try:
            self.gather_files()
            self.process_files()
        except KeyboardInterrupt:
            print_yellow("\ncancelled...")

        self.print_stats()

    def gather_files(self) -> None:
        """Get all audio files recursively from the root path."""
        print_bold(f"Getting audio files from {get_color(str(self.root), color=Color.cyan)}")
        file_list: list[Track] = []

        for file in self.root.rglob("*"):
            if file.suffix in self.file_formats:
                file_list.append(Track(file.stem, file.suffix, file.parent))

        if not file_list:
            sys.exit("no audio files found!")

        self.total_tracks = len(file_list)

        if self.sort_files:
            file_list.sort()

        self.file_list = file_list

    def process_files(self) -> None:
        """Format all tracks."""
        print_bold(f"Formatting {self.total_tracks} tracks...")
        current_path = self.root
        for number, file in enumerate(self.file_list):
            if not self.sort_files:
                # Print current directory when iterating in directory order
                if current_path != file.path:
                    current_path = file.path
                    print_bold(str(current_path), Color.magenta)

            # Check tags
            tag_data = taglib.File(file.full_path)
            if not tag_data:
                print_error(f"Failed to load tags for: '{file.full_path}'")
                continue

            artist = "".join(tag_data.tags.get("ARTIST", []))
            title = "".join(tag_data.tags.get("TITLE", []))
            current_tags = f"{artist} - {title}"

            if not artist and not title:
                print_warn(f"Missing tags: {file.full_path}")
                artist, title = self.get_tags_from_filename(file.name)
            elif not artist:
                print_warn(f"Missing artist tag: {file.full_path}")
                artist, _ = self.get_tags_from_filename(file.name)
            elif not title:
                print_warn(f"Missing title tag: {file.full_path}")
                _, title = self.get_tags_from_filename(file.name)

            formatted_artist, formatted_title = self.formatter.format_track(artist, title)
            new_tags = f"{formatted_artist} - {formatted_title}"

            tag_changed = False
            track_printed = False
            if current_tags != new_tags:
                print(f"{number}/{self.total_tracks}:")
                track_printed = True
                print_bold("Fix tags:", Color.blue)
                self.show_diff(current_tags, new_tags)
                self.num_tags_fixed += 1
                if not self.print_only and (self.force or self.confirm()):
                    tag_data.tags["ARTIST"] = [formatted_artist]
                    tag_data.tags["TITLE"] = [formatted_title]
                    tag_data.save()
                    tag_changed = True

                print("-" * len(new_tags))

            tag_data.close()

            if self.tags_only:
                continue

            # Check file name
            # Remove forbidden characters
            file_artist, file_title = self.formatter.format_filename(formatted_artist, formatted_title)
            new_file = f"{file_artist} - {file_title}{file.extension}"
            new_path = file.path / new_file

            if not new_path.is_file():
                # Rename files if flag was given or if tags were not changed
                if self.rename_files or not tag_changed:
                    if not track_printed:
                        print(f"{number}/{self.total_tracks}:")

                    print_bold("Rename file:", Color.yellow)
                    self.show_diff(file.filename, new_file)
                    self.num_renamed += 1
                    if not self.print_only and (self.force or self.confirm()):
                        os.rename(file.full_path, new_path)

                    print("-" * len(new_file))

    @staticmethod
    def get_tags_from_filename(filename: str) -> (str, str):
        if " - " not in filename:
            print_error(f"Can't parse tag data from malformed filename: {filename}")
            return "", ""

        artist, title = filename.split(" - ", 1)
        return artist, title

    @staticmethod
    def confirm() -> bool:
        """
        Ask user to confirm action.
        Note: everything except 'n' is a yes.
        """
        ans = input("Proceed (*/n)? ").strip()
        return ans.lower() != "n"

    @staticmethod
    def show_diff(old: str, new: str) -> None:
        """Print a stacked diff of the changes."""
        # http://stackoverflow.com/a/788780
        sequence = difflib.SequenceMatcher(None, old, new)
        diff_old = []
        diff_new = []
        for opcode, i1, i2, j1, j2 in sequence.get_opcodes():
            match opcode:
                case "equal":
                    diff_old.append(old[i1:i2])
                    diff_new.append(new[j1:j2])
                case "insert":
                    # use background color for whitespace changes
                    diff_new.append(
                        get_color(new[j1:j2], colorama.Back.GREEN if not new[j1:j2].strip() else Color.green)
                    )
                case "delete":
                    diff_old.append(get_color(old[i1:i2], colorama.Back.RED if not old[i1:i2].strip() else Color.red))
                case "replace":
                    diff_old.append(get_color(old[i1:i2], Color.red))
                    diff_new.append(get_color(new[j1:j2], Color.green))

        old = "".join(diff_old)
        new = "".join(diff_new)
        print(old)
        print(new)

    def print_stats(self):
        print_bold("Finished", Color.green)
        print(f"Tags:   {self.num_tags_fixed}")
        print(f"Rename: {self.num_renamed}")


@click.command()
@click.help_option("-h", "--help")
@click.argument("directory", type=click.Path(exists=True, file_okay=False, dir_okay=True), default=".")
@click.option("--print", "-p", "print_only", is_flag=True, help="Only print changes")
@click.option("--rename", "-r", is_flag=True, help="Rename all audio files")
@click.option("--sort", "-s", is_flag=True, help="Sort audio files by name")
@click.option("--tags", "-t", is_flag=True, help="Only fix tags")
@click.option("--force", "-f", is_flag=True, help="Do not ask for confirmation")
def main(directory: str, rename: bool, sort: bool, print_only: bool, tags: bool, force: bool):
    """
    Check and rename audio files.

    DIRECTORY: Optional input directory for audio files to format.
    """
    filepath = Path(directory).resolve()

    try:
        Renamer(filepath, rename, sort, print_only, tags, force).run()
    except KeyboardInterrupt:
        click.echo("\ncancelled...")


if __name__ == "__main__":
    main()
