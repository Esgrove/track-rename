import pytest

from rename.formatter import Formatter
from tests.test_data import (
    BALANCE_PARENTHESES_IDS,
    BALANCE_PARENTHESES_TEST_DATA,
    FEAT_IDS,
    FEAT_TEST_DATA,
    FORMATTING_IDS,
    FORMATTING_TEST_DATA,
    NESTED_PARENTHESES_IDS,
    NESTED_PARENTHESES_TEST_DATA,
    PARENTHESES_IDS,
    PARENTHESES_TEST_DATA,
    WHITESPACE_IDS,
    WHITESPACE_TEST_DATA,
)


@pytest.fixture(scope="module")
def formatter():
    formatter = Formatter()
    yield formatter


@pytest.mark.parametrize("artist, correct_artist, title, correct_title", FORMATTING_TEST_DATA, ids=FORMATTING_IDS)
def test_formatting(formatter, artist, correct_artist, title, correct_title):
    _check_format_track(formatter, artist, title, correct_artist, correct_title)


@pytest.mark.parametrize("artist, correct_artist, title, correct_title", WHITESPACE_TEST_DATA, ids=WHITESPACE_IDS)
def test_whitespace(formatter, artist, correct_artist, title, correct_title):
    _check_format_track(formatter, artist, title, correct_artist, correct_title)


@pytest.mark.parametrize("artist, correct_artist, title, correct_title", PARENTHESES_TEST_DATA, ids=PARENTHESES_IDS)
def test_add_parenthesis(formatter, artist, correct_artist, title, correct_title):
    _check_format_track(formatter, artist, title, correct_artist, correct_title)


@pytest.mark.parametrize(
    "artist, correct_artist, title, correct_title", BALANCE_PARENTHESES_TEST_DATA, ids=BALANCE_PARENTHESES_IDS
)
def test_balance_parenthesis(formatter, artist, correct_artist, title, correct_title):
    _check_format_track(formatter, artist, title, correct_artist, correct_title)


@pytest.mark.parametrize(
    "artist, correct_artist, title, correct_title", NESTED_PARENTHESES_TEST_DATA, ids=NESTED_PARENTHESES_IDS
)
def test_nested_parenthesis(formatter, artist, correct_artist, title, correct_title):
    _check_format_track(formatter, artist, title, correct_artist, correct_title)


@pytest.mark.parametrize("artist, correct_artist, title, correct_title", FEAT_TEST_DATA, ids=FEAT_IDS)
def test_feat_formatting(formatter, artist, correct_artist, title, correct_title):
    _check_format_track(formatter, artist, title, correct_artist, correct_title)


def _check_format_track(formatter, artist, title, correct_artist, correct_title):
    formatted_artist, formatted_title = formatter.format_track(artist, title)
    assert formatted_artist == correct_artist
    assert formatted_title == correct_title
