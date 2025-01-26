use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::formatting;

// Map folder names to default genre for that folder.
// If the genre tag is empty, can apply default genre tag.
pub static GENRE_MAPPINGS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("DISCO 1", "Disco"),
        ("DISCO CLASSICS", "Disco"),
        ("DISCO EDITS 1", "Disco"),
        ("DISCO EDITS WAACKING", "Disco"),
        ("DISCO EDITS", "Disco"),
        ("DISCO ELECTRO", "Electro"),
        ("DISCO HI-NRG", "Disco Hi-NRG"),
        ("DISCO ITALO", "Disco Italo"),
        ("DISCO JAM 1", "Disco"),
        ("DISCO JAM", "Disco"),
        ("DISCO", "Disco"),
        ("DRUM&BASS CHILL", "Drum & Bass"),
        ("DRUM&BASS DARK", "Drum & Bass"),
        ("DRUM&BASS EDITS", "Drum & Bass"),
        ("DRUM&BASS POP", "Drum & Bass"),
        ("DRUM&BASS", "Drum & Bass"),
        ("DUBSTEP", "Dubstep"),
        ("ELECTRONICA CHILL", "Electronica"),
        ("ELECTRONICA", "Electronica"),
        ("FUNK 80s 1", "Funk 80s"),
        ("FUNK 80s", "Funk 80s"),
        ("FUNK BOOGIE 1", "Funk Boogie"),
        ("FUNK BOOGIE EDITS 1", "Funk Boogie"),
        ("FUNK BOOGIE EDITS", "Funk Boogie"),
        ("FUNK BOOGIE JAM", "Funk Boogie"),
        ("FUNK BOOGIE POP", "Funk Boogie"),
        ("FUNK BOOGIE", "Funk Boogie"),
        ("FUNK BREAKS 1", "Funk Breaks"),
        ("FUNK BREAKS CHILL", "Funk Breaks"),
        ("FUNK BREAKS EDITS", "Funk Breaks"),
        ("FUNK BREAKS JAM", "Funk Breaks"),
        ("FUNK BREAKS TOP ROCK 1", "Funk Breaks"),
        ("FUNK BREAKS TOP ROCK", "Funk Breaks"),
        ("FUNK BREAKS", "Funk Breaks"),
        ("FUNK CHILL 1", "Funk"),
        ("FUNK CHILL", "Funk"),
        ("FUNK CLASSICS 1", "Funk"),
        ("FUNK CLASSICS", "Funk"),
        ("FUNK EDITS 1", "Funk"),
        ("FUNK EDITS 80s", "Funk"),
        ("FUNK EDITS BATTLE", "Funk"),
        ("FUNK EDITS CHILL 1", "Funk"),
        ("FUNK EDITS CHILL", "Funk"),
        ("FUNK EDITS CLASSICS", "Funk"),
        ("FUNK EDITS JAM 1", "Funk"),
        ("FUNK EDITS JAM", "Funk"),
        ("FUNK EDITS", "Funk"),
        ("FUNK JAM 1", "Funk"),
        ("FUNK JAM CHILL", "Funk"),
        ("FUNK JAM", "Funk"),
        ("FUNK LOVE 1", "Funk"),
        ("FUNK LOVE CHILL", "Funk"),
        ("FUNK LOVE HITAAT", "Funk"),
        ("FUNK LOVE", "Funk"),
        ("FUNK MASHUP", "Funk"),
        ("FUNK PARTY 1", "Funk"),
        ("FUNK PARTY", "Funk"),
        ("FUNK SAMPLE", "Funk"),
        ("FUNK", "Funk"),
        ("FUNKY 1", "Funk"),
        ("FUNKY AFRO & LATIN", "Latin"),
        ("FUNKY BOOGIE 1", "Funk Boogie"),
        ("FUNKY BOOGIE JAM", "Funk Boogie"),
        ("FUNKY BOOGIE", "Funk Boogie"),
        ("FUNKY BREAKS BATTLE", "Funk Breaks"),
        ("FUNKY BREAKS CHILL", "Funk Breaks"),
        ("FUNKY BREAKS GHETTOFUNK", "Ghettofunk"),
        ("FUNKY BREAKS JAM 1", "Funk Breaks"),
        ("FUNKY BREAKS JAM", "Funk Breaks"),
        ("FUNKY BREAKS LATIN", "Funk Breaks"),
        ("FUNKY BREAKS PARTY", "Funk Breaks"),
        ("FUNKY BREAKS RAP JAM", "Funk Breaks"),
        ("FUNKY BREAKS RAP", "Funk Breaks"),
        ("FUNKY BREAKS REGGAE", "Funk Breaks"),
        ("FUNKY BREAKS", "Funk Breaks"),
        ("FUNKY CHILL", "Funk"),
        ("FUNKY JAM 1", "Disco Nu"),
        ("FUNKY JAM CHILL", "Disco Nu"),
        ("FUNKY JAM", "Disco Nu"),
        ("FUNKY NU DISCO", "Disco Nu"),
        ("FUNKY POP 1", "Funk"),
        ("FUNKY POP", "Funk"),
        ("FUNKY", "Funk"),
        ("HIPHOP 1", "Hip-Hop"),
        ("HIPHOP AFRO BEATS INSTRUMENTAL", "Afrobeats"),
        ("HIPHOP AFRO BEATS", "Afrobeats"),
        ("HIPHOP BATTLE 1", "Hip-Hop"),
        ("HIPHOP BATTLE 90s", "Hip-Hop 90s"),
        ("HIPHOP BATTLE BREAKS", "Hip-Hop"),
        ("HIPHOP BATTLE CLUB", "Hip-Hop"),
        ("HIPHOP BATTLE INSTRUMENTAL", "Hip-Hop"),
        ("HIPHOP BATTLE NEW", "Hip-Hop"),
        ("HIPHOP BATTLE POPPING INSTRUMENTAL", "Hip-Hop"),
        ("HIPHOP BATTLE POPPING", "Hip-Hop"),
        ("HIPHOP BATTLE TRAP", "Hip-Hop Trap"),
        ("HIPHOP BATTLE", "Hip-Hop"),
        ("HIPHOP BEATS 1", "Hip-Hop"),
        ("HIPHOP BEATS BATTLE 1", "Hip-Hop"),
        ("HIPHOP BEATS BATTLE", "Hip-Hop"),
        ("HIPHOP BEATS CHILL", "Hip-Hop"),
        ("HIPHOP BEATS ELECTRO", "Hip-Hop"),
        ("HIPHOP BEATS GLITCH", "Glitch Hop"),
        ("HIPHOP BEATS JAM 1", "Hip-Hop"),
        ("HIPHOP BEATS JAM", "Hip-Hop"),
        ("HIPHOP BEATS POPPING JAM", "Hip-Hop"),
        ("HIPHOP BEATS POPPING", "Hip-Hop"),
        ("HIPHOP BEATS TRAP", "Hip-Hop Trap"),
        ("HIPHOP BEATS", "Hip-Hop"),
        ("HIPHOP CHILL 1", "Hip-Hop"),
        ("HIPHOP CHILL", "Hip-Hop"),
        ("HIPHOP CLUB", "Hip-Hop"),
        ("HIPHOP DANCEHALL", "Dancehall"),
        ("HIPHOP ELECTRO", "Electro"),
        ("HIPHOP FUTURE BASS", "Hip-Hop"),
        ("HIPHOP G-FUNK", "Hip-Hop G-Funk"),
        ("HIPHOP INSTRUMENTAL 1", "Hip-Hop"),
        ("HIPHOP INSTRUMENTAL 90s", "Hip-Hop 90s"),
        ("HIPHOP INSTRUMENTAL BREAKS", "Hip-Hop"),
        ("HIPHOP INSTRUMENTAL CHILL", "Hip-Hop"),
        ("HIPHOP INSTRUMENTAL CLUB", "Hip-Hop"),
        ("HIPHOP INSTRUMENTAL G-FUNK", "Hip-Hop G-Funk"),
        ("HIPHOP INSTRUMENTAL JAM 1", "Hip-Hop"),
        ("HIPHOP INSTRUMENTAL JAM", "Hip-Hop"),
        ("HIPHOP INSTRUMENTAL PARTY", "Hip-Hop"),
        ("HIPHOP INSTRUMENTAL POP", "Hip-Hop"),
        ("HIPHOP INSTRUMENTAL RNB", "R&B"),
        ("HIPHOP INSTRUMENTAL SUOMIRAP", "Hip-Hop Suomirap"),
        ("HIPHOP INSTRUMENTAL", "Hip-Hop"),
        ("HIPHOP JAM 1", "Hip-Hop"),
        ("HIPHOP JAM 90s", "Hip-Hop 90s"),
        ("HIPHOP JAM BREAKS", "Hip-Hop"),
        ("HIPHOP JAM CHILL", "Hip-Hop"),
        ("HIPHOP JAM EDITS", "Hip-Hop"),
        ("HIPHOP JAM G-FUNK", "Hip-Hop G-Funk"),
        ("HIPHOP JAM NEW", "Hip-Hop"),
        ("HIPHOP JAM", "Hip-Hop"),
        ("HIPHOP JERSEY CLUB", "Jersey Club"),
        ("HIPHOP NEW JACK SWING 1", "New Jack Swing"),
        ("HIPHOP NEW JACK SWING CHILL", "New Jack Swing"),
        ("HIPHOP NEW JACK SWING", "New Jack Swing"),
        ("HIPHOP OLD SCHOOL", "Hip-Hop"),
        ("HIPHOP PARTY 1", "Hip-Hop"),
        ("HIPHOP PARTY 90s", "Hip-Hop 90s"),
        ("HIPHOP PARTY BREAKS", "Hip-Hop"),
        ("HIPHOP PARTY", "Hip-Hop"),
        ("HIPHOP RNB 1", "R&B"),
        ("HIPHOP RNB 90s", "R&B"),
        ("HIPHOP RNB CHILL 1", "R&B"),
        ("HIPHOP RNB CHILL", "R&B"),
        ("HIPHOP RNB CLUB", "R&B"),
        ("HIPHOP RNB EDITS", "R&B"),
        ("HIPHOP RNB", "R&B"),
        ("HIPHOP SAMPLE", "Hip-Hop"),
        ("HIPHOP SUOMIRAP 1", "Hip-Hop Suomirap"),
        ("HIPHOP SUOMIRAP LOPETUS", "Hip-Hop Suomirap"),
        ("HIPHOP SUOMIRAP", "Hip-Hop Suomirap"),
        ("HIPHOP TRAP 1", "Hip-Hop Trap"),
        ("HIPHOP TRAP POP", "Hip-Hop Trap"),
        ("HIPHOP TRAP", "Hip-Hop Trap"),
        ("HIPHOP", "Hip-Hop"),
        ("HOUSE 1", "House"),
        ("HOUSE ACID", "House Acid"),
        ("HOUSE AFRO 1", "House Afro"),
        ("HOUSE AFRO AMAPIANO", "Amapiano"),
        ("HOUSE AFRO BEATS", "Afrobeats"),
        ("HOUSE AFRO JAM", "House Afro"),
        ("HOUSE AFRO LATIN CLUB", "House Latin"),
        ("HOUSE AFRO LATIN", "House Latin"),
        ("HOUSE AFRO POP", "House Afro"),
        ("HOUSE AFRO TRIBAL CLUB", "House Tribal"),
        ("HOUSE AFRO TRIBAL JAM", "House Tribal"),
        ("HOUSE AFRO TRIBAL", "House Tribal"),
        ("HOUSE AFRO", "House Afro"),
        ("HOUSE BAILE FUNK POP", "Baile Funk"),
        ("HOUSE BAILE FUNK", "Baile Funk"),
        ("HOUSE BALTIMORE", "Baltimore Club"),
        ("HOUSE BASS 1", "House"),
        ("HOUSE BASS CLUB", "House"),
        ("HOUSE BASS FUTURE", "House Future"),
        ("HOUSE BASS POP", "House"),
        ("HOUSE BASS UG 1", "House"),
        ("HOUSE BASS UG", "House"),
        ("HOUSE BASS", "House"),
        ("HOUSE BIG ROOM", "House"),
        ("HOUSE BOOGIE", "House"),
        ("HOUSE BREAKBEAT", "Breakbeat"),
        ("HOUSE BROKEN BEAT", "Broken Beat"),
        ("HOUSE CHILL 1", "House"),
        ("HOUSE CHILL AMBIENT", "House"),
        ("HOUSE CHILL", "House"),
        ("HOUSE DEEP 1", "House Deep"),
        ("HOUSE DEEP BASS 1", "House Deep"),
        ("HOUSE DEEP BASS", "House Deep"),
        ("HOUSE DEEP CHILL 1", "House Deep"),
        ("HOUSE DEEP JAM 1", "House Deep"),
        ("HOUSE DEEP JAM", "House Deep"),
        ("HOUSE DEEP OLD 1", "House Deep"),
        ("HOUSE DEEP OLD JAM 1", "House Deep"),
        ("HOUSE DEEP OLD JAM", "House Deep"),
        ("HOUSE DEEP OLD", "House Deep"),
        ("HOUSE DEEP RAW", "House Deep"),
        ("HOUSE DEEP TECH 1", "House Deep"),
        ("HOUSE DEEP TECH", "House Deep"),
        ("HOUSE DEEP VOGUE", "House Deep"),
        ("HOUSE DEEP", "House Deep"),
        ("HOUSE DISCO 1", "House Disco"),
        ("HOUSE DISCO JAM", "House Disco"),
        ("HOUSE DISCO", "House Disco"),
        ("HOUSE ELECTRO", "House Electro"),
        ("HOUSE ELECTRONICA", "House"),
        ("HOUSE EURODANCE", "Eurodance"),
        ("HOUSE FUNKY JAM", "House Funky"),
        ("HOUSE FUNKY", "House Funky"),
        ("HOUSE JAM CHILL", "House"),
        ("HOUSE JAM", "House"),
        ("HOUSE JAZZY", "House"),
        ("HOUSE MAINSTREAM INSTRUMENTAL", "House"),
        ("HOUSE MAINSTREAM", "House"),
        ("HOUSE MINIMAL", "House"),
        ("HOUSE MOOMBAHTON", "Moombahton"),
        ("HOUSE OLD SCHOOL 1", "House"),
        ("HOUSE OLD SCHOOL CHILL", "House"),
        ("HOUSE OLD SCHOOL DISCO", "House"),
        ("HOUSE OLD SCHOOL HIP", "House"),
        ("HOUSE OLD SCHOOL JAM", "House"),
        ("HOUSE OLD SCHOOL POP", "House"),
        ("HOUSE OLD SCHOOL REMIX", "House"),
        ("HOUSE OLD SCHOOL VOGUE", "House"),
        ("HOUSE OLD SCHOOL", "House"),
        ("HOUSE PIANO", "House"),
        ("HOUSE POP CLUB INSTRUMENTAL", "House"),
        ("HOUSE POP CLUB", "House"),
        ("HOUSE PROGRESSIVE CLUB", "House Progressive"),
        ("HOUSE PROGRESSIVE", "House Progressive"),
        ("HOUSE RAVE NU", "Rave"),
        ("HOUSE RAVE OLD SCHOOL", "Rave"),
        ("HOUSE RAVE", "House"),
        ("HOUSE REMIX", "House"),
        ("HOUSE SOULFUL CHILL", "House Soulful"),
        ("HOUSE SOULFUL VOCAL", "House Soulful"),
        ("HOUSE SOULFUL", "House Soulful"),
        ("HOUSE TECH 1", "House Tech"),
        ("HOUSE TECH CLUB", "House Tech"),
        ("HOUSE TECH DARK 1", "House Tech"),
        ("HOUSE TECH DARK", "House Tech"),
        ("HOUSE TECH JAM", "House Tech"),
        ("HOUSE TECH VOGUE 1", "House Tech"),
        ("HOUSE TECH VOGUE", "House Tech"),
        ("HOUSE TECH", "House Tech"),
        ("HOUSE TECHNO DEEP", "Techno"),
        ("HOUSE TECHNO OLD SCHOOL", "Techno"),
        ("HOUSE TECHNO", "Techno"),
        ("HOUSE TRANCE", "Trance"),
        ("HOUSE UK BEATS", "Electronic"),
        ("HOUSE UK FUNKY", "UK Funky"),
        ("HOUSE UK GARAGE", "UK Garage"),
        ("HOUSE VOGUE BEATS 1", "Vogue Beats"),
        ("HOUSE VOGUE BEATS ARMS HANDS", "Vogue Beats"),
        ("HOUSE VOGUE BEATS CLUB", "Vogue Beats"),
        ("HOUSE VOGUE BEATS DEEP", "Vogue Beats"),
        ("HOUSE VOGUE BEATS HARD", "Vogue Beats"),
        ("HOUSE VOGUE BEATS RUNWAY", "Vogue Beats"),
        ("HOUSE VOGUE BEATS", "Vogue Beats"),
        ("HOUSE VOGUE FEMME 1", "Vogue Beats"),
        ("HOUSE VOGUE FEMME VOCAL", "Vogue Beats"),
        ("HOUSE VOGUE FEMME", "Vogue Beats"),
        ("HOUSE VOGUE JERSEY CLUB", "Jersey Club"),
        ("HOUSE VOGUE NEW WAY", "House"),
        ("HOUSE VOGUE OLD WAY", "House"),
        ("HOUSE VOGUE", "House"),
        ("HOUSE", "House"),
        ("INDIE DANCE", "Indie Dance"),
        ("INDIE ELECTRO", "Electro"),
        ("INDIE SYNTHWAVE 1", "Synthwave"),
        ("INDIE SYNTHWAVE", "Synthwave"),
        ("JAZZ", "Jazz"),
        ("LATIN CLUB", "Latin"),
        ("LATIN EDIT", "Latin"),
        ("LATIN", "Latin"),
        ("POP 00s", "Pop"),
        ("POP 1", "Pop"),
        ("POP 80s SYNTH", "Pop 80s"),
        ("POP 80s", "Pop 80s"),
        ("POP 90s EURODANCE", "House Eurodance"),
        ("POP 90s", "Pop 90s"),
        ("POP CHILL 1", "Pop"),
        ("POP CHILL HITAAT", "Pop"),
        ("POP CHILL", "Pop"),
        ("POP CLASSICAL", "Classical"),
        ("POP EDITS", "Pop"),
        ("POP INSTRUMENTAL", "Pop"),
        ("POP MAINSTREAM INSTRUMENTAL", "Pop"),
        ("POP MAINSTREAM", "Pop"),
        ("POP MASHUP", "Pop"),
        ("POP RANDOM", "Pop"),
        ("POP SUOMI", "Pop Suomi"),
        ("POP SYNTH", "Pop"),
        ("POP", "Pop"),
        ("REGGAE", "Reggae"),
        ("ROCK MASHUP", "Rock"),
        ("ROCK", "Rock"),
    ])
});

