#!/usr/bin/env python3

import difflib
import os
import re
import sys
from pathlib import Path

import click
import colorama
from titlecase import titlecase

try:
    from colorprint import Color, get_color, print_bold, print_error, print_warn, print_yellow
    from track import Track
except ModuleNotFoundError:
    from rename.colorprint import Color, get_color, print_bold, print_error, print_warn, print_yellow
    from rename.track import Track

try:
    # Workaround to be able to run tests on Apple Silicon while pytaglib is broken
    import taglib
except ImportError:
    taglib = None


class Renamer:
    """Audio track tag and filename formatting."""

    def __init__(self, path: Path, rename_files: bool, sort_files: bool, print_only: bool, tags_only: bool):
        self.root: Path = path
        self.rename_files: bool = rename_files
        self.sort_files: bool = sort_files
        self.print_only: bool = print_only
        self.tags_only: bool = tags_only

        self.file_list: list[Track] = []
        self.file_formats = (".mp3", ".flac", ".aif", ".aiff", ".m4a")
        self.total_tracks = 0
        self.num_renamed = 0
        self.num_tags_fixed = 0
        self.common_substitutes = (
            (" feat ", " feat. "),
            (" ft. ", " feat. "),
            (" Feat ", " feat. "),
            (" featuring ", " feat. "),
            (" Featuring ", " feat. "),
            ("(feat ", "(feat. "),
            ("(ft. ", "(feat. "),
            ("(Feat ", "(feat. "),
            ("(featuring ", "(feat. "),
            ("(Featuring ", "(feat. "),
            (") - (", ""),
            (" - (", " ("),
            ("(- ", "("),
            ("( - ", "("),
            (" -)", " )"),
            (" - ) ", ")"),
            ("!!!", ""),
            ("...", " "),
        )
        self.title_substitutes = (
            (" (Original Mix)", ""),
            (" DJcity", ""),
            (" DJCity", ""),
            ("(DJcity - ", "("),
            ("DJcity ", ""),
            ("DJCity ", ""),
            ('12"', "12''"),
            ("Intro - Dirty", "Dirty Intro"),
            ("Intro - Clean", "Clean Intro"),
            ("Acap - DIY", "Acapella DIY"),
            ("(Acap)", "(Acapella)"),
            ("Acap ", "Acapella "),
            ("(Inst)", "(Instrumental)"),
            (" 12 Inch ", " 12'' "),
        )
        self.regex_substitutes = (
            (r"[\[{]+", "("),
            (r"[\]}]+", ")"),
            (r"\s+", " "),
            (r"\s{2,}", " "),
            (r"\.{2,}", "."),
            (r"\(\s*?\)", ""),
            (r"(\S)\(", r"\1 ("),
        )

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

            formatted_artist, formatted_title = self.format_track(artist, title)
            new_tags = f"{formatted_artist} - {formatted_title}"

            tag_changed = False
            track_printed = False
            if current_tags != new_tags:
                print(f"{number}/{self.total_tracks}:")
                track_printed = True
                print_bold("Fix tags:", Color.blue)
                self.show_diff(current_tags, new_tags)
                self.num_tags_fixed += 1
                if not self.print_only and self.confirm():
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
            file_artist, file_title = self.format_filename(formatted_artist, formatted_title)
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
                    if not self.print_only and self.confirm():
                        os.rename(file.full_path, new_path)

                    print("-" * len(new_file))

    def format_track(self, artist: str, title: str) -> (str, str):
        """Return formatted artist and title string."""
        artist = artist.strip()
        title = title.strip()
        if not artist and not title:
            return artist, title

        # check if artist name is duplicated
        if title.startswith(f"{artist} - "):
            title = title.replace(f"{artist} - ", "", 1)

        if artist.islower():
            artist = titlecase(artist)

        if title.islower() or (title.isupper() and len(title) > 5):
            title = titlecase(title)

        for pattern, replacement in self.common_substitutes:
            artist = artist.replace(pattern, replacement)
            title = title.replace(pattern, replacement)

        for pattern, replacement in self.title_substitutes:
            title = title.replace(pattern, replacement)

        for pattern, replacement in self.regex_substitutes:
            artist = re.sub(pattern, replacement, artist)
            title = re.sub(pattern, replacement, title)

        title = self.use_parenthesis_for_mix(title)

        artist, title = self.move_feat_from_title_to_artist(artist, title)

        title = self.balance_parenthesis(title)
        title = self.wrap_text_after_parentheses(title)

        if title.endswith("."):
            title = title[:-1]

        # Double-check whitespace
        artist = artist.strip()
        artist = re.sub(r"\s+", " ", artist)
        artist = artist.replace(" )", ")").replace("( ", "(")

        title = title.strip()
        title = re.sub(r"\s+", " ", title)
        title = title.replace(" )", ")").replace("( ", "(")

        return artist, title

    def format_filename(self, artist: str, title: str) -> (str, str):
        """Return formatted artist and title string for filename."""
        # Remove forbidden characters
        file_artist = re.sub('[\\/"<>|]+', "", artist).strip()
        file_artist = re.sub(r"[:\*\?]", "-", file_artist)
        file_artist = re.sub(r"\s+", " ", file_artist)

        file_title = re.sub('[\\/"<>|]+', "", title).strip()
        file_title = re.sub(r"[:\*\?]", "-", file_title)
        file_title = re.sub(r"\s+", " ", file_title)

        return file_artist, file_title

    def balance_parenthesis(self, title):
        """Check parenthesis match and insert missing."""
        open_count = title.count("(")
        close_count = title.count(")")
        if open_count > close_count:
            title = self.add_missing_closing_parentheses(title)
        elif open_count < close_count:
            title = self.add_missing_opening_parentheses(title)

        title = title.replace(")(", ") (")
        title = title.replace(" )", ")")
        title = title.replace("( ", "(")
        title = title.replace("()", "")
        return title

    @staticmethod
    def get_tags_from_filename(filename: str) -> (str, str):
        if " - " not in filename:
            print_error(f"Can't parse tag data from malformed filename: {filename}")
            return "", ""

        artist, title = filename.split(" - ", 1)
        return artist, title

    @staticmethod
    def use_parenthesis_for_mix(title: str) -> str:
        """Wrap the mix version in parentheses."""
        # Fix DJCity formatting style for Remix / Edit
        if " - " in title and not re.search(r"\([^()]+-[^()]+\)", title):
            index = title.index(" - ")
            if " (" in title[index:]:
                title = title[:index] + title[index:].replace(" (", ") (", 1)
            else:
                title += ")"
            title = title.replace(" - ", " (", 1)

        return title

    @staticmethod
    def move_feat_from_title_to_artist(artist: str, title: str) -> (str, str):
        """Move featuring artist(s) to the artist field and remove duplicate info."""
        if " feat. " in title or "(feat. " in title:
            feat_match = re.search(r"feat\. .*?(?=( -|\(|\)|$))", title)
            if feat_match:
                feat = feat_match.group()
                title = title.replace(feat, "")

                # Get artist names without "feat". Join is used to remove extra whitespace.
                feat_artist = " ".join(feat.split()[1:])
                feat_artist = feat_artist.replace(", and ", " & ").replace(" and ", " & ")

                # Remove duplicate feat artist names from the artist string
                for delimiter in [", ", " & ", " and ", " + "]:
                    artist = artist.replace(f"{delimiter}{feat_artist}", "").replace(f"{feat_artist}{delimiter}", "")

                new_feat = f" feat. {feat_artist}"
                if new_feat not in artist:
                    artist += new_feat

        # Replace ' - - ' or ' - ' inside parentheses
        title = re.sub(
            r"\([^)]*( - - | - )[^)]*\)", lambda m: m.group().replace(" - - ", ") (").replace(" - ", ") ("), title
        )

        title = title.replace("((", "(")
        title = title.replace("))", ")")
        title = title.replace("(- ", "(")
        title = title.replace("( - ", "(")
        title = title.replace(" -)", ")")
        title = title.replace(" - )", ")")
        title = title.replace("()", "")
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

    @staticmethod
    def add_missing_closing_parentheses(text: str) -> str:
        open_count = 0
        result = []

        for char in text:
            if char == "(":
                # If there are unclosed parentheses before opening a new one, close them
                if open_count > 0:
                    result.append(") ")
                    open_count = max(0, open_count - 1)
                else:
                    open_count += 1
            elif char == ")":
                open_count = max(0, open_count - 1)

            result.append(char)

        # Add any remaining closing parentheses at the end
        if open_count > 0:
            result.append(")")

        return "".join(result)

    @staticmethod
    def add_missing_opening_parentheses(text: str) -> str:
        open_count = 0
        result = []

        for char in reversed(text):
            if char == ")":
                if open_count > 0:
                    result.append(" (")
                    open_count = max(0, open_count - 1)
                else:
                    open_count += 1
            elif char == "(":
                open_count = max(0, open_count - 1)

            result.append(char)

        if open_count > 0:
            result.append("(")

        return "".join(reversed(result))

    @staticmethod
    def wrap_text_after_parentheses(text: str) -> str:
        """Add parentheses around text following text in parentheses."""
        if text.endswith(")") or text.startswith("("):
            return text

        # Regex pattern to match text after the last closing parenthesis
        # The negative lookahead (?!.*\() ensures no opening parenthesis follows
        pattern = r"(\([^)]*\))(?!.*\()\s(.+)"

        # Using regex substitution to wrap the text after the last closing parenthesis
        return re.sub(pattern, r"\1 (\2)", text)

    def print_stats(self):
        print_bold("Finished", Color.green)
        print(f"Tags:   {self.num_tags_fixed}")
        print(f"Rename: {self.num_renamed}")


@click.command()
@click.help_option("-h", "--help")
@click.argument("directory", type=click.Path(exists=True, file_okay=False, dir_okay=True), default=".")
@click.option("--print", "-p", "print_only", is_flag=True, help="Only print changes")
@click.option("--rename", "-r", is_flag=True, help="Rename audio files")
@click.option("--sort", "-s", is_flag=True, help="Sort audio files by name")
@click.option("--tags", "-t", is_flag=True, help="Only fix tags")
def main(directory: str, rename: bool, sort: bool, print_only: bool, tags: bool):
    """
    Check and rename audio files.

    DIRECTORY: Optional input directory for audio files to format.
    """
    filepath = Path(directory).resolve()

    try:
        Renamer(filepath, rename, sort, print_only, tags).run()
    except KeyboardInterrupt:
        click.echo("\ncancelled...")


if __name__ == "__main__":
    main()
