from pathlib import Path


class Track:
    def __init__(self, name: str, extension: str, path: Path):
        self.name: str = name
        self.extension: str = extension
        self.path: Path = path

        if not self.extension.startswith("."):
            self.extension = f".{self.extension}"

    def is_mp3(self) -> bool:
        return self.extension == ".mp3"

    @property
    def filename(self):
        return self.name + self.extension

    @property
    def full_path(self):
        return self.path / self.filename

    @property
    def full_path_without_extension(self):
        return self.path / self.name

    def __eq__(self, other):
        if isinstance(other, Track):
            return self.path == other.path and self.name == other.name
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