static COMMON_SUBSTITUTES: [(&str, &str); 7] = [
    ("\0", "/"),
    ("`", "'"),
    ("´", "'"),
    (" ,", ","),
    ("\\", "/"),
    ("/", " / "),
    ("\u{FFFD}", " "),
];

static REGEX_SUBSTITUTES: LazyLock<[(Regex, &'static str); 5]> = LazyLock::new(|| {
    [
        // Replace various opening bracket types with "("
        (Regex::new(r"[\[{]+").unwrap(), "("),
        // Replace various closing bracket types with ")"
        (Regex::new(r"[]}]+").unwrap(), ")"),
        // Collapse multiple consecutive opening parentheses into one
        (Regex::new(r"\(\s*\){2,}").unwrap(), "("),
        // Collapse multiple consecutive closing parentheses into one
        (Regex::new(r"\)\s*\){2,}").unwrap(), ")"),
        // Collapse multiple spaces into a single space
        (Regex::new(r"\s{2,}").unwrap(), " "),
    ]
});

/// Map various genres to the correct version
static REGEX_MAPPINGS: LazyLock<[(Regex, &'static str); 42]> = LazyLock::new(|| {
    [
        (Regex::new(r"(?i)\br\s*[&'n]*\s*b\b").unwrap(), "R&B"),
        (Regex::new(r"(?i)\bother\b").unwrap(), ""),
        (Regex::new(r"(?i)\bAccapella\b").unwrap(), "Acapella"),
        (Regex::new(r"(?i)\bHip Hop\b").unwrap(), "Hip-Hop"),
        (Regex::new(r"(?i)\bHip / Hop\b").unwrap(), "Hip-Hop"),
        (Regex::new(r"(?i)\bHip-Hop 90's\b").unwrap(), "Hip-Hop 90s"),
        (Regex::new(r"(?i)\bHip-Hop 80's\b").unwrap(), "Hip-Hop 80s"),
        (Regex::new(r"(?i)\bHip-Hop 90$").unwrap(), "Hip-Hop 90s"),
        (Regex::new(r"(?i)\bHip-Hop 80$").unwrap(), "Hip-Hop 80s"),
        (Regex::new(r"(?i)\b90's Hip-Hop\b").unwrap(), "Hip-Hop 90s"),
        (Regex::new(r"(?i)\b80's Hip-Hop\b").unwrap(), "Hip-Hop 80s"),
        (Regex::new(r"(?i)\bHip-Hop / Rap\b").unwrap(), "Hip-Hop"),
        (Regex::new(r"(?i)\bRap & Hip-Hop\b").unwrap(), "Hip-Hop"),
        (Regex::new(r"(?i)^Rap$").unwrap(), "Hip-Hop"),
        (Regex::new(r"(?i)\bNu Disco / Disco\b").unwrap(), "Disco Nu"),
        (Regex::new(r"(?i)\bSoul / Funk / Disco\b").unwrap(), "Funk"),
        (Regex::new(r"(?i)\bFunk / Soul\b").unwrap(), "Soul"),
        (Regex::new(r"(?i)\bSoul / Funk\b").unwrap(), "Soul"),
        (Regex::new(r"(?i)\bAfro beats\b").unwrap(), "Afrobeats"),
        (Regex::new(r"(?i)\bblend\b").unwrap(), "Mashup"),
        (Regex::new(r"(?i)\bDrum 'n' Bass\b").unwrap(), "Drum & Bass"),
        (Regex::new(r"(?i)\bD'n'B\b").unwrap(), "Drum & Bass"),
        (Regex::new(r"(?i)\bD&B\b").unwrap(), "Drum & Bass"),
        (Regex::new(r"(?i)\bDisco, Funk\b").unwrap(), "Disco"),
        (Regex::new(r"(?i)\bDisco Funk\b").unwrap(), "Disco"),
        (Regex::new(r"(?i)\bFunk / Boogie\b").unwrap(), "Funk Boogie"),
        (Regex::new(r"(?i)\bHouse / Funk\b").unwrap(), "House"),
        (Regex::new(r"(?i)\bHousemusic\b").unwrap(), "House"),
        (Regex::new(r"(?i)^House, Deep House\b").unwrap(), "House Deep"),
        (Regex::new(r"(?i)^West Coast$").unwrap(), "Hip-Hop West Coast"),
        (Regex::new(r"(?i)^West Coast, Hip-Hop$").unwrap(), "Hip-Hop West Coast"),
        (Regex::new(r"(?i)^Dance, Electro Pop$").unwrap(), "Dance"),
        (Regex::new(r"(?i)^90s X Golden Era$").unwrap(), "Hip-Hop 90s"),
        (Regex::new(r"(?i)\bB-more\b").unwrap(), "Baltimore Club"),
        (Regex::new(r"(?i)\bBmore\b").unwrap(), "Baltimore Club"),
        (Regex::new(r"(?i)\bBreaks, Funk\b").unwrap(), "Funk Breaks"),
        (Regex::new(r"(?i)\bClassic House\b").unwrap(), "House Old School"),
        (Regex::new(r"(?i)\bHouse Classic\b").unwrap(), "House Old School"),
        (Regex::new(r"(?i)^Italo$").unwrap(), "Disco Italo"),
        (Regex::new(r"(?i)\b70's\b").unwrap(), "70s"),
        (Regex::new(r"(?i)\b80's\b").unwrap(), "80s"),
        (Regex::new(r"(?i)\b90's\b").unwrap(), "90s"),
    ]
});

