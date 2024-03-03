#[cfg(test)]
mod tests {
    use crate::formatter;

    #[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
    struct FormattingTestData {
        artist: &'static str,
        correct_artist: &'static str,
        title: &'static str,
        correct_title: &'static str,
    }

    static BALANCE_PARENTHESES_TEST_DATA: &[FormattingTestData] = &[
        FormattingTestData {
            artist: "Janet Jackson",
            correct_artist: "Janet Jackson",
            title: "If (Kaytranada Edition (Live Set Version)",
            correct_title: "If (Kaytranada Edition) (Live Set Version)",
        },
        FormattingTestData {
            artist: "Another Jackson",
            correct_artist: "Another Jackson",
            title: "If (Kaytranada Edition) (Live Set Version",
            correct_title: "If (Kaytranada Edition) (Live Set Version)",
        },
        FormattingTestData {
            artist: "Jackson 3",
            correct_artist: "Jackson 3",
            title: "If (Live Set Version",
            correct_title: "If (Live Set Version)",
        },
        FormattingTestData {
            artist: "Jackson 5",
            correct_artist: "Jackson 5",
            title: "If (Jes) Live Set Version)",
            correct_title: "If (Jes) (Live Set Version)",
        },
    ];

    static FEAT_TEST_DATA: &[FormattingTestData] = &[
        FormattingTestData {
            artist: "Seige",
            correct_artist: "Seige feat. Busta Rhymes, Little Brother, Kurupt, Crooked I & Willie B",
            title: "Holla Remix (featuring Busta Rhymes, Little Brother, Kurupt, Crooked I, and Willie B)",
            correct_title: "Holla Remix",
        },
        FormattingTestData {
            artist: "Fanu & Ane Brun",
            correct_artist: "Fanu feat. Ane Brun",
            title: "Taivaita ja Tarinoita (feat. Ane Brun)",
            correct_title: "Taivaita ja Tarinoita",
        },
        FormattingTestData {
            artist: "Lakim",
            correct_artist: "Lakim feat. High Klassified",
            title: "The Abyss (feat. High Klassified)",
            correct_title: "The Abyss",
        },
        FormattingTestData {
            artist: "Rihanna feat. Drake",
            correct_artist: "Rihanna feat. Drake",
            title: "Whats My Name (Trayze Intro) feat. Drake",
            correct_title: "Whats My Name (Trayze Intro)",
        },
        FormattingTestData {
            artist: "Audiojack",
            correct_artist: "Audiojack feat. Kevin Knapp",
            title: "Stay Glued (Feat Kevin Knapp - Zds Remix)",
            correct_title: "Stay Glued (Zds Remix)",
        },
        FormattingTestData {
            artist: "Fatima Njai, Jerome Sydenham",
            correct_artist: "Jerome Sydenham feat. Fatima Njai",
            title: "Waiting For You (Club Remix feat. Fatima Njai)",
            correct_title: "Waiting For You (Club Remix)",
        },
        FormattingTestData {
            artist: "Mike Dunn & Riva Starr",
            correct_artist: "Riva Starr feat. Mike Dunn",
            title: "Feel The Heat feat. Mike Dunn (Extended Mix)",
            correct_title: "Feel The Heat (Extended Mix)",
        },
        FormattingTestData {
            artist: "DJ Chus & David Penn",
            correct_artist: "DJ Chus & David Penn feat. Concha Buika",
            title: "Will I (Discover Love - feat. Concha Buika - Mediterranean Club Mix)",
            correct_title: "Will I (Discover Love) (Mediterranean Club Mix)",
        },
        FormattingTestData {
            artist: "Furry Phreaks",
            correct_artist: "Furry Phreaks feat. Terra Deva",
            title: "Want Me (Like Water) (feat. Terra Deva - Charles Webster Club Mix 1 - 2013 Re-Edit)",
            correct_title: "Want Me (Like Water) (Charles Webster Club Mix 1) (2013 Re-Edit)",
        },
        FormattingTestData {
            artist: "Spiller & Sophie Ellis-Bextor",
            correct_artist: "Spiller feat. Sophie Ellis-Bextor",
            title: "Groovejet (If This Ain't Love) feat. Sophie Ellis-Bextor (Riva Starr Skylight Hard Dub)",
            correct_title: "Groovejet (If This Ain't Love) (Riva Starr Skylight Hard Dub)",
        },
        FormattingTestData {
            artist: "Daft Punk, Pharrell Williams & Nile Rodgers",
            correct_artist: "Daft Punk feat. Pharrell Williams & Nile Rodgers",
            title: "Get Lucky (Drumless Edition) (feat. Pharrell Williams and Nile Rodgers)",
            correct_title: "Get Lucky (Drumless Edition)",
        },
        FormattingTestData {
            artist: "DJ S.K.T, Iris Gold",
            correct_artist: "DJ S.K.T feat. Iris Gold",
            title: "4am In London (Feat. Iris Gold) (Harry Romero Extended Mix)",
            correct_title: "4am In London (Harry Romero Extended Mix)",
        },
        FormattingTestData {
            artist: "Major Lazer (feat. Laidback Luke & Ms. Dynamite)",
            correct_artist: "Major Lazer feat. Laidback Luke & Ms. Dynamite",
            title: "Sweat (Trayze Qh)",
            correct_title: "Sweat (Trayze Qh)",
        },
        FormattingTestData {
            artist: "Copyright",
            correct_artist: "Copyright feat. Mr. V & Miss Patty",
            title: "In Da Club (Shake Shit Up) (Feat. Mr. V & Miss Patty) (Copyright Main Mix)",
            correct_title: "In Da Club (Shake Shit Up) (Copyright Main Mix)",
        },
        FormattingTestData {
            artist: "Copyright, Miss Patty, Mr. V",
            correct_artist: "Copyright feat. Mr. V & Miss Patty",
            title: "In Da Club (Shake Sh-T Up) (Feat. Mr. V & Miss Patty) (Copyright Main Mix)",
            correct_title: "In Da Club (Shake Sh-T Up) (Copyright Main Mix)",
        },
        FormattingTestData {
            artist: "The Weeknd",
            correct_artist: "The Weeknd feat. Ariana Grande",
            title: "Save Your Tears feat Ariana Grande (Flipout Purple Disco Machine Edit)",
            correct_title: "Save Your Tears (Flipout Purple Disco Machine Edit)",
        },
        FormattingTestData {
            artist: "The Weeknd",
            correct_artist: "The Weeknd feat. Ariana Grande",
            title: "Save Your Tears feat Ariana Grande - Flipout Purple Disco Machine Edit",
            correct_title: "Save Your Tears (Flipout Purple Disco Machine Edit)",
        },
        FormattingTestData {
            artist: "Madonna Feat. Kanye West",
            correct_artist: "Madonna feat. Kanye West",
            title: "Beat Goes On (feat. Kanye West) (Featuring Kanye West Album Version)",
            correct_title: "Beat Goes On (Album Version)",
        },
    ];

