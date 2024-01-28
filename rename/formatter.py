import re

from titlecase import titlecase


class TrackFormatter:
    """Handle track formatting."""

    def __init__(self):
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
            ("..", " "),
            (" ***", ""),
            (" **", ""),
            (" * ", ""),
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
            ("(12 Inch ", "(12'' "),
            (" 12in ", " 12'' "),
            ("(12in ", "(12'' "),
            ("(7in ", "(7'' "),
            (" 7in ", " 7'' "),
            ("Intro/Outro", "Intro-Outro"),
            (" In/Out", " Intro-Outro"),
            ("In/Out ", "Intro-Outro "),
            ("Aca In/Aca Out", "Acapella In-Out"),
            ("Intro/Outro", "Intro"),
            ("Intro-Outro", "Intro"),
            ("In+Out", "In-Out"),
            ("In+out", "In-Out"),
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
        self.filename_regex_substitutes = (
            ('"', "''"),
            ("[<>|!]+", ""),
            (r"[\\/:\*\?]", "-"),
            (r"\s+", " "),
        )

    def format_tags(self, artist: str, title: str) -> (str, str):
        """Return formatted artist and title string."""
        artist = artist.strip()
        title = title.strip()
        if not artist and not title:
            return artist, title

        # check if artist name is duplicated
        if title.startswith(f"{artist} - "):
            title = title.replace(f"{artist} - ", "", 1)

        if artist.islower() or (artist.isupper() and len(artist) > 12):
            artist = titlecase(artist)

        if title.islower() or (title.isupper() and len(title) > 12):
            title = titlecase(title)

        if " clean" in title.lower():
            title = self.replace_clean_suffix(title)
        if " dirty" in title.lower():
            title = self.replace_dirty_suffix(title)

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

        title = self.fix_nested_parentheses(title)
        title = self.balance_parenthesis(title)
        title = self.wrap_text_after_parentheses(title)
        title = self.remove_bpm_in_parentheses_from_end(title)

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
        file_artist = artist.strip()
        file_title = title.strip()
        for pattern, replacement in self.filename_regex_substitutes:
            file_artist = re.sub(pattern, replacement, file_artist)
            file_title = re.sub(pattern, replacement, file_title)

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
    def replace_dirty_suffix(text: str) -> str:
        pattern = re.compile(r"\s*-\s*dirty$", re.IGNORECASE)
        # Replace the matched part with " (Clean)"
        return pattern.sub(" (Dirty)", text)

    @staticmethod
    def replace_clean_suffix(text: str) -> str:
        pattern = re.compile(r"\s*-\s*clean$", re.IGNORECASE)
        # Replace the matched part with " (Clean)"
        return pattern.sub(" (Clean)", text)

    @staticmethod
    def fix_nested_parentheses(text: str) -> str:
        # Initialize a stack to keep track of parentheses
        stack = []
        result = []

        for char in text:
            if char == "(":
                # If the stack is not empty and the top element is also '(', add a closing ')' before the new '('
                if stack and stack[-1] == "(":
                    result.append(") ")
                stack.append(char)
                result.append(char)
            elif char == ")":
                # If the stack is not empty, pop an element from the stack
                if stack:
                    stack.pop()
                # Add the closing parenthesis only if the stack is empty or the top element is not '('
                if not stack or stack[-1] != "(":
                    result.append(char)
            else:
                # Add any other characters to the result
                result.append(char)

        # If there are any remaining opening parentheses, close them
        while stack:
            stack.pop()
            result.append(")")

        return "".join(result).replace(" )", ")").replace("( ", "(")

    @staticmethod
    def remove_bpm_in_parentheses_from_end(text):
        """Remove a BPM and key from the end of the string."""
        pattern = r" \((\d{2,3}(\.\d)?|\d{2,3} \d{1,2}a)\)$"
        result = re.sub(pattern, "", text)

        pattern = r"\s\(\d{1,2}(?:\s\d{1,2})?\s?[a-zA-Z]\)$"
        result = re.sub(pattern, "", result)

        pattern = r"\s\(\d{2,3}\s?[a-zA-Z]{2,3}\)$"
        result = re.sub(pattern, "", result)

        return result

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
