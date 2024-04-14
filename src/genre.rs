use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
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
    static ref GENRE_MAPPINGS: [(Regex, &'static str); 10] = [
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

    for (regex, replacement) in GENRE_MAPPINGS.iter() {
        formatted_genre = regex.replace_all(&formatted_genre, *replacement).to_string();
    }

    reorder_house_genres(&mut formatted_genre);

    formatted_genre
}

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
