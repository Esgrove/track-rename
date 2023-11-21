from pathlib import Path

import pytest

from renamer import Renamer


@pytest.fixture(scope="module")
def renamer():
    renamer = Renamer(Path(""), False, False, True, False)
    yield renamer


def test_formatting(renamer):
    test_cases = [
        (
            "ACA",
            "ACA",
            "Azn Danza - Myles Club Edit",
            "Azn Danza (Myles Club Edit)",
        ),
        (
            "ASAP Ferg x A-Ha",
            "ASAP Ferg x A-Ha",
            "Plain Jane (Nick Bike Edit + Acap In & Out)[Clean]",
            "Plain Jane (Nick Bike Edit + Acap In & Out) (Clean)",
        ),
        (
            "Aazar ft. French Montana",
            "Aazar feat. French Montana",
            "The Carnival (Inst)",
            "The Carnival (Inst)",
        ),
        (
            "Aitch & AJ Tracey ft. Tay Keith",
            "Aitch & AJ Tracey feat. Tay Keith",
            "Rain (DJcity Intro - Clean)",
            "Rain (Clean Intro)",
        ),
        (
            "Lizzo",
            "Lizzo",
            "About Damn Time - Purple Disco Machine (Dirty Intro)",
            "About Damn Time (Purple Disco Machine) (Dirty Intro)",
        ),
        (
            "GloRilla x Kendrick Lamar",
            "GloRilla x Kendrick Lamar",
            "FNF Let's Go (Nick Bike 'Humble' Edit)(Acap In Out)(Clean)",
            "FNF Let's Go (Nick Bike 'Humble' Edit) (Acap In Out) (Clean)",
        ),
    ]

    for artist, correct_artist, title, correct_title in test_cases:
        _check_format_track(renamer, artist, title, correct_artist, correct_title)


def test_whitespace(renamer):
    test_cases = [
        (
            "That Chick Angel, Casa Di & Steve Terrell\n",
            "That Chick Angel, Casa Di & Steve Terrell",
            "One Margarita\t(Margarita Song) (Clean)",
            "One Margarita (Margarita Song) (Clean)",
        ),
        (
            " That Chick Angel, Casa Di &  Steve Terrell   ",
            "That Chick Angel, Casa Di & Steve Terrell",
            "One      \t\tMargarita(Margarita Song )( Clean)",
            "One Margarita (Margarita Song) (Clean)",
        ),
        (
            "A.D.  ",
            "A.D.",
            " Through the Shuffle ",
            "Through the Shuffle",
        ),
    ]

    for artist, correct_artist, title, correct_title in test_cases:
        _check_format_track(renamer, artist, title, correct_artist, correct_title)


def test_add_parenthesis(renamer):
    test_cases = [
        (
            "Redbone",
            "Redbone",
            "Come And Get Your Love (Nick Bike Extended Mix) (Instrumental) 2.2",
            "Come And Get Your Love (Nick Bike Extended Mix) (Instrumental) (2.2)",
        ),
        (
            "Patrick Adams",
            "Patrick Adams",
            "I'm A Big Freak (R U 1 2) Alkalino re-edit",
            "I'm A Big Freak (R U 1 2) (Alkalino re-edit)",
        ),
        (
            "Sylvester",
            "Sylvester",
            "You Make Me Feel (Mighty Real) (Clean)",
            "You Make Me Feel (Mighty Real) (Clean)",
        ),
        (
            "Sylvester",
            "Sylvester",
            "You Make Me Feel (Mighty Real) Clean",
            "You Make Me Feel (Mighty Real) (Clean)",
        ),
        (
            "Sylvester",
            "Sylvester",
            "(You Make Me Feel) Mighty Real",
            "(You Make Me Feel) Mighty Real",
        ),
        (
            "The Bucketheads",
            "The Bucketheads",
            "The Bomb (These Sounds Fall Into My Mind) - KARYO, LPACA & James August Remix",
            "The Bomb (These Sounds Fall Into My Mind) (KARYO, LPACA & James August Remix)",
        ),
    ]

    for artist, correct_artist, title, correct_title in test_cases:
        _check_format_track(renamer, artist, title, correct_artist, correct_title)


def test_balance_parenthesis(renamer):
    test_cases = [
        (
            "Janet Jackson",
            "Janet Jackson",
            "If (Kaytranada Edition (Live Set Version)",
            "If (Kaytranada Edition) (Live Set Version)",
        ),
    ]

    for artist, correct_artist, title, correct_title in test_cases:
        _check_format_track(renamer, artist, title, correct_artist, correct_title)


def test_feat(renamer):
    test_cases = [
        (
            "seige",
            "Seige feat. Busta Rhymes, Little Brother, Kurupt, Crooked I, and Willie B",
            "Holla Remix (featuring Busta Rhymes, Little Brother, Kurupt, Crooked I, and Willie B)",
            "Holla Remix",
        ),
        (
            "Fanu & Ane Brun",
            "Fanu feat. Ane Brun",
            "Taivaita ja Tarinoita (feat. Ane Brun)",
            "Taivaita ja Tarinoita",
        ),
        (
            "Lakim",
            "Lakim feat. High Klassified",
            "The Abyss (feat. High Klassified)",
            "The Abyss",
        ),
        (
            "Audiojack",
            "Audiojack feat. Kevin Knapp",
            "Stay Glued (Feat Kevin Knapp - Zds Remix)",
            "Stay Glued (Zds Remix)",
        ),
        (
            "Fatima Njai, Jerome Sydenham",
            "Jerome Sydenham feat. Fatima Njai",
            "Waiting For You (Club Remix feat. Fatima Njai)",
            "Waiting For You (Club Remix)",
        ),
        (
            "Mike Dunn & Riva Starr",
            "Riva Starr feat. Mike Dunn",
            "Feel The Heat feat. Mike Dunn (Extended Mix)",
            "Feel The Heat (Extended Mix)",
        ),
    ]

    for artist, correct_artist, title, correct_title in test_cases:
        _check_format_track(renamer, artist, title, correct_artist, correct_title)


def _check_format_track(renamer, artist, title, correct_artist, correct_title):
    formatted_artist, formatted_title = renamer.format_track(artist, title)
    assert formatted_artist == correct_artist
    assert formatted_title == correct_title
