WHITESPACE_TEST_DATA = [
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

WHITESPACE_IDS = tuple(case[0] for case in WHITESPACE_TEST_DATA)


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
]

FORMATTING_IDS = tuple(case[0] for case in FORMATTING_TEST_DATA)

FEAT_TEST_DATA = [
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
]

FEAT_IDS = tuple(case[0] for case in FEAT_TEST_DATA)

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
        "a",
        "A",
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

PARENTHESES_IDS = tuple(case[0] for case in PARENTHESES_TEST_DATA)

BALANCE_PARENTHESES_DATA = [
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

BALANCE_PARENTHESES_IDS = tuple(case[0] for case in BALANCE_PARENTHESES_DATA)
