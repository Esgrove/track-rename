#!/usr/bin/env python3

import datetime
import difflib
import hashlib
import os
import re
import sys
import time
from pathlib import Path

import click
import colorama

try:
    from colorprint import Color, get_color, print_bold, print_error, print_red, print_warn, print_yellow
    from formatter import TrackFormatter
    from track import Track
except ModuleNotFoundError:
    # poetry run needs the full import path
    from rename.colorprint import Color, get_color, print_bold, print_error, print_red, print_warn, print_yellow
    from rename.formatter import TrackFormatter
    from rename.track import Track

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
        self.num_removed = 0
        self.num_duplicates = 0

        self.formatter = TrackFormatter()
        self.processed: dict[str, Track] = dict()

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
            sys.exit("No audio files found")

        self.total_tracks = len(file_list)

        if self.sort_files:
            file_list.sort()

        self.file_list = file_list

    def process_files(self) -> None:
        """Format all tracks."""
        print_bold(f"Formatting {self.total_tracks} tracks...")
        current_root = self.root

        for number, track in enumerate(self.file_list):
            track.number = number
            if not self.sort_files:
                # Print current directory when iterating in directory order
                if current_root != track.root:
                    current_root = track.root
                    print_bold(str(current_root), Color.magenta)

            # Might have deleted or renamed a duplicate track earlier
            if not track.full_path.exists():
                print_red(f"File no longer exists: '{track.full_path}'")
                continue

            self.process_track(track)

    def process_track(self, track: Track) -> None:
        """Format tags and rename one track."""
        track = self.process_tags(track)
        if self.tags_only or not track:
            return

        self.rename_track(track)

    def process_tags(self, track) -> Track | None:
        """Read and format tags."""
        attempts = 0
        tag_data = None
        while attempts < 2 and not tag_data:
            try:
                tag_data = taglib.File(track.full_path)
            except OSError as e:
                print_red(str(e))
                attempts += 1
                time.sleep(1)

        if not tag_data:
            print_error(f"Failed to load tags for: '{track.full_path}'")
            return None

        artist = "".join(tag_data.tags.get("ARTIST", []))
        title = "".join(tag_data.tags.get("TITLE", []))

        if not artist and not title:
            print_warn(f"Missing tags: {track.full_path}")
            artist, title = self.get_tags_from_filename(track.name)
        elif not artist:
            print_warn(f"Missing artist tag: {track.full_path}")
            artist, _ = self.get_tags_from_filename(track.name)
        elif not title:
            print_warn(f"Missing title tag: {track.full_path}")
            _, title = self.get_tags_from_filename(track.name)

        track.artist = artist
        track.title = title

        formatted_artist, formatted_title = self.formatter.format_tags(artist, title)

        track.formatted_artist = formatted_artist
        track.formatted_title = formatted_title

        if track.original_tags != track.formatted_tags:
            track.show(self.total_tracks)
            print_bold("Fix tags:", Color.blue)
            self.show_diff(track.original_tags, track.formatted_tags)
            self.num_tags_fixed += 1
            if not self.print_only and (self.force or self.confirm()):
                tag_data.tags["ARTIST"] = [formatted_artist]
                tag_data.tags["TITLE"] = [formatted_title]
                tag_data.save()
                track.tag_changed = True

            self.print_divider(track.formatted_tags)

        tag_data.close()

        return track

    def rename_track(self, track: Track) -> None:
        """Format filename and rename if needed."""
        file_artist, file_title = self.formatter.format_filename(track.formatted_artist, track.formatted_title)
        formatted_name = f"{file_artist} - {file_title}"
        formatted_filename = f"{formatted_name}{track.formatted_extension}"
        new_path = track.root / formatted_filename

        if not new_path.exists():
            # Rename file if rename flag was given or if tags were not changed
            if self.rename_files or not track.tags_updated:
                track.show(self.total_tracks)
                print_yellow("Rename file:", bold=True)
                self.show_diff(track.filename, formatted_filename)
                if not self.print_only and (self.force or self.confirm()):
                    self.try_rename(track.full_path, new_path)
                    track.renamed = True

                self.num_renamed += 1
                self.print_divider(formatted_filename)
        elif new_path != track.full_path:
            # This file is a duplicate of an existing file in the same root directory
            track.show(self.total_tracks)
            print_red("Duplicate:", bold=True)
            print(formatted_filename)
            if not self.print_only and (self.force or self.confirm("Delete duplicate")):
                track.full_path.unlink()
                self.num_removed += 1
            else:
                print_yellow("Marking as duplicate...")
                current_duplicate_path = self.append_duplicate_tag_to_name(new_path)
                new_duplicate_path = self.append_duplicate_tag_to_name(track.full_path)
                if not current_duplicate_path.exists():
                    self.try_rename(new_path, current_duplicate_path)
                if not new_duplicate_path.exists():
                    self.try_rename(track.full_path, new_duplicate_path)

                self.num_duplicates += 1

            self.print_divider(formatted_filename)
            return

        self.check_for_duplicates(track, formatted_name, new_path)

    def check_for_duplicates(self, track: Track, formatted_name: str, new_path: Path):
        updated_track = Track(new_path.stem, new_path.suffix, new_path.parent) if track.renamed else track
        if formatted_name in self.processed and self.processed[formatted_name].full_path.exists():
            track.show(self.total_tracks)
            existing_track = self.processed[formatted_name]
            if existing_track.extension == updated_track.extension:
                print_red("Duplicate:", bold=True)
                print(existing_track.full_path)
                print(updated_track.full_path)

                print_yellow("Marking as duplicates...")
                existing_duplicate_path = self.append_duplicate_tag_to_name(existing_track.full_path)
                updated_duplicate_path = self.append_duplicate_tag_to_name(updated_track.full_path)

                self.try_rename(existing_track.full_path, existing_duplicate_path)
                self.try_rename(updated_track.full_path, updated_duplicate_path)

                self.num_duplicates += 1
            else:
                print_red("Multiple formats:", bold=True)
                print(existing_track.full_path)
                print(updated_track.full_path)
                if existing_track.is_mp3():
                    if not self.print_only and (self.force or self.confirm(f"Delete {existing_track.extension}")):
                        existing_track.full_path.unlink()
                        self.processed[formatted_name] = updated_track
                        self.num_removed += 1
                else:
                    if not self.print_only and (self.force or self.confirm(f"Delete {updated_track.extension}")):
                        updated_track.full_path.unlink()
                        self.num_removed += 1

            self.print_divider(str(existing_track.full_path))
        else:
            self.processed[formatted_name] = updated_track

    @staticmethod
    def get_tags_from_filename(filename: str) -> (str, str):
        """
        Convert filename to artist and title tags.
        Expects filename to be in format 'artist - title'.
        """
        if " - " not in filename:
            print_error(f"Can't parse tag data from malformed filename: {filename}")
            return "", ""

        artist, title = filename.split(" - ", 1)
        return str(artist).strip(), str(title).strip()

    @staticmethod
    def append_duplicate_tag_to_name(filepath: Path) -> Path:
        """
        Add a duplicate string with a short hash based on current timestamp to
        create a unique identifier.
        """
        if re.search(r"(Duplicate-[A-Za-z0-9]+)", filepath.stem):
            return filepath

        current_datetime = datetime.datetime.now()
        datetime_string = current_datetime.isoformat(timespec="milliseconds")
        hash_object = hashlib.sha256(datetime_string.encode())
        hash_digest = hash_object.hexdigest()
        new_file_name = f"{filepath.stem} (Duplicate-{hash_digest[:8]}){filepath.suffix}"
        return filepath.with_name(new_file_name)

    @staticmethod
    def confirm(message="Proceed") -> bool:
        """
        Ask user to confirm action.
        Note: everything except 'n' is a yes.
        """
        ans = input(f"{message} (*/n)? ").strip()
        return ans.lower() != "n"

    @staticmethod
    def show_diff(old: str, new: str) -> None:
        """Print a stacked colorized diff of the changes."""
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

    @staticmethod
    def try_rename(path: Path, new_path: Path):
        try:
            if path.exists() and not new_path.exists():
                os.rename(path, new_path)
        except Exception as e:
            print_red(f"Rename failed: {e}")

    def print_stats(self) -> None:
        """Print number of changes made."""
        print_bold("Finished", Color.green)
        print(f"Tags:      {self.num_tags_fixed}")
        print(f"Rename:    {self.num_renamed}")
        print(f"Delete:    {self.num_removed}")
        print(f"Duplicate: {self.num_duplicates}")

    @staticmethod
    def print_divider(for_text: str) -> None:
        print("-" * len(str(for_text)))


@click.command()
@click.help_option("-h", "--help")
@click.argument("directory", type=click.Path(exists=True, file_okay=False, dir_okay=True), default=".")
@click.option("--force", "-f", is_flag=True, help="Do not ask for confirmation")
@click.option("--print", "-p", "print_only", is_flag=True, help="Only print changes")
@click.option("--rename", "-r", is_flag=True, help="Rename all audio files")
@click.option("--sort", "-s", is_flag=True, help="Sort audio files by name")
@click.option("--tags", "-t", is_flag=True, help="Only fix tags, do not rename")
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
