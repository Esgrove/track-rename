import difflib
import os
import re
import sys
from pathlib import Path

import click
import taglib
from titlecase import titlecase

from colorprint import Color, get_color, print_bold, print_warn


class Track:
    def __init__(self, name: str, extension: str, path: Path):
        self.name: str = name
        self.extension: str = extension
        self.path: Path = path

        if self.extension[0] != ".":
            self.extension = "." + self.extension

    @property
    def filename(self):
        return self.name + self.extension

    @property
    def full_path(self):
        return self.path / self.filename

    def __eq__(self, other):
        if isinstance(other, Track):
            return self.name == other.name
        if isinstance(other, str):
            return self.name == other

        return NotImplemented

    def __ne__(self, other):
        return not self.__eq__(other)

    def __lt__(self, other):
        if isinstance(other, Track):
            return self.name < other.name
        if isinstance(other, str):
            return self.name < other

        return NotImplemented

    def __le__(self, other):
        return self.__lt__(other) or self.__eq__(other)

    def __gt__(self, other):
        return not self.__le__(other)

    def __ge__(self, other):
        return not self.__lt__(other)


class Renamer:
    def __init__(self, path: Path, rename_files: bool, sort_files: bool, print_only: bool, tags_only: bool):
        self.root: Path = path
        self.rename_files: bool = rename_files
        self.sort_files: bool = sort_files
        self.print_only: bool = print_only
        self.tags_only: bool = tags_only

        self.file_list: list[Track] = []
        self.file_formats = (".mp3", ".flac", ".aif", ".aiff", ".m4a", ".mp4", ".wav")
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
            ("!!!", ""),
            ("...", " "),
        )
        self.title_substitutes = (
            (" (Original Mix)", ""),
            ("(DJcity - ", "("),
            ("DJcity ", ""),
            (" DJcity", ""),
            ("DJCity ", ""),
            (" DJCity", ""),
            ('12"', "12''"),
            ("Intro - Dirty", "Dirty Intro"),
            ("Intro - Clean", "Clean Intro"),
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
        self.gather_files()
        self.process_files()
        self.print_stats()

    def gather_files(self) -> None:
        file_list: list[Track] = []
        print_bold(f"Getting audio files from {get_color(str(self.root), color=Color.cyan)}")
        for file in self.root.rglob("*"):
            if file.suffix in self.file_formats:
                file_list.append(Track(file.stem, file.suffix, file.parent))

        if not file_list:
            sys.exit("no audio files found!")

        self.total_tracks = len(file_list)
        self.file_list = file_list

        if self.sort_files:
            self.file_list.sort()

    def process_files(self) -> None:
        print_bold(f"Checking {self.total_tracks} tracks...")
        current_path = self.root
        for number, file in enumerate(self.file_list):
            if not self.sort_files:
                # Print current directory when iterating in directory order
                if current_path != file.path:
                    current_path = file.path
                    print_bold(str(current_path), Color.magenta)

            # Check tags
            tag_data = taglib.File(file.full_path)
            if not tag_data.tags.get("ARTIST") or not tag_data.tags.get("TITLE"):
                print_warn(f"Missing tags: {file.full_path}")
                continue

            artist = "".join(tag_data.tags["ARTIST"])
            title = "".join(tag_data.tags["TITLE"])

            current_tags = f"{artist} - {title}"
            artist, title = self.format_track(artist, title)
            new_tags = f"{artist} - {title}"

            tag_changed = False
            track_printed = False
            if current_tags != new_tags:
                print(f"{number}/{self.total_tracks}:")
                track_printed = True
                print_bold("Fix tags:", Color.blue)
                self.show_diff(current_tags, new_tags)
                self.num_tags_fixed += 1
                if not self.print_only and self.confirm():
                    tag_data.tags["ARTIST"] = [artist]
                    tag_data.tags["TITLE"] = [title]
                    tag_data.save()
                    tag_changed = True

                print("-" * len(current_tags))

            tag_data.close()

            if self.tags_only:
                continue

            # Check file name
            # Remove forbidden characters
            file_artist = re.sub('[\\/:"*?<>|]+', "", artist).strip()
            file_title = re.sub('[\\/:"*?<>|]+', "", title).strip()
            file_artist = re.sub(r"\s+", " ", file_artist)
            file_title = re.sub(r"\s+", " ", file_title)
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

                    print("-" * len(file.filename))


    def format_track(self, artist: str, title: str) -> (str, str):
        """Return formatted artist and title string."""
        if artist.islower():
            artist = titlecase(artist)

        if title.islower():
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

        if title.endswith("."):
            title = title[:-1]

        title = self.balance_parenthesis(title)
        title = self.wrap_text_after_parentheses(title)

        # Double check whitespace
        artist = artist.strip()
        title = title.strip()
        artist = re.sub(r"\s+", " ", artist)
        title = re.sub(r"\s+", " ", title)

        return artist, title

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
    def use_parenthesis_for_mix(title: str) -> str:
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
        if "feat. " in title:
            start = title.index("feat. ")
            end = len(title)
            if " (" in title[start:]:
                end = title.index(" (")
            if " -" in title[start:]:
                new = title.index(" -")
                end = min(end, new)
            if ")" in title[start:]:
                new = title.index(")")
                end = min(end, new)

            feat = title[start:end]
            if feat:
                other = " ".join(feat.split()[1:])
                if other in artist:
                    if f", {other}" in artist:
                        artist = artist.replace(f", {other}", "")
                    elif f" & {other}" in artist:
                        artist = artist.replace(f" & {other}", "")
                if feat not in artist:
                    artist += " " + feat

                title = title[:start] + title[end:]

        title = title.replace("((", "(")
        title = title.replace("))", ")")
        return artist, title

    @staticmethod
    def confirm() -> bool:
        ans = input("Proceed (*/n)? ").strip()
        return ans.lower() != "n"

    @staticmethod
    def show_diff(old: str, new: str) -> None:
        # http://stackoverflow.com/a/788780
        sequence = difflib.SequenceMatcher(None, old, new)
        diff_old = []
        diff_new = []
        for opcode, a0, a1, b0, b1 in sequence.get_opcodes():
            if opcode == "equal":
                diff_old.append(old[a0:a1])
                diff_new.append(new[b0:b1])
            elif opcode == "insert":
                diff_new.append(get_color(new[b0:b1], Color.green))
            elif opcode == "delete":
                diff_old.append(get_color(old[a0:a1], Color.red))
            elif opcode == "replace":
                diff_old.append(get_color(old[a0:a1], Color.red))
                diff_new.append(get_color(new[b0:b1], Color.green))
            else:
                raise RuntimeError("unexpected diff opcode")

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
        if text.endswith(")"):
            return text

        if text.startswith("("):
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

    DIRECTORY: Optional input directory for audio files.
    """
    filepath = Path(directory).resolve()

    try:
        Renamer(filepath, rename, sort, print_only, tags).run()
    except KeyboardInterrupt:
        click.echo("\ncancelled...")


if __name__ == "__main__":
    main()
