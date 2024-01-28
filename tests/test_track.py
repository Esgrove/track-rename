from pathlib import Path
from unittest import mock

import pytest

from rename.track import Track


def test_constructor():
    track = Track("song", ".mp3", Path("/user/test/music"))
    assert track.name == "song"
    assert track.extension == ".mp3"
    assert track.root == Path("/user/test/music")

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
def test_extensions(extension):
    track = Track("song", extension, Path("/user/test/music"))
    assert track.extension == extension
    assert track.filename == "song" + extension
    assert track.full_path == Path("/user/test/music/song" + extension)


@pytest.mark.parametrize("extension", [".mp3", ".FLAC", ".aif", ".AIFF", ".m4a"])
def test_remove_extra_extensions(extension):
    track = Track(f"Test - Song with extra extension{extension}", extension, Path("/user/test/music"))
    assert track.extension == extension
    assert track.filename == "Test - Song with extra extension" + extension

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


@mock.patch("builtins.print")
def test_show(mock_print):
    track = Track("song", ".mp3", Path("/music"), number=1)
    track.show(10)
    mock_print.assert_called_with("1/10:")


def test_is_mp3():
    track = Track("song", ".mp3", Path("/music"))
    assert track.is_mp3() is True

    track = Track("song", ".flac", Path("/music"))
    assert track.is_mp3() is False


def test_is_aif():
    track = Track("song", ".aif", Path("/music"))
    assert track.is_aif() is True

    track = Track("song", ".aiff", Path("/music"))
    assert track.is_aif() is True

    track = Track("song", ".mp3", Path("/music"))
    assert track.is_aif() is False


def test_new_with_number():
    track = Track("song", ".mp3", Path("/music"))
    new_track = track.new_with_number(5)
    assert new_track.number == 5
    assert new_track.name == track.name
    assert new_track.extension == track.extension
    assert new_track.root == track.root


def test_original_tags_property():
    track = Track("song", ".mp3", Path("/music"))
    track.artist = "Artist"
    track.title = "Title"
    assert track.original_tags == "Artist - Title"


def test_formatted_tags_property():
    track = Track("song", ".mp3", Path("/music"))
    track.formatted_artist = "Formatted Artist"
    track.formatted_title = "Formatted Title"
    assert track.formatted_tags == "Formatted Artist - Formatted Title"


def test_formatted_extension_property():
    track = Track("song", ".AIFF", Path("/music"))
    assert track.formatted_extension == ".aif"

    track = Track("song", ".mp3", Path("/music"))
    assert track.formatted_extension == ".mp3"


def test_full_path_without_extension_property():
    track = Track("song", ".mp3", Path("/user/test/music"))
    assert track.full_path_without_extension == Path("/user/test/music/song")