    static FORMATTING_TEST_DATA: &[FormattingTestData] = &[
        FormattingTestData {
            artist: "ASAP Ferg x A-Ha",
            correct_artist: "ASAP Ferg x A-Ha",
            title: "Plain Jane (Nick Bike Edit + Acap In & Out)[Clean]",
            correct_title: "Plain Jane (Nick Bike Edit + Acapella In-Out) (Clean)",
        },
        FormattingTestData {
            artist: "Aazar ft. French Montana",
            correct_artist: "Aazar feat. French Montana",
            title: "The Carnival (Inst)",
            correct_title: "The Carnival (Instrumental)",
        },
        FormattingTestData {
            artist: "Aitch & AJ Tracey ft. Tay Keith",
            correct_artist: "Aitch & AJ Tracey feat. Tay Keith",
            title: "Rain (DJcity Intro - Clean)",
            correct_title: "Rain (Clean Intro)",
        },
        FormattingTestData {
            artist: "GoRilla x Kendrick Lamar",
            correct_artist: "GoRilla x Kendrick Lamar",
            title: "FNF Let's Go (Nick Bike 'Humble' Edit)(Acap In Out)(Clean)",
            correct_title: "FNF Let's Go (Nick Bike 'Humble' Edit) (Acapella In-Out) (Clean)",
        },
        FormattingTestData {
            artist: "Big Sean",
            correct_artist: "Big Sean",
            title: "Dance (A$$) - Tall Boys Remix (DJcity Intro - Dirty)",
            correct_title: "Dance (A$$) (Tall Boys Remix) (Dirty Intro)",
        },
        FormattingTestData {
            artist: "Big Sean W/Taku",
            correct_artist: "Big Sean feat. Taku",
            title: "Dance (A$$)",
            correct_title: "Dance (A$$)",
        },
        FormattingTestData {
            artist: "Beyonce",
            correct_artist: "Beyonce",
            title: "Beyonce - Texas Hold Em (Flipout X KON _I Want To Thank You_ Edit)(Instrumental)",
            correct_title: "Texas Hold Em (Flipout X KON 'I Want To Thank You' Edit) (Instrumental)",
        },
        FormattingTestData {
            artist: "Evelyn 'Champagne' King",
            correct_artist: "Evelyn 'Champagne' King",
            title: "Im In Love (TRAYZE QUANT REMASTER)",
            correct_title: "I'm In Love (TRAYZE QUANT REMASTER)",
        },
        FormattingTestData {
            artist: "SWV",
            correct_artist: "SWV",
            title: "Youre The One (TRAYZE REMASTER EDIT EXT)",
            correct_title: "You're The One (TRAYZE REMASTER EDIT EXT)",
        },
        FormattingTestData {
            artist: "Giacca & Flores",
            correct_artist: "Giacca & Flores",
            title: "New Monday (Original Mix/Cyberkid Re-Edit)",
            correct_title: "New Monday (Cyberkid Re-Edit)",
        },
        FormattingTestData {
            artist: "Curtis Mayfield",
            correct_artist: "Curtis Mayfield",
            title: "Do Do Wap Is Strong In Here (Ashley Beedle Re-Edit/Beat Junkie Sound Edit)",
            correct_title: "Do Do Wap Is Strong In Here (Ashley Beedle Re-Edit) (Beat Junkie Sound Edit)",
        },
        FormattingTestData {
            artist: "L.B.C. Crew (Feat. - Tray D & South Sentrel)",
            correct_artist: "L.B.C. Crew feat. Tray D & South Sentrel",
            title: "Beware Of My Crew (Dj Pooh Remix Instrumental)",
            correct_title: "Beware Of My Crew (DJ Pooh Remix Instrumental)",
        },
        FormattingTestData {
            artist: "S'hustryi Beats",
            correct_artist: "S'hustryi Beats feat. Theodor",
            title: "Force (feat.Theodor) (Remaster)",
            correct_title: "Force (Remaster)",
        },
        FormattingTestData {
            artist: "Son Of Kick & Paigey Cakey feat.Lady Leshurr",
            correct_artist: "Son Of Kick & Paigey Cakey feat. Lady Leshurr",
            title: "Hours (Marshall F Remix)",
            correct_title: "Hours (Marshall F Remix)",
        },
        FormattingTestData {
            artist: "Nina Sky",
            correct_artist: "Nina Sky",
            title: "Move Ya Body (Trayze Acap-In)",
            correct_title: "Move Ya Body (Trayze Acapella Intro)",
        },
        FormattingTestData {
            artist: "Beyoncé",
            correct_artist: "Beyoncé",
            title: "Break My Soul (Trayze Acap-In Out)",
            correct_title: "Break My Soul (Trayze Acapella In-Out)",
        },
        FormattingTestData {
            artist: "Talib Kweli feat. Anny Dobson & William Taylor & Nina Simone)",
            correct_artist: "Talib Kweli feat. Anny Dobson & William Taylor & Nina Simone",
            title: "Get By (Trayze Resist Acapella In Edit)",
            correct_title: "Get By (Trayze Resist Acapella Intro Edit)",
        },
        FormattingTestData {
            artist: "Drake & 21 Savage",
            correct_artist: "Drake & 21 Savage",
            title: "Pussy & Millions (Nick Bike Acapella In:Out)",
            correct_title: "Pussy & Millions (Nick Bike Acapella In-Out)",
        },
        FormattingTestData {
            artist: "Drake & 21 Savage",
            correct_artist: "Drake & 21 Savage",
            title: "Pussy & Millions (Nick Bike Acap In:Out)",
            correct_title: "Pussy & Millions (Nick Bike Acapella In-Out)",
        },
        FormattingTestData {
            artist: "Tony Touch, Oscar G",
            correct_artist: "Tony Touch, Oscar G",
            title: "Sacude (Oscar G 305 Dub)",
            correct_title: "Sacude (Oscar G 305 Dub)",
        },
        FormattingTestData {
            artist: "The Prodigy",
            correct_artist: "The Prodigy",
            title: "Poison (95 EQ)",
            correct_title: "Poison (95 EQ)",
        },
    ];

