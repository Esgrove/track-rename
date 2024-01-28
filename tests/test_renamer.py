from pathlib import Path
from unittest import mock

from rename.renamer import Renamer, Track


# Mock the taglib.File class for testing tag-related functionality
@mock.patch("taglib.File")
def test_renamer_process_tags(mock_taglib_file):
    # Create a mock taglib file instance with pre-defined tag data
    mock_file_instance = mock.Mock()
    mock_file_instance.tags = {"ARTIST": ["Test Artist"], "TITLE": ["Test Title"]}
    mock_taglib_file.return_value = mock_file_instance

    # Set up the Renamer instance
    renamer = Renamer(
        Path("/test/path"), rename_files=True, sort_files=False, print_only=False, tags_only=False, force=True
    )

    # Create a mock Track instance
    test_track = Track("test_track", ".mp3", Path("/test/path"))

    # Call process_tags method
    processed_track = renamer.process_tags(test_track)

    # Assertions
    assert processed_track is not None
    assert processed_track.artist == "Test Artist"
    assert processed_track.title == "Test Title"

    # Verify that taglib.File was called with the correct path
    mock_taglib_file.assert_called_with(test_track.full_path)