static RE_HOUSE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[^,]* House$").unwrap());

/// Format genre string.
pub fn format_genre(genre: &str) -> String {
    let mut formatted_genre = genre.trim().to_string();
    if formatted_genre.chars().count() < 3 {
        formatted_genre = String::new();
    }

    for (pattern, replacement) in &COMMON_SUBSTITUTES {
        formatted_genre = formatted_genre.replace(pattern, replacement);
    }

    for (regex, replacement) in REGEX_SUBSTITUTES.iter() {
        formatted_genre = regex.replace_all(&formatted_genre, *replacement).to_string();
    }

    for (regex, replacement) in REGEX_MAPPINGS.iter() {
        formatted_genre = regex.replace_all(&formatted_genre, *replacement).to_string();
    }

    formatted_genre = formatted_genre.replace("Original Samples / ", "").replace(" / ", ", ");

    reorder_house_genres(&mut formatted_genre);
    formatting::fix_whitespace(&mut formatted_genre);

    formatted_genre.replace(" / ", ", ")
}

/// Reorder house genres to start with "House".
///
/// For example, "Tech House" -> "House Tech".
fn reorder_house_genres(genre: &mut String) {
    if RE_HOUSE.is_match(genre) {
        let parts: Vec<&str> = genre.split(' ').collect();
        if let Some((last, elements)) = parts.split_last() {
            *genre = format!("{} {}", last, elements.join(" "));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rnb() {
        assert_eq!(format_genre(" Rnb   "), "R&B");
        assert_eq!(format_genre("R'n'B"), "R&B");
        assert_eq!(format_genre("R&B"), "R&B");
    }

    #[test]
    fn test_formatting() {
        assert_eq!(format_genre("Hip\\Hop"), "Hip-Hop");
        assert_eq!(format_genre("Hip/Hop"), "Hip-Hop");
        assert_eq!(format_genre("Hip  Hop"), "Hip-Hop");
        assert_eq!(format_genre("Jazz\u{FFFD}Blues"), "Jazz Blues");
        assert_eq!(format_genre("Hi"), "");
    }

    #[test]
    fn test_genre_mappings() {
        assert_eq!(format_genre(" other "), "");
        assert_eq!(format_genre("Other"), "");
        assert_eq!(format_genre("Funk 80's"), "Funk 80s");
        assert_eq!(format_genre("Hip-Hop 80's"), "Hip-Hop 80s");
        assert_eq!(format_genre("Hip-Hop 90's"), "Hip-Hop 90s");
        assert_eq!(format_genre("90's"), "90s");
        assert_eq!(format_genre("70's"), "70s");
    }

    #[test]
    fn test_house_genre_reordering() {
        assert_eq!(format_genre("Deep    House"), "House Deep");
        assert_eq!(format_genre("Progressive House"), "House Progressive");
    }
}
