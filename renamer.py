import difflib
import os
import re
import sys
from pathlib import Path

import click
import colorama
import taglib
from titlecase import titlecase

from colorprint import Color, get_color, print_bold, print_color


class Track:
    def __init__(self, name: str, extension: str, path: Path):
        self.name: str = name
        self.extension: str = extension
        self.path: Path = path
        if self.extension[0] != ".":
            self.extension = "." + self.extension

        self.differ = difflib.Differ()

    @property
    def filename(self):
        return self.name + self.extension

    @property
    def full_path(self):
        return self.path / self.filename


class Renamer:
    def __init__(self, path: Path):
        self.root: Path = path
        self.file_list = []
        self.file_formats = (".mp3", ".flac", ".aif", ".aiff", ".m4a", ".mp4", ".wav")
        self.re_substitutes = (
            (r"[\[{]+", "("),
            (r"[\]}]+", ")"),
            (r"\t", " "),
            (r"\n", " "),
            (r"\r\n", " "),
            (r"\r", " "),
            (r"\s{2,}", " "),
            (r"\.{2,}", "."),
            (r"\(\s*?\)", ""),
        )
        self.total_tracks = 0
        self.print = False

        self.gather_files()
        self.track_rename()

    def gather_files(self):
        file_list = []
        print_bold(f"Getting audio files from {get_color(str(self.root), color=Color.cyan)}")
        for file in self.root.rglob("*"):
            if file.suffix in self.file_formats:
                file_list.append(Track(file.stem, file.suffix, file.parent))

        if not file_list:
            sys.exit("no audio files found!")

        self.total_tracks = len(file_list)
        print(f"Found {self.total_tracks} tracks.\n")
        self.file_list = file_list

    def track_rename(self):
        print_bold("Renaming tracks...")
        current_path = ""
        for number, file in enumerate(self.file_list):
            if current_path != file.path:
                current_path = file.path
                print_color(current_path, Color.yellow)

            tag_data = taglib.File(file.full_path)
            if not tag_data.tags.get("ARTIST") or not tag_data.tags.get("TITLE"):
                continue

            artist = "".join(tag_data.tags["ARTIST"])
            title = "".join(tag_data.tags["TITLE"])

            current_tags = artist + " - " + title
            artist, title = self.process_track(artist, title)

            new_tags = artist + " - " + title
            if current_tags != new_tags:
                self.check_print(number)
                print_bold("Fix tags:", Color.blue)
                self.show_diff(current_tags, new_tags)
                if self.confirm():
                    tag_data.tags["ARTIST"] = [artist]
                    tag_data.tags["TITLE"] = [title]
                    tag_data.save()
                print("-------------------------------")

            tag_data.close()

            file_artist = re.sub('[\\/:"*?<>|]+', "", artist).strip()
            file_title = re.sub('[\\/:"*?<>|]+', "", title).strip()
            new_file = file_artist + " - " + file_title + file.extension
            new_path = file.path / new_file

            if not new_path.is_file():
                self.check_print(number)
                print_bold("Rename file:", Color.yellow)
                self.show_diff(file.filename, new_file)
                if self.confirm():
                    os.rename(file.full_path, new_path)
                print("-------------------------------")

            self.print = False

    def process_track(self, artist, title):
        if " - " in title and not re.search(r"\([^()]+-[^()]+\)", title):
            index = title.index(" - ")
            if " (" in title[index:]:
                title = title[:index] + title[index:].replace(" (", ") (", 1)
            else:
                title += ")"
            title = title.replace(" - ", " (", 1)

        if " (Original Mix)" in title:
            title = title.replace(" (Original Mix)", "")
        if "DJcity " in title:
            title = title.replace("DJcity ", "")
        if " DJcity" in title:
            title = title.replace(" DJcity", "")
        if "DJCity " in title:
            title = title.replace("DJCity ", "")
        if " DJCity" in title:
            title = title.replace(" DJCity", "")
        if '12"' in title:
            title = title.replace('12"', "12''")

        if artist.islower():
            artist = titlecase(artist)

        if title.islower():
            title = titlecase(title)

        artist = artist.replace(" feat ", " feat. ").replace(" ft. ", " feat. ").replace(" Feat ", " feat. ")
        title = title.replace(" feat ", " feat. ").replace(" ft. ", " feat. ").replace(" Feat ", " feat. ")
        if " feat. " in title:
            start = title.index(" feat. ")
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
                    artist += feat
                title = title[:start] + title[end:]

        if "((" in title:
            title = title.replace("((", "(")
        if "))" in title:
            title = title.replace("))", ")")
        if "..." in title:
            title = title.replace("...", " ")

        for pattern, sub in self.re_substitutes:
            artist = re.sub(pattern, sub, artist)
            title = re.sub(pattern, sub, title)

        # Regular expression pattern: any non-whitespace character followed by '('
        pattern = r"(\S)\("
        # Replacement: the matched non-whitespace character, a space, then '('
        replacement = r"\1 ("
        # Perform the replacement
        re.sub(pattern, replacement, title)

        artist = artist.strip()
        title = title.strip()
        if title.endswith("."):
            title = title[:-1]

        # Check parenthesis
        open_count = title.count("(")
        close_count = title.count(")")
        if open_count > close_count:
            title = self.add_missing_closing_parentheses(title)
        elif open_count < close_count:
            title = self.add_missing_opening_parentheses(title)

        title = title.replace(")(", ") (")
        title = title.replace(" )", ")")
        title = title.replace("( ", "(")

        title = self.wrap_text_after_parentheses(title)

        return artist, title

    def check_print(self, number: int):
        if not self.print:
            print(f"{number}/{self.total_tracks}:")
            self.print = True

    @staticmethod
    def confirm() -> bool:
        ans = input("Proceed (*/n)? ").strip()
        return ans.lower() != "n"

    @staticmethod
    def show_diff(old, new):
        # http://stackoverflow.com/a/788780
        sequence = difflib.SequenceMatcher(None, old, new)
        diff_old = []
        diff_new = []
        for opcode, a0, a1, b0, b1 in sequence.get_opcodes():
            if opcode == "equal":
                diff_old.append(old[a0:a1])
                diff_new.append(new[b0:b1])
            elif opcode == "insert":
                diff_new.append(get_color(new[b0:b1], colorama.Back.GREEN))
            elif opcode == "delete":
                diff_old.append(get_color(old[a0:a1], colorama.Back.RED))
            elif opcode == "replace":
                diff_old.append(get_color(old[a0:a1], colorama.Back.RED))
                diff_new.append(get_color(new[b0:b1], colorama.Back.GREEN))
            else:
                raise RuntimeError("unexpected opcode")

        old = "".join(diff_old)
        new = "".join(diff_new)
        if old != new:
            print(old)
            print(new)

    @staticmethod
    def add_missing_closing_parentheses(text):
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
    def add_missing_opening_parentheses(text):
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
    def wrap_text_after_parentheses(text):
        if text.endswith(")"):
            return text

        if text.startswith("("):
            return text

        # Regex pattern to match text after the last closing parenthesis
        # The negative lookahead (?!.*\() ensures no opening parenthesis follows
        pattern = r"(\([^)]*\))(?!.*\()\s(.+)"

        # Using regex substitution to wrap the text after the last closing parenthesis
        return re.sub(pattern, r"\1 (\2)", text)


@click.command()
@click.argument(
    "directory",
    type=click.Path(exists=True, file_okay=False, dir_okay=True),
    default=".",
)
def main(directory):
    """Check and rename audio files."""
    filepath = Path(directory).resolve()

    try:
        Renamer(filepath)
    except KeyboardInterrupt:
        click.echo("\ncancelled...")


if __name__ == "__main__":
    main()