    static NESTED_PARENTHESES_TEST_DATA: &[FormattingTestData] = &[
        FormattingTestData {
            artist: "Janet Jackson",
            correct_artist: "Janet Jackson",
            title: "This is a test (with some (nested) parentheses (and (some) more))",
            correct_title: "This is a test (with some) (nested parentheses) (and) (some more)",
        },
        FormattingTestData {
            artist: "Krewella",
            correct_artist: "Krewella",
            title: "Live For The Night (Simo 128 (70) (Trayze Rmx) Transition) (Quick)",
            correct_title: "Live For The Night (Simo 128) (70) (Trayze Rmx Transition) (Quick)",
        },
    ];

    static PARENTHESES_TEST_DATA: &[FormattingTestData] = &[
        FormattingTestData {
            artist: "Redbone",
            correct_artist: "Redbone",
            title: "Come And Get Your Love (Nick Bike Extended Mix) (Instrumental) 2.2",
            correct_title: "Come And Get Your Love (Nick Bike Extended Mix) (Instrumental) (2.2)",
        },
        FormattingTestData {
            artist: "Patrick Adams",
            correct_artist: "Patrick Adams",
            title: "I'm A Big Freak (R U 1 2) Alkalino re-edit",
            correct_title: "I'm A Big Freak (R U 1 2) (Alkalino re-edit)",
        },
        FormattingTestData {
            artist: "Sylvester",
            correct_artist: "Sylvester",
            title: "You Make Me Feel (Mighty Real) (Clean)",
            correct_title: "You Make Me Feel (Mighty Real) (Clean)",
        },
        FormattingTestData {
            artist: "Cover Artist",
            correct_artist: "Cover Artist",
            title: "You Make Me Feel (Mighty Real) Clean",
            correct_title: "You Make Me Feel (Mighty Real) (Clean)",
        },
        FormattingTestData {
            artist: "abcdefg",
            correct_artist: "abcdefg",
            title: "(You Make Me Feel) Mighty Real",
            correct_title: "(You Make Me Feel) Mighty Real",
        },
        FormattingTestData {
            artist: "The Bucketheads",
            correct_artist: "The Bucketheads",
            title: "The Bomb (These Sounds Fall Into My Mind) - KARYO, LPACA & James August Remix",
            correct_title: "The Bomb (These Sounds Fall Into My Mind) (KARYO, LPACA & James August Remix)",
        },
    ];

