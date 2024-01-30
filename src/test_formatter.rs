#[cfg(test)]
mod tests {
    use super::*;
    use crate::fileformat::FileFormat;
    use crate::formatter::TrackFormatter;
    use crate::track::Track;

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
    ];

    static FORMATTING_TEST_DATA: &[FormattingTestData] = &[
        FormattingTestData {
            artist: "ASAP Ferg x A-Ha",
            correct_artist: "ASAP Ferg x A-Ha",
            title: "Plain Jane (Nick Bike Edit + Acap In & Out)[Clean]",
            correct_title: "Plain Jane (Nick Bike Edit + Acapella In & Out) (Clean)",
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
            correct_title: "FNF Let's Go (Nick Bike 'Humble' Edit) (Acapella In Out) (Clean)",
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

    fn run_formatter_tests(test_data: &[FormattingTestData]) {
        let formatter = TrackFormatter::new();
        for data in test_data {
            let (formatted_artist, formatted_title) = formatter.format_tags(data.artist, data.title);
            assert_eq!(formatted_artist, data.correct_artist);
            assert_eq!(formatted_title, data.correct_title);
        }
    }

    #[test]
    fn test_balance_parentheses() {
        run_formatter_tests(&BALANCE_PARENTHESES_TEST_DATA)
    }
    #[test]
    fn test_feat_formatting() {
        run_formatter_tests(&FEAT_TEST_DATA)
    }
    #[test]
    fn test_formatting() {
        run_formatter_tests(&FORMATTING_TEST_DATA)
    }
    #[test]
    fn test_nested_parentheses() {
        run_formatter_tests(&NESTED_PARENTHESES_TEST_DATA)
    }
    #[test]
    fn test_parentheses() {
        run_formatter_tests(&PARENTHESES_TEST_DATA)
    }
    #[test]
    fn test_remix_formatting() {
        run_formatter_tests(&REMIX_FORMATTING_TEST_DATA)
    }
    #[test]
    fn test_remove_bpm_and_key() {
        run_formatter_tests(&REMOVE_BPM_AND_KEY_TEST_DATA)
    }
    #[test]
    fn test_whitespace_formatting() {
        run_formatter_tests(&WHITESPACE_TEST_DATA)
    }
}
