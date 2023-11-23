from pathlib import Path

import pytest

from rename.track import Track


def test_constructor():
    track = Track("song", ".mp3", Path("/user/test/music"))
    assert track.name == "song"
    assert track.extension == ".mp3"
    assert track.path == Path("/user/test/music")

    # Test automatic addition of dot in extension
    track = Track("song", "mp3", Path("/user/test/music"))
    assert track.extension == ".mp3"


def test_filename_property():
    track = Track("song", ".mp3", Path("/user/test/music"))
    assert track.filename == "song.mp3"


def test_full_path_property():
    track = Track("song", ".mp3", Path("/user/test/music"))
    assert track.full_path == Path("/user/test/music/song.mp3")


@pytest.mark.parametrize("extension", [".mp3", ".flac", ".aif", ".aiff", ".m4a"])
def test_various_extensions(extension):
    track = Track("song", extension, Path("/user/test/music"))
    assert track.extension == extension
    assert track.filename == "song" + extension
    assert track.full_path == Path("/user/test/music/song" + extension)


def test_eq_method():
    track1 = Track("song1", ".mp3", Path("/music"))
    track2 = Track("song1", ".mp3", Path("/different/path"))
    track3 = Track("song2", ".mp3", Path("/music"))

    # Test equality with another Track
    assert track1 == track2
    assert not (track1 == track3)

    # Test equality with a string
    assert track1 == "song1"
    assert not (track1 == "song2")


def test_ne_method():
    track1 = Track("song1", ".mp3", Path("/music"))
    track2 = Track("song2", ".mp3", Path("/music"))

    # Test inequality with another Track
    assert track1 != track2

    # Test inequality with a string
    assert track1 != "song2"


def test_lt_method():
    track1 = Track("song1", ".mp3", Path("/music"))
    track2 = Track("song2", ".mp3", Path("/music"))

    # Test less than with another Track
    assert track1 < track2

    # Test less than with a string
    assert track1 < "song2"


def test_le_method():
    track1 = Track("song1", ".mp3", Path("/music"))
    track2 = Track("song1", ".mp3", Path("/different/path"))
    track3 = Track("song2", ".mp3", Path("/music"))

    # Test less than or equal with another Track
    assert track1 <= track2  # Equal
    assert track1 <= track3  # Less than

    # Test less than or equal with a string
    assert track1 <= "song1"  # Equal
    assert track1 <= "song2"  # Less than


def test_gt_method():
    track1 = Track("song1", ".mp3", Path("/music"))
    track2 = Track("song2", ".mp3", Path("/music"))

    # Test greater than with another Track
    assert track2 > track1

    # Test greater than with a string
    assert track2 > "song1"


def test_ge_method():
    track1 = Track("song1", ".mp3", Path("/music"))
    track2 = Track("song1", ".mp3", Path("/different/path"))

    # Test greater than or equal with another Track
    assert track2 >= track1  # Greater than
    assert track1 >= track2  # Equal

    # Test greater than or equal with a string
    assert track2 >= "song1"  # Equal
    assert track2 >= "song1"  # Greater than
