# Data is in format:
# 1. artist
# 2. formatted artist
# 3. title
# 4. formatted title

BALANCE_PARENTHESES_TEST_DATA = [
    (
        "Janet Jackson",
        "Janet Jackson",
        "If (Kaytranada Edition (Live Set Version)",
        "If (Kaytranada Edition) (Live Set Version)",
    ),
    (
        "Another Jackson",
        "Another Jackson",
        "If (Kaytranada Edition) (Live Set Version",
        "If (Kaytranada Edition) (Live Set Version)",
    ),
]

NESTED_PARENTHESES_TEST_DATA = [
    (
        "Janet Jackson",
        "Janet Jackson",
        "This is a test (with some (nested) parentheses (and (some) more))",
        "This is a test (with some) (nested parentheses) (and) (some more)",
    ),
    (
        "Krewella",
        "Krewella",
        "Live For The Night (Simo 128 (70) (Trayze Rmx) Transition) (Quick)",
        "Live For The Night (Simo 128) (70) (Trayze Rmx Transition) (Quick)",
    ),
]

FEAT_TEST_DATA = [
    (
        "seige",
        "Seige feat. Busta Rhymes, Little Brother, Kurupt, Crooked I & Willie B",
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
        "Rihanna feat. Drake",
        "Rihanna feat. Drake",
        "Whats My Name (Trayze Intro) feat. Drake",
        "Whats My Name (Trayze Intro)",
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
    (
        "DJ Chus & David Penn",
        "DJ Chus & David Penn feat. Concha Buika",
        "Will I (Discover Love - feat. Concha Buika - Mediterranean Club Mix)",
        "Will I (Discover Love) (Mediterranean Club Mix)",
    ),
    (
        "Furry Phreaks",
        "Furry Phreaks feat. Terra Deva",
        "Want Me (Like Water) (feat. Terra Deva - Charles Webster Club Mix 1 - 2013 Re-Edit)",
        "Want Me (Like Water) (Charles Webster Club Mix 1) (2013 Re-Edit)",
    ),
    (
        "Spiller & Sophie Ellis-Bextor",
        "Spiller feat. Sophie Ellis-Bextor",
        "Groovejet (If This Ain't Love) feat. Sophie Ellis-Bextor (Riva Starr Skylight Hard Dub)",
        "Groovejet (If This Ain't Love) (Riva Starr Skylight Hard Dub)",
    ),
    (
        "Daft Punk, Pharrell Williams & Nile Rodgers",
        "Daft Punk feat. Pharrell Williams & Nile Rodgers",
        "Get Lucky (Drumless Edition) (feat. Pharrell Williams and Nile Rodgers)",
        "Get Lucky (Drumless Edition)",
    ),
]

FORMATTING_TEST_DATA = [
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
        "Plain Jane (Nick Bike Edit + Acapella In & Out) (Clean)",
    ),
    (
        "Aazar ft. French Montana",
        "Aazar feat. French Montana",
        "The Carnival (Inst)",
        "The Carnival (Instrumental)",
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
        "FNF Let's Go (Nick Bike 'Humble' Edit) (Acapella In Out) (Clean)",
    ),
    (
        "Big Sean",
        "Big Sean",
        "Dance (A$$) - Tall Boys Remix (DJcity Intro - Dirty)",
        "Dance (A$$) (Tall Boys Remix) (Dirty Intro)",
    ),
    (
        "Toosii",
        "Toosii",
        "Favorite Song (Trayze My Boo Edit) 130 11a",
        "Favorite Song (Trayze My Boo Edit)",
    ),
    (
        "Tori Kelly",
        "Tori Kelly",
        "Cut (Trayze Acap Out) 136",
        "Cut (Trayze Acapella Out)",
    ),
    (
        "Stevie Wonder",
        "Stevie Wonder",
        "Signed, Sealed, Delivered - Trayze Nola Bounce Flip - 102 4a",
        "Signed, Sealed, Delivered (Trayze Nola Bounce Flip)",
    ),
    (
        "Rihanna",
        "Rihanna",
        "Right Now (Facetyme Remix) (132 Ebm)",
        "Right Now (Facetyme Remix)",
    ),
    (
        "Rihanna",
        "Rihanna",
        "Lift Me Up (Trayze Drop Leaf Edit) (89 11b)",
        "Lift Me Up (Trayze Drop Leaf Edit)",
    ),
]

PARENTHESES_TEST_DATA = [
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
        "Cover Artist",
        "Cover Artist",
        "You Make Me Feel (Mighty Real) Clean",
        "You Make Me Feel (Mighty Real) (Clean)",
    ),
    (
        "abc",
        "Abc",
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

WHITESPACE_TEST_DATA = [
    (
        "That Chick Angel, Casa Di & Steve Terrell\n",
        "That Chick Angel, Casa Di & Steve Terrell",
        "One Margarita\t(Margarita Song) (Clean)",
        "One Margarita (Margarita Song) (Clean)",
    ),
    (
        " That Chick Angel,  Steve Terrell   ",
        "That Chick Angel, Steve Terrell",
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


def _get_test_ids(data: list[tuple[str, str, str, str]]) -> tuple[str]:
    """Use the formatted artist name as the test id."""
    return tuple(name[1][:32] for name in data)


BALANCE_PARENTHESES_IDS = _get_test_ids(BALANCE_PARENTHESES_TEST_DATA)
FEAT_IDS = _get_test_ids(FEAT_TEST_DATA)
FORMATTING_IDS = _get_test_ids(FORMATTING_TEST_DATA)
NESTED_PARENTHESES_IDS = _get_test_ids(NESTED_PARENTHESES_TEST_DATA)
PARENTHESES_IDS = _get_test_ids(PARENTHESES_TEST_DATA)
WHITESPACE_IDS = _get_test_ids(WHITESPACE_TEST_DATA)
