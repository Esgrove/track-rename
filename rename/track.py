from pathlib import Path


class Track:
    def __init__(self, name: str, extension: str, path: Path, number: int | None = None):
        self.name: str = name
        self.extension: str = extension
        self.root: Path = path
        self.number: int | None = number

        self.artist: str = ""
        self.title: str = ""
        self.formatted_artist: str = ""
        self.formatted_title: str = ""

        self.tags_updated: bool = False
        self.renamed: bool = False
        self.printed: bool = False

        if not self.extension.startswith("."):
            self.extension = f".{self.extension}"

    def show(self, total_tracks: int):
        """Print track if it has not been already."""
        if not self.printed:
            print(f"{self.number}/{total_tracks}:")
            self.printed = True

    def is_mp3(self) -> bool:
        return self.extension == ".mp3"

    def is_aif(self) -> bool:
        return self.extension == ".aif" or self.extension == ".aiff"

    @property
    def original_tags(self):
        return f"{self.artist} - {self.title}"

    @property
    def formatted_tags(self):
        return f"{self.formatted_artist} - {self.formatted_title}"

    @property
    def formatted_extension(self):
        return ".aif" if self.extension.lower() == ".aiff" else self.extension.lower()

    @property
    def filename(self):
        return self.name + self.extension

    @property
    def full_path(self):
        return self.root / self.filename

    @property
    def full_path_without_extension(self):
        return self.root / self.name

    def __eq__(self, other):
        if isinstance(other, Track):
            return self.name == other.name
        if isinstance(other, str):
            return self.name == other

        return NotImplemented

    def __hash__(self):
        return hash(self.full_path_without_extension)

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