    static REMIX_FORMATTING_TEST_DATA: &[FormattingTestData] = &[
        FormattingTestData {
            artist: "ACA",
            correct_artist: "ACA",
            title: "Azn Danza - Myles Club Edit",
            correct_title: "Azn Danza (Myles Club Edit)",
        },
        FormattingTestData {
            artist: "Lizzo",
            correct_artist: "Lizzo",
            title: "About Damn Time - Purple Disco Machine (Dirty Intro)",
            correct_title: "About Damn Time (Purple Disco Machine) (Dirty Intro)",
        },
    ];

    static REMOVE_BPM_AND_KEY_TEST_DATA: &[FormattingTestData] = &[
        FormattingTestData {
            artist: "Toosii",
            correct_artist: "Toosii",
            title: "Favorite Song (Trayze My Boo Edit) 130 11a",
            correct_title: "Favorite Song (Trayze My Boo Edit)",
        },
        FormattingTestData {
            artist: "Tori Kelly",
            correct_artist: "Tori Kelly",
            title: "Cut (Trayze Acap Out) 136",
            correct_title: "Cut (Trayze Acapella Out)",
        },
        FormattingTestData {
            artist: "Stevie Wonder",
            correct_artist: "Stevie Wonder",
            title: "Signed, Sealed, Delivered - Trayze Nola Bounce Flip - 102 4a",
            correct_title: "Signed, Sealed, Delivered (Trayze Nola Bounce Flip)",
        },
        FormattingTestData {
            artist: "Rihanna",
            correct_artist: "Rihanna",
            title: "Right Now (Facetyme Remix) (132 Ebm)",
            correct_title: "Right Now (Facetyme Remix)",
        },
        FormattingTestData {
            artist: "Rihanna",
            correct_artist: "Rihanna",
            title: "Lift Me Up (Trayze Drop Leaf Edit) (89 11b)",
            correct_title: "Lift Me Up (Trayze Drop Leaf Edit)",
        },
        FormattingTestData {
            artist: "Rihanna",
            correct_artist: "Rihanna",
            title: "Lift Me Up (Trayze Drop Leaf Edit) (89 11b)",
            correct_title: "Lift Me Up (Trayze Drop Leaf Edit)",
        },
        FormattingTestData {
            artist: "Don Omar feat. Lucenzo",
            correct_artist: "Don Omar feat. Lucenzo",
            title: "Danza Kuduro (Trayze Acapella In Out Edit) (130 8b)",
            correct_title: "Danza Kuduro (Trayze Acapella In-Out Edit)",
        },
        FormattingTestData {
            artist: "Libianca feat. Ayra Starr & Omah Lay",
            correct_artist: "Libianca feat. Ayra Starr & Omah Lay",
            title: "People (Trayze Remix) (113 4b)",
            correct_title: "People (Trayze Remix)",
        },
    ];

