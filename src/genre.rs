use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref GENRE_MAPPINGS: HashMap<&'static str, &'static str> = {
        HashMap::from([
            ("DISCO", "Disco"),
            ("DISCO 1", "Disco"),
            ("DISCO CLASSICS", "Disco"),
            ("DISCO EDITS", "Disco"),
            ("DISCO EDITS 1", "Disco"),
            ("DISCO EDITS WAACKING", "Disco"),
            ("DISCO ELECTRO", "Electro"),
            ("DISCO HI-NRG", "Disco Hi-NRG"),
            ("DISCO ITALO", "Disco Italo"),
            ("DISCO JAM", "Disco"),
            ("DISCO JAM 1", "Disco"),
            ("DRUM&BASS", "Drum & Bass"),
            ("DRUM&BASS CHILL", "Drum & Bass"),
            ("DRUM&BASS DARK", "Drum & Bass"),
            ("DRUM&BASS EDITS", "Drum & Bass"),
            ("DRUM&BASS POP", "Drum & Bass"),
            ("DUBSTEP", "Dubstep"),
            ("ELECTRONICA", "Electronica"),
            ("ELECTRONICA CHILL", "Electronica"),
            ("FUNK", "Funk"),
            ("FUNK 80s", "Funk 80's"),
            ("FUNK 80s 1", "Funk 80's"),
            ("FUNK BOOGIE", "Funk Boogie"),
            ("FUNK BOOGIE 1", "Funk Boogie"),
            ("FUNK BOOGIE EDITS", "Funk Boogie"),
            ("FUNK BOOGIE EDITS 1", "Funk Boogie"),
            ("FUNK BOOGIE JAM", "Funk Boogie"),
            ("FUNK BOOGIE POP", "Funk Boogie"),
            ("FUNK BREAKS", "Funk Breaks"),
            ("FUNK BREAKS 1", "Funk Breaks"),
            ("FUNK BREAKS CHILL", "Funk Breaks"),
            ("FUNK BREAKS EDITS", "Funk Breaks"),
            ("FUNK BREAKS JAM", "Funk Breaks"),
            ("FUNK BREAKS TOP ROCK", "Funk Breaks"),
            ("FUNK BREAKS TOP ROCK 1", "Funk Breaks"),
            ("FUNK CHILL", "Funk"),
            ("FUNK CHILL 1", "Funk"),
            ("FUNK CLASSICS", "Funk"),
            ("FUNK CLASSICS 1", "Funk"),
            ("FUNK EDITS", "Funk"),
            ("FUNK EDITS 1", "Funk"),
            ("FUNK EDITS 80's", "Funk"),
            ("FUNK EDITS BATTLE", "Funk"),
            ("FUNK EDITS CHILL", "Funk"),
            ("FUNK EDITS CHILL 1", "Funk"),
            ("FUNK EDITS CLASSICS", "Funk"),
            ("FUNK EDITS JAM", "Funk"),
            ("FUNK EDITS JAM 1", "Funk"),
            ("FUNK JAM", "Funk"),
            ("FUNK JAM 1", "Funk"),
            ("FUNK JAM CHILL", "Funk"),
            ("FUNK LOVE", "Funk"),
            ("FUNK LOVE 1", "Funk"),
            ("FUNK LOVE CHILL", "Funk"),
            ("FUNK LOVE HITAAT", "Funk"),
            ("FUNK MASHUP", "Funk"),
            ("FUNK PARTY", "Funk"),
            ("FUNK PARTY 1", "Funk"),
            ("FUNK SAMPLE", "Funk"),
            ("FUNKY", "Funk"),
            ("FUNKY 1", "Funk"),
            ("FUNKY AFRO & LATIN", "Latin"),
            ("FUNKY BOOGIE", "Funk Boogie"),
            ("FUNKY BOOGIE 1", "Funk Boogie"),
            ("FUNKY BOOGIE JAM", "Funk Boogie"),
            ("FUNKY BREAKS", "Funk Breaks"),
            ("FUNKY BREAKS BATTLE", "Funk Breaks"),
            ("FUNKY BREAKS CHILL", "Funk Breaks"),
            ("FUNKY BREAKS GHETTOFUNK", "Ghettofunk"),
            ("FUNKY BREAKS JAM", "Funk Breaks"),
            ("FUNKY BREAKS JAM 1", "Funk Breaks"),
            ("FUNKY BREAKS LATIN", "Funk Breaks"),
            ("FUNKY BREAKS PARTY", "Funk Breaks"),
            ("FUNKY BREAKS RAP", "Funk Breaks"),
            ("FUNKY BREAKS RAP JAM", "Funk Breaks"),
            ("FUNKY BREAKS REGGAE", "Funk Breaks"),
            ("FUNKY CHILL", "Funk"),
            ("FUNKY JAM", "Disco Nu"),
            ("FUNKY JAM 1", "Disco Nu"),
            ("FUNKY JAM CHILL", "Disco Nu"),
            ("FUNKY NU DISCO", "Disco Nu"),
            ("FUNKY POP", "Funk"),
            ("FUNKY POP 1", "Funk"),
            ("HIPHOP", "Hip-Hop"),
            ("HIPHOP 1", "Hip-Hop"),
            ("HIPHOP AFRO BEATS", "Afrobeats"),
            ("HIPHOP BATTLE", "Hip-Hop"),
            ("HIPHOP BATTLE 1", "Hip-Hop"),
            ("HIPHOP BATTLE 90's", "Hip-Hop 90's"),
            ("HIPHOP BATTLE BREAKS", "Hip-Hop"),
            ("HIPHOP BATTLE CLUB", "Hip-Hop"),
            ("HIPHOP BATTLE INSTRUMENTAL", "Hip-Hop"),
            ("HIPHOP BATTLE NEW", "Hip-Hop"),
            ("HIPHOP BATTLE POPPING", "Hip-Hop"),
            ("HIPHOP BATTLE POPPING INSTRUMENTAL", "Hip-Hop"),
            ("HIPHOP BATTLE TRAP", "Hip-Hop Trap"),
            ("HIPHOP BEATS", "Hip-Hop"),
            ("HIPHOP BEATS 1", "Hip-Hop"),
            ("HIPHOP BEATS BATTLE", "Hip-Hop"),
            ("HIPHOP BEATS BATTLE 1", "Hip-Hop"),
            ("HIPHOP BEATS CHILL", "Hip-Hop"),
            ("HIPHOP BEATS ELECTRO", "Hip-Hop"),
            ("HIPHOP BEATS GLITCH", "Glitch Hop"),
            ("HIPHOP BEATS JAM", "Hip-Hop"),
            ("HIPHOP BEATS JAM 1", "Hip-Hop"),
            ("HIPHOP BEATS POPPING", "Hip-Hop"),
            ("HIPHOP BEATS POPPING JAM", "Hip-Hop"),
            ("HIPHOP BEATS TRAP", "Hip-Hop Trap"),
            ("HIPHOP CHILL", "Hip-Hop"),
            ("HIPHOP CHILL 1", "Hip-Hop"),
            ("HIPHOP CLUB", "Hip-Hop"),
            ("HIPHOP DANCEHALL", "Dancehall"),
            ("HIPHOP ELECTRO", "Electro"),
            ("HIPHOP FUTURE BASS", "Hip-Hop"),
            ("HIPHOP G-FUNK", "Hip-Hop G-Funk"),
            ("HIPHOP INSTRUMENTAL", "Hip-Hop"),
            ("HIPHOP INSTRUMENTAL 1", "Hip-Hop"),
            ("HIPHOP INSTRUMENTAL 90's", "Hip-Hop 90's"),
            ("HIPHOP INSTRUMENTAL BREAKS", "Hip-Hop"),
            ("HIPHOP INSTRUMENTAL CHILL", "Hip-Hop"),
            ("HIPHOP INSTRUMENTAL CLUB", "Hip-Hop"),
            ("HIPHOP INSTRUMENTAL G-FUNK", "Hip-Hop G-Funk"),
            ("HIPHOP INSTRUMENTAL JAM", "Hip-Hop"),
            ("HIPHOP INSTRUMENTAL JAM 1", "Hip-Hop"),
            ("HIPHOP INSTRUMENTAL PARTY", "Hip-Hop"),
            ("HIPHOP INSTRUMENTAL POP", "Hip-Hop"),
            ("HIPHOP INSTRUMENTAL RNB", "R&B"),
            ("HIPHOP INSTRUMENTAL SUOMIRAP", "Hip-Hop Suomirap"),
            ("HIPHOP JAM", "Hip-Hop"),
            ("HIPHOP JAM 1", "Hip-Hop"),
            ("HIPHOP JAM 90's", "Hip-Hop 90's"),
            ("HIPHOP JAM BREAKS", "Hip-Hop"),
            ("HIPHOP JAM CHILL", "Hip-Hop"),
            ("HIPHOP JAM EDITS", "Hip-Hop"),
            ("HIPHOP JAM G-FUNK", "Hip-Hop G-Funk"),
            ("HIPHOP JAM NEW", "Hip-Hop"),
            ("HIPHOP JERSEY CLUB", "Jersey Club"),
            ("HIPHOP NEW JACK SWING", "New Jack Swing"),
            ("HIPHOP NEW JACK SWING 1", "New Jack Swing"),
            ("HIPHOP NEW JACK SWING CHILL", "New Jack Swing"),
            ("HIPHOP OLD SCHOOL", "Hip-Hop"),
            ("HIPHOP PARTY", "Hip-Hop"),
            ("HIPHOP PARTY 1", "Hip-Hop"),
            ("HIPHOP PARTY 90's", "Hip-Hop 90's"),
            ("HIPHOP PARTY BREAKS", "Hip-Hop"),
            ("HIPHOP RNB", "R&B"),
            ("HIPHOP RNB 1", "R&B"),
            ("HIPHOP RNB 90's", "R&B"),
            ("HIPHOP RNB CHILL", "R&B"),
            ("HIPHOP RNB CHILL 1", "R&B"),
            ("HIPHOP RNB CLUB", "R&B"),
            ("HIPHOP RNB EDITS", "R&B"),
            ("HIPHOP SAMPLE", "Hip-Hop"),
            ("HIPHOP SUOMIRAP", "Hip-Hop Suomirap"),
            ("HIPHOP SUOMIRAP 1", "Hip-Hop Suomirap"),
            ("HIPHOP SUOMIRAP LOPETUS", "Hip-Hop Suomirap"),
            ("HIPHOP TRAP", "Hip-Hop Trap"),
            ("HIPHOP TRAP 1", "Hip-Hop Trap"),
            ("HIPHOP TRAP POP", "Hip-Hop Trap"),
            ("HOUSE", "House"),
            ("HOUSE 1", "House"),
            ("HOUSE ACID", "House Acid"),
            ("HOUSE AFRO", "House Afro"),
            ("HOUSE AFRO 1", "House Afro"),
            ("HOUSE AFRO AMAPIANO", ""),
            ("HOUSE AFRO BEATS", "Afrobeats"),
            ("HOUSE AFRO JAM", "House Afro"),
            ("HOUSE AFRO LATIN", "House Latin"),
            ("HOUSE AFRO LATIN CLUB", "House Latin"),
            ("HOUSE AFRO POP", "House Afro"),
            ("HOUSE AFRO TRIBAL", "House Tribal"),
            ("HOUSE AFRO TRIBAL CLUB", "House Tribal"),
            ("HOUSE AFRO TRIBAL JAM", "House Tribal"),
            ("HOUSE BAILE FUNK", "Baile Funk"),
            ("HOUSE BAILE FUNK POP", "Baile Funk"),
            ("HOUSE BALTIMORE", "Baltimore Club"),
            ("HOUSE BASS", "House"),
            ("HOUSE BASS 1", "House"),
            ("HOUSE BASS CLUB", "House"),
            ("HOUSE BASS FUTURE", "House Future"),
            ("HOUSE BASS POP", "House"),
            ("HOUSE BASS UG", "House"),
            ("HOUSE BASS UG 1", "House"),
            ("HOUSE BIG ROOM", "House"),
            ("HOUSE BOOGIE", "House"),
            ("HOUSE BREAKBEAT", "Breakbeat"),
            ("HOUSE BROKEN BEAT", "Broken Beat"),
            ("HOUSE CHILL", "House"),
            ("HOUSE CHILL 1", "House"),
            ("HOUSE CHILL AMBIENT", "House"),
            ("HOUSE DEEP", "House Deep"),
            ("HOUSE DEEP 1", "House Deep"),
            ("HOUSE DEEP BASS", "House Deep"),
            ("HOUSE DEEP BASS 1", "House Deep"),
            ("HOUSE DEEP CHILL 1", "House Deep"),
            ("HOUSE DEEP JAM", "House Deep"),
            ("HOUSE DEEP JAM 1", "House Deep"),
            ("HOUSE DEEP OLD", "House Deep"),
            ("HOUSE DEEP OLD 1", "House Deep"),
            ("HOUSE DEEP OLD JAM", "House Deep"),
            ("HOUSE DEEP OLD JAM 1", "House Deep"),
            ("HOUSE DEEP RAW", "House Deep"),
            ("HOUSE DEEP TECH", "House Deep"),
            ("HOUSE DEEP TECH 1", "House Deep"),
            ("HOUSE DEEP VOGUE", "House Deep"),
            ("HOUSE DISCO", "House Disco"),
            ("HOUSE DISCO 1", "House Disco"),
            ("HOUSE DISCO JAM", "House Disco"),
            ("HOUSE ELECTRO", "House Electro"),
            ("HOUSE ELECTRONICA", "House"),
            ("HOUSE EURODANCE", "Eurodance"),
            ("HOUSE FUNKY", "House Funky"),
            ("HOUSE FUNKY JAM", "House Funky"),
            ("HOUSE JAM", "House"),
            ("HOUSE JAM CHILL", "House"),
            ("HOUSE JAZZY", "House"),
            ("HOUSE MAINSTREAM", "House"),
            ("HOUSE MAINSTREAM INSTRUMENTAL", "House"),
            ("HOUSE MINIMAL", "House"),
            ("HOUSE MOOMBAHTON", "Moombahton"),
            ("HOUSE OLD SCHOOL", "House"),
            ("HOUSE OLD SCHOOL 1", "House"),
            ("HOUSE OLD SCHOOL CHILL", "House"),
            ("HOUSE OLD SCHOOL DISCO", "House"),
            ("HOUSE OLD SCHOOL HIP", "House"),
            ("HOUSE OLD SCHOOL JAM", "House"),
            ("HOUSE OLD SCHOOL POP", "House"),
            ("HOUSE OLD SCHOOL REMIX", "House"),
            ("HOUSE OLD SCHOOL VOGUE", "House"),
            ("HOUSE PIANO", "House"),
            ("HOUSE POP CLUB", "House"),
            ("HOUSE POP CLUB INSTRUMENTAL", "House"),
            ("HOUSE PROGRESSIVE", "House Progressive"),
            ("HOUSE PROGRESSIVE CLUB", "House Progressive"),
            ("HOUSE RAVE", "House"),
            ("HOUSE RAVE NU", "Rave"),
            ("HOUSE RAVE OLD SCHOOL", "Rave"),
            ("HOUSE REMIX", "House"),
            ("HOUSE SOULFUL", "House Soulful"),
            ("HOUSE SOULFUL CHILL", "House Soulful"),
            ("HOUSE SOULFUL VOCAL", "House Soulful"),
            ("HOUSE TECH", "House Tech"),
            ("HOUSE TECH 1", "House Tech"),
            ("HOUSE TECH CLUB", "House Tech"),
            ("HOUSE TECH DARK", "House Tech"),
            ("HOUSE TECH DARK 1", "House Tech"),
            ("HOUSE TECH JAM", "House Tech"),
            ("HOUSE TECH VOGUE", "House Tech"),
            ("HOUSE TECH VOGUE 1", "House Tech"),
            ("HOUSE TECHNO", "Techno"),
            ("HOUSE TECHNO DEEP", "Techno"),
            ("HOUSE TECHNO OLD SCHOOL", "Techno"),
            ("HOUSE TRANCE", "Trance"),
            ("HOUSE UK BEATS", "Electronic"),
            ("HOUSE UK FUNKY", "UK Funky"),
            ("HOUSE UK GARAGE", "UK Garage"),
            ("HOUSE VOGUE", "House"),
            ("HOUSE VOGUE BEATS", "Vogue Beats"),
            ("HOUSE VOGUE BEATS 1", "Vogue Beats"),
            ("HOUSE VOGUE BEATS ARMS HANDS", "Vogue Beats"),
            ("HOUSE VOGUE BEATS CLUB", "Vogue Beats"),
            ("HOUSE VOGUE BEATS DEEP", "Vogue Beats"),
            ("HOUSE VOGUE BEATS RUNWAY", "Vogue Beats"),
            ("HOUSE VOGUE FEMME", "Vogue Beats"),
            ("HOUSE VOGUE FEMME 1", "Vogue Beats"),
            ("HOUSE VOGUE FEMME VOCAL", "Vogue Beats"),
            ("HOUSE VOGUE JERSEY CLUB", "Jersey Club"),
            ("HOUSE VOGUE NEW WAY", "House"),
            ("HOUSE VOGUE OLD WAY", "House"),
            ("INDIE DANCE", "Indie Dance"),
            ("INDIE ELECTRO", "Electro"),
            ("INDIE SYNTHWAVE", "Synthwave"),
            ("INDIE SYNTHWAVE 1", "Synthwave"),
            ("JAZZ", "Jazz"),
            ("LATIN", "Latin"),
            ("LATIN CLUB", "Latin"),
            ("LATIN EDIT", "Latin"),
            ("POP", "Pop"),
            ("POP 00s", "Pop"),
            ("POP 1", "Pop"),
            ("POP 80s", "Pop 80's"),
            ("POP 90s", "Pop 90's"),
            ("POP CHILL", "Pop"),
            ("POP CHILL 1", "Pop"),
            ("POP CHILL HITAAT", "Pop"),
            ("POP CLASSICAL", "Classical"),
            ("POP EDITS", "Pop"),
            ("POP INSTRUMENTAL", "Pop"),
            ("POP ISKELMÄ", "Pop Iskelmä"),
            ("POP MAINSTREAM", "Pop"),
            ("POP MAINSTREAM INSTRUMENTAL", "Pop"),
            ("POP MASHUP", "Pop"),
            ("POP RANDOM", "Pop"),
            ("POP SYNTH", "Pop"),
            ("REGGAE", "Reggae"),
            ("ROCK", "Rock"),
            ("ROCK MASHUP", "Rock"),
        ])
    };

    static ref COMMON_SUBSTITUTES: [(&'static str, &'static str); 7] = [
        ("\0", "/"),
        ("`", "'"),
        ("´", "'"),
        (" ,", ","),
        ("\\", "/"),
        ("/", " / "),
        ("\u{FFFD}", " "),
    ];

    static ref REGEX_SUBSTITUTES: [(Regex, &'static str); 3] = [
        // Replace various opening bracket types with "("
        (Regex::new(r"[\[{]+").unwrap(), "("),
        // Replace various closing bracket types with ")"
        (Regex::new(r"[\]}]+").unwrap(), ")"),
        // Collapse multiple spaces into a single space
        (Regex::new(r"\s{2,}").unwrap(), " "),
    ];

    /// Map various genres to the correct version
    static ref REGEX_MAPPINGS: [(Regex, &'static str); 10] = [
        (Regex::new(r"(?i)\br\s*[&'n]*\s*b\b").unwrap(), "R&B"),
        (Regex::new(r"(?i)\bother\b").unwrap(), ""),
        (Regex::new(r"(?i)\bAccapella\b").unwrap(), "Acapella"),
        (Regex::new(r"(?i)\bHip Hop\b").unwrap(), "Hip-Hop"),
        (Regex::new(r"(?i)\bHip-Hop 90s\b").unwrap(), "Hip-Hop 90's"),
        (Regex::new(r"(?i)\bHip-Hop 80s\b").unwrap(), "Hip-Hop 80's"),
        (Regex::new(r"(?i)\bHip-Hop 90$").unwrap(), "Hip-Hop 90's"),
        (Regex::new(r"(?i)\bHip-Hop 80$").unwrap(), "Hip-Hop 80's"),
        (Regex::new(r"(?i)\b90's Hip-Hop").unwrap(), "Hip-Hop 90's"),
        (Regex::new(r"(?i)\b80's Hip-Hop").unwrap(), "Hip-Hop 80's"),
    ];

    static ref RE_HOUSE: Regex = Regex::new(r"^[^,]* House$").unwrap();
}

/// Format genre string
pub fn format_genre(genre: &str) -> String {
    let mut formatted_genre = genre.trim().to_string();
    if formatted_genre.chars().count() < 3 {
        formatted_genre = String::new();
    }

    for (pattern, replacement) in COMMON_SUBSTITUTES.iter() {
        formatted_genre = formatted_genre.replace(pattern, replacement);
    }

    for (regex, replacement) in REGEX_SUBSTITUTES.iter() {
        formatted_genre = regex.replace_all(&formatted_genre, *replacement).to_string();
    }

    for (regex, replacement) in REGEX_MAPPINGS.iter() {
        formatted_genre = regex.replace_all(&formatted_genre, *replacement).to_string();
    }

    reorder_house_genres(&mut formatted_genre);

    formatted_genre
}

/// Reorder house genres to start with "House".
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
    }

    #[test]
    fn test_formatting() {
        assert_eq!(format_genre("Hip\\Hop"), "Hip / Hop");
        assert_eq!(format_genre("Hip  Hop"), "Hip-Hop");
        assert_eq!(format_genre("Jazz\u{FFFD}Blues"), "Jazz Blues");
        assert_eq!(format_genre("Hi"), "");
    }

    #[test]
    fn test_genre_mappings() {
        assert_eq!(format_genre("Hip-Hop 90s"), "Hip-Hop 90's");
        assert_eq!(format_genre(" other "), "");
    }

    #[test]
    fn test_house_genre_reordering() {
        assert_eq!(format_genre("Deep    House"), "House Deep");
        assert_eq!(format_genre("Progressive House"), "House Progressive");
    }
}
