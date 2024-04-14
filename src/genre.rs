use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref REGEX_SUBSTITUTES: [(Regex, &'static str); 3] = [
        // Replace various opening bracket types with "("
        (Regex::new(r"[\[{]+").unwrap(), "("),
        // Replace various closing bracket types with ")"
        (Regex::new(r"[\]}]+").unwrap(), ")"),
        // Collapse multiple spaces into a single space
        (Regex::new(r"\s{2,}").unwrap(), " "),
    ];

    static ref RE_HOUSE: Regex = Regex::new(r"^[^,]* House$").unwrap();
}

/// Map various genres to the correct version
static GENRE_MAPPINGS: [(&str, &str); 8] = [
    ("rnb", "R&B"),
    ("r & b", "R&B"),
    ("r'n'b", "R&B"),
    ("other", ""),
    ("accapella", "Acapella"),
    ("hip hop", "Hip-Hop"),
    ("Hip-Hop 90", "Hip-Hop 90's"),
    ("Hip-Hop 80", "Hip-Hop 80's"),
];

pub fn format_genre(genre: &str) -> String {
    let mut formatted_genre = genre.trim().to_string();
    if formatted_genre.chars().count() < 3 {
        formatted_genre = String::new();
    }

    for (regex, replacement) in REGEX_SUBSTITUTES.iter() {
        formatted_genre = regex.replace_all(&formatted_genre, *replacement).to_string();
    }

    for (pattern, replacement) in GENRE_MAPPINGS.iter() {
        formatted_genre = formatted_genre.replace(pattern, replacement);
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