    static WHITESPACE_TEST_DATA: &[FormattingTestData] = &[
        FormattingTestData {
            artist: "That Chick Angel, Casa Di & Steve Terrell\n",
            correct_artist: "That Chick Angel, Casa Di & Steve Terrell",
            title: "One Margarita\t(Margarita Song) (Clean)",
            correct_title: "One Margarita (Margarita Song) (Clean)",
        },
        FormattingTestData {
            artist: " That Chick Angel,  Steve Terrell   ",
            correct_artist: "That Chick Angel, Steve Terrell",
            title: "One      \t\tMargarita(Margarita Song )( Clean)",
            correct_title: "One Margarita (Margarita Song) (Clean)",
        },
        FormattingTestData {
            artist: "A.D.  ",
            correct_artist: "A.D.",
            title: " Through the Shuffle ",
            correct_title: "Through the Shuffle",
        },
    ];

    static FILE_FORMATTING_TEST_DATA: &[FormattingTestData] = &[
        FormattingTestData {
            artist: "A*rtist",
            correct_artist: "A-rtist",
            title: "Na<me",
            correct_title: "Na-me",
        },
        FormattingTestData {
            artist: "Artist with  Spaces",
            correct_artist: "Artist with Spaces",
            title: "Title  with   Spaces",
            correct_title: "Title with Spaces",
        },
        FormattingTestData {
            artist: "Artist \"Name\"",
            correct_artist: "Artist ''Name''",
            title: "Title \"Version\"",
            correct_title: "Title ''Version''",
        },
        FormattingTestData {
            artist: "A/rtist|Name",
            correct_artist: "A-rtist-Name",
            title: "T:itle*Name?",
            correct_title: "T-itle-Name-",
        },
        FormattingTestData {
            artist: "Artist-Name",
            correct_artist: "Artist-Name",
            title: "Title (Original Mix)",
            correct_title: "Title (Original Mix)",
        },
        FormattingTestData {
            artist: "Artist/Name",
            correct_artist: "Artist-Name",
            title: "Title/Name (VIP Remix)",
            correct_title: "Title-Name (VIP Remix)",
        },
        FormattingTestData {
            artist: "Mary J Blige",
            correct_artist: "Mary J Blige",
            title: "Love No Limit (Flipout Acoustic Mix W/Drums)",
            correct_title: "Love No Limit (Flipout Acoustic Mix W-Drums)",
        },
    ];

    fn run_tag_formatting_tests(test_data: &[FormattingTestData]) {
        for data in test_data {
            let (formatted_artist, formatted_title) = formatter::format_tags(data.artist, data.title);
            assert_eq!(formatted_artist, data.correct_artist);
            assert_eq!(formatted_title, data.correct_title);
        }
    }

    #[test]
    fn test_balance_parentheses() {
        run_tag_formatting_tests(BALANCE_PARENTHESES_TEST_DATA)
    }

    #[test]
    fn test_feat_formatting() {
        run_tag_formatting_tests(FEAT_TEST_DATA)
    }

    #[test]
    fn test_formatting() {
        run_tag_formatting_tests(FORMATTING_TEST_DATA)
    }

    #[test]
    fn test_nested_parentheses() {
        run_tag_formatting_tests(NESTED_PARENTHESES_TEST_DATA)
    }

    #[test]
    fn test_parentheses() {
        run_tag_formatting_tests(PARENTHESES_TEST_DATA)
    }

    #[test]
    fn test_remix_formatting() {
        run_tag_formatting_tests(REMIX_FORMATTING_TEST_DATA)
    }

    #[test]
    fn test_remove_bpm_and_key() {
        run_tag_formatting_tests(REMOVE_BPM_AND_KEY_TEST_DATA)
    }

    #[test]
    fn test_whitespace_formatting() {
        run_tag_formatting_tests(WHITESPACE_TEST_DATA)
    }

    #[test]
    fn test_filename_formatting() {
        for data in FILE_FORMATTING_TEST_DATA {
            let (formatted_artist, formatted_title) = formatter::format_filename(data.artist, data.title);
            assert_eq!(formatted_artist, data.correct_artist);
            assert_eq!(formatted_title, data.correct_title);
        }
    }
}
