use std::cmp::Ordering;

use lazy_static::lazy_static;
use regex::{Captures, Regex};

lazy_static! {
    static ref COMMON_SUBSTITUTES: [(&'static str, &'static str); 23] = [
        ("\0", "/"),
        ("`", "'"),
        ("´", "'"),
        (")(", ") ("),
        ("()", " "),
        (") - (", ""),
        (" - (", " ("),
        ("(- ", "("),
        ("( - ", "("),
        (" -)", " )"),
        (" - ) ", ")"),
        (" )", ")"),
        ("( ", "("),
        ("...", " "),
        ("..", " "),
        (" feat. - ", " feat. "),
        (" feat.-", " feat. "),
        ("o¨", "ö"),
        ("e¨", "ë"),
        (" ,", ","),
        ("\u{FFFD}", " "),
        // Replace en and em dashes with regular dash
        ("–", "-"),
        ("—", "-"),
    ];
    static ref TITLE_SUBSTITUTES: [(&'static str, &'static str); 15] = [
        ("(Original Mix/", "("),
        ("12\"", "12''"),
        (" (12 Version) ", " (12'' Version) "),
        ("(Inst)", "(Instrumental)"),
        (" W/Drums", " With Drums"),
        ("), Pt. 1", ") (Pt. 1)"),
        ("/Cyberkid ", " - Cyberkid "),
        ("-Dirty/Beat Junkie Sound ", " - Dirty Beat Junkie Sound "),
        ("-DirtyBeat Junkie Sound ", " - Dirty Beat Junkie Sound "),
        ("/Clean-Beat Junkie Sound ", " - Clean Beat Junkie Sound "),
        ("-Clean/Beat Junkie Sound ", " - Clean Beat Junkie Sound "),
        ("-CleanBeat Junkie Sound ", " - Clean Beat Junkie Sound "),
        (" Version/cyberkid ", " Version - Cyberkid "),
        ("/Beat Junkie ", " - Beat Junkie "),
        ("(Clean- ", "(Clean "),
    ];
    static ref REGEX_SUBSTITUTES: [(Regex, &'static str); 12] = [
        // Replace various opening bracket types with "("
        (Regex::new(r"[\[{]+").unwrap(), "("),
        // Replace various closing bracket types with ")"
        (Regex::new(r"[\]}]+").unwrap(), ")"),
        // Collapse multiple exclamation marks into one
        (Regex::new(r"!{2,}").unwrap(), "!"),
        // Collapse multiple periods into a single period
        (Regex::new(r"\.{2,}").unwrap(), "."),
        // Remove empty parentheses
        (Regex::new(r"\(\s*?\)").unwrap(), ""),
        // Ensure a space before an opening parenthesis
        (Regex::new(r"(\S)\(").unwrap(), "$1 ("),
        // Ensure a space after a closing parenthesis
        (Regex::new(r"\)([A-Za-z0-9])").unwrap(), ") $1"),
        // Collapse multiple consecutive opening parentheses into one
        (Regex::new(r"\(\s*\){2,}").unwrap(), "("),
        // Collapse multiple consecutive closing parentheses into one
        (Regex::new(r"\)\s*\){2,}").unwrap(), ")"),
        // Transforms underscore-wrapped text into single-quoted text
        (Regex::new(r"\s_(.*?)_\s").unwrap(), " '$1' "),
        // Remove asterisks after a word boundary
        (Regex::new(r"\s\*+\b").unwrap(), ""),
        // Collapses multiple spaces into a single space
        (Regex::new(r"\s+").unwrap(), " "),
    ];
    static ref REGEX_NAME_SUBSTITUTES: [(Regex, &'static str); 41] = [
        // Matches "12 Inch" or "12Inch" with optional space, case-insensitive
        (Regex::new(r"(?i)\b12\s?inch\b").unwrap(), "12''"),
        // Matches "12in" or "12 in" with optional space, case-insensitive
        (Regex::new(r"(?i)\b12\s?in\b").unwrap(), "12''"),
        // Matches "7 Inch" or "7Inch" with optional space, case-insensitive
        (Regex::new(r"(?i)\b7\s?inch\b").unwrap(), "7''"),
        // Matches "7in" or "7 in" with optional space, case-insensitive
        (Regex::new(r"(?i)\b7\s?in\b").unwrap(), "7''"),
        // Standardize various forms of "featuring" to "feat."
        (Regex::new(r"(?i)\b(?:feat\.?|ft\.?|featuring)\b").unwrap(), "feat."),
        (Regex::new(r"(?i)\(\s*(?:feat\.?|ft\.?|featuring)\b").unwrap(), "(feat."),
        // Standardize "w/" to "feat."
        (Regex::new(r"(?i)\sW/").unwrap(), " feat. "),
        // Remove trademark symbols
        (Regex::new(r"[®™]").unwrap(), ""),
        // Correct name for "Missy Elliott"
        (
            Regex::new(r"(?i)\bMissy Elliot\b|\bMissy Elliot$").unwrap(),
            "Missy Elliott",
        ),
        // Correct name for "Gang Starr"
        (Regex::new(r"(?i)\bGangstarr\b|\bGangstarr$").unwrap(), "Gang Starr"),
        // Fix spelling for "You're"
        (Regex::new(r"(?i)\bYoure\b").unwrap(), "You're"),
        // Fix spelling for "I'm"
        (Regex::new(r"(?i)\bIm\b").unwrap(), "I'm"),
        // Fix spelling for "You've"
        (Regex::new(r"(?i)\bYouve\b").unwrap(), "You've"),
        // Fix spelling for "Can't"
        (Regex::new(r"(?i)\bCant\b").unwrap(), "Can't"),
        // Fix spelling for "Won't"
        (Regex::new(r"(?i)\bWont\b").unwrap(), "Won't"),
        // Fix spelling for "Don't"
        (Regex::new(r"(?i)\bDont\b").unwrap(), "Don't"),
        // Fix capitalization for "DJ"
        (Regex::new(r"(?i)\bDj\b").unwrap(), "DJ"),
        // Ensure one whitespace after "feat."
        (Regex::new(r"\bfeat\.([A-Za-z0-9])").unwrap(), "feat. $1"),
        (Regex::new(r"(?i)\b(dirty!)\b").unwrap(), "(Dirty)"),
        // Removes "Original Mix" with case-insensitivity
        (Regex::new(r"(?i)\b\(original mix\)\b").unwrap(), ""),
        // Removes "DJCity" with case-insensitivity
        (Regex::new(r"(?i)\bdjcity\b").unwrap(), ""),
        (Regex::new(r"(?i)\bintro - clean\b").unwrap(), "Clean Intro"),
        (Regex::new(r"(?i)\bintro - dirty\b").unwrap(), "Dirty Intro"),
        (Regex::new(r"(?i)\(clean - intro\)").unwrap(), "(Clean Intro)"),
        (Regex::new(r"(?i)\(dirty - intro\)").unwrap(), "(Dirty Intro)"),
        (Regex::new(r"(?i)\bIn[:\s/+\-&]*out\b").unwrap(), "In-Out"),
        (Regex::new(r"(?i)\bIntro[:\s/+\-&]*outro\b").unwrap(), "Intro"),
        (Regex::new(r"(?i)\bAca In\b").unwrap(), "Acapella Intro"),
        (Regex::new(r"(?i)\bAca intro[:\s/+\-&]*aca outro\b").unwrap(), "Acapella In-Out"),
        (Regex::new(r"(?i)\bAcapella Intro[:\s/+\-&]*aca out\b").unwrap(), "Acapella In-Out"),
        (Regex::new(r"(?i)\bAca Out\b").unwrap(), "Acapella Out"),
        (Regex::new(r"(?i)\bAcap-In\b").unwrap(), "Acapella Intro"),
        (Regex::new(r"(?i)\bAcap - diy\b").unwrap(), "Acapella DIY"),
        (Regex::new(r"(?i)\bAcap in[:\s/+\-&]*out\b").unwrap(), "Acapella In-Out"),
        (Regex::new(r"(?i)\bAcap\b").unwrap(), "Acapella"),
        (Regex::new(r"(?i)\bAcapella[\s/+\-]*In[:\s/+\-&]*Out\b").unwrap(), "Acapella In-Out"),
        (Regex::new(r"(?i)\bAcapella[\s/+\-]*In\b").unwrap(), "Acapella Intro"),
        (Regex::new(r"(?i)\bAcapella Intro[:\s/+\-&]*Out\b").unwrap(), "Acapella In-Out"),
        (Regex::new(r"(?i)\bAcapella-Intro[:\s/+\-&]*Out\b").unwrap(), "Acapella In-Out"),
        (Regex::new(r"(?i)\bAcapella-Intro\b").unwrap(), "Acapella Intro"),
        (Regex::new(r"(?i)\bAcapella-out\b").unwrap(), "Acapella Out"),
    ];
    static ref REGEX_FILENAME_SUBSTITUTES: [(Regex, &'static str); 3] = [
        // Replace double quotes with two single quotes
        (Regex::new("\"").unwrap(), "''"),
        // Replace characters that are not allowed in filenames with a hyphen
        (Regex::new(r"([\\/<>|:\*\?])").unwrap(), "-"),
        // Collapse multiple spaces into a single space
        (Regex::new(r"\s+").unwrap(), " "),
    ];
    // Matches "feat." followed by any text until a dash, parenthesis, or end of string
    static ref RE_FEAT: Regex = Regex::new(r"\bfeat\. .*?( -|\(|\)|$)").unwrap();

    // Matches text after a closing parenthesis until the next opening parenthesis
    static ref RE_TEXT_AFTER_PARENTHESES: Regex = Regex::new(r"\)\s(.*?)\s\(").unwrap();

    // Matches BPM information inside parentheses at the end of a string,
    // allowing for decimal BPMs or BPM with a trailing "a"
    static ref RE_BPM_IN_PARENTHESES: Regex = Regex::new(r" \((\d{2,3}(\.\d)?|\d{2,3} \d{1,2}a)\)$").unwrap();

    // Matches BPM with an optional key, formatted within parentheses at the end of a string
    static ref RE_BPM_WITH_KEY: Regex = Regex::new(r"\s\(\d{1,3}(?:\s\d{1,2})?\s?[a-zA-Z]\)$").unwrap();

    // Matches BPM followed by two or three letters (likely denoting key or mode),
    // formatted within parentheses at the end of a string
    static ref RE_BPM_WITH_TEXT_PARENTHESES: Regex = Regex::new(r"\s\(\d{2,3}\s?[a-zA-Z]{2,3}\)$").unwrap();
    static ref RE_BPM_WITH_TEXT: Regex = Regex::new(r"\b\d{2,3}\s?[a-zA-Z]{2,3}\)$").unwrap();

    // Matches any text within parentheses that contains a dash, separating it into two groups
    static ref RE_DASH_IN_PARENTHESES: Regex = Regex::new(r"\((.*?) - (.*?)\)").unwrap();

    // Matches variations on "and" in feat artist names
    static ref RE_FEAT_AND: Regex = Regex::new(r"(?i),?\s+and\s+").unwrap();

    // Collapse multiple spaces into a single space
    static ref RE_MULTIPLE_SPACES: Regex = Regex::new(r"\s{2,}").unwrap();

    static ref RE_WWW: Regex = Regex::new(r"(?i)^www\.").unwrap();
}

/// Return formatted artist and title string.
pub fn format_tags_for_artist_and_title(artist: &str, title: &str) -> (String, String) {
    let mut formatted_artist = artist.to_string();
    let mut formatted_title = title.to_string();

    // Remove an extra file extension from the end
    let extensions = [".mp3", ".flac", ".aif", ".aiff", ".m4a"];
    for ext in &extensions {
        if formatted_artist.to_lowercase().ends_with(ext) {
            formatted_artist = formatted_artist[0..formatted_artist.len() - ext.len()].to_string();
        }
        if formatted_title.to_lowercase().ends_with(ext) {
            formatted_title = formatted_title[0..formatted_title.len() - ext.len()].to_string();
        }
    }

    for (pattern, replacement) in COMMON_SUBSTITUTES.iter() {
        formatted_artist = formatted_artist.replace(pattern, replacement);
        formatted_title = formatted_title.replace(pattern, replacement);
    }

    for (pattern, replacement) in TITLE_SUBSTITUTES.iter() {
        formatted_title = formatted_title.replace(pattern, replacement);
    }

    for (regex, replacement) in REGEX_NAME_SUBSTITUTES.iter() {
        formatted_artist = regex.replace_all(&formatted_artist, *replacement).to_string();
        formatted_title = regex.replace_all(&formatted_title, *replacement).to_string();
    }

    for (regex, replacement) in REGEX_SUBSTITUTES.iter() {
        formatted_artist = regex.replace_all(&formatted_artist, *replacement).to_string();
        formatted_title = regex.replace_all(&formatted_title, *replacement).to_string();
    }

    // Remove duplicate artist name from title
    let artist_with_dash = format!("{formatted_artist} - ");
    if formatted_title.starts_with(&artist_with_dash) {
        formatted_title = formatted_title.replacen(&artist_with_dash, "", 1);
    }

    // Artist name should not start with a dot since this will make it a hidden file
    formatted_artist = formatted_artist.trim_start_matches('.').to_string();

    use_parenthesis_for_mix(&mut formatted_title);
    move_feat_from_title_to_artist(&mut formatted_artist, &mut formatted_title);
    replace_dash_in_parentheses(&mut formatted_title);
    fix_nested_parentheses(&mut formatted_title);
    wrap_text_after_parentheses(&mut formatted_title);
    remove_bpm_in_parentheses_from_end(&mut formatted_title);
    remove_unmatched_closing_parenthesis(&mut formatted_artist);

    // TODO: Fix above so this is not needed
    formatted_title = formatted_title.replace("((", "(").replace("))", ")");

    extract_feat_from_parentheses(&mut formatted_artist);
    balance_parenthesis(&mut formatted_title);

    for (regex, replacement) in REGEX_SUBSTITUTES.iter() {
        formatted_artist = regex.replace_all(&formatted_artist, *replacement).to_string();
        formatted_title = regex.replace_all(&formatted_title, *replacement).to_string();
    }

    for (pattern, replacement) in COMMON_SUBSTITUTES.iter() {
        formatted_artist = formatted_artist.replace(pattern, replacement);
        formatted_title = formatted_title.replace(pattern, replacement);
    }

    (formatted_artist.trim().to_string(), formatted_title.trim().to_string())
}

/// Apply filename formatting.
pub fn format_filename(artist: &str, title: &str) -> (String, String) {
    let mut formatted_artist = artist.to_string();
    let mut formatted_title = title.to_string();

    for (regex, replacement) in REGEX_FILENAME_SUBSTITUTES.iter() {
        formatted_artist = regex.replace_all(&formatted_artist, *replacement).to_string();
        formatted_title = regex.replace_all(&formatted_title, *replacement).to_string();
    }

    (formatted_artist.trim().to_string(), formatted_title.trim().to_string())
}

pub fn format_album(album: &str) -> String {
    let mut formatted_album = album.trim().to_string();
    formatted_album = RE_WWW.replace(&formatted_album, "").to_string();
    fix_whitespace(&mut formatted_album);
    formatted_album
}

pub fn fix_whitespace(text: &mut String) {
    *text = RE_MULTIPLE_SPACES.replace_all(text, " ").to_string().trim().to_string();
}

/// Check parenthesis counts match and insert missing.
fn balance_parenthesis(title: &mut String) {
    let open_count = title.matches('(').count();
    let close_count = title.matches(')').count();
    match open_count.cmp(&close_count) {
        Ordering::Greater => add_missing_closing_parentheses(title),
        Ordering::Less => add_missing_opening_parentheses(title),
        _ => {}
    }
}

fn remove_unmatched_closing_parenthesis(input: &mut String) {
    *input = input.trim().to_string();
    if input.ends_with(')') && !input.contains('(') {
        input.pop();
    }
}

fn move_feat_from_title_to_artist(artist: &mut String, title: &mut String) {
    if let Some(feat_match) = RE_FEAT.find(&title.clone()) {
        let feat = feat_match
            .as_str()
            .trim_end_matches(|c| c == '(' || c == ')' || c == '-');

        // Remove the feat from the title
        *title = title.replace(feat, "").trim().to_string();

        // Format feat artists string: remove "feat. ", and change all "and" variations to "&"
        let feat = RE_FEAT_AND
            .replace_all(&feat.replacen("feat. ", "", 1), " & ")
            .trim()
            .to_string();

        // Split featuring artists on common delimiters and handle them individually
        let feat_artists: Vec<String> = feat
            .split(&['&', ',', '+'][..])
            .map(str::trim)
            .map(|s| s.to_string())
            .collect();

        for feat_artist in &feat_artists {
            for delimiter in [", ", " & ", " and ", " + "] {
                // Remove the individual featuring artist from the artist string if present
                *artist = artist
                    .replace(&format!("{delimiter}{feat_artist}"), "")
                    .replace(&format!("{feat_artist}{delimiter}"), "");
            }
        }

        let formatted_feat = format!(" feat. {feat}");
        if !artist.contains(&formatted_feat) {
            artist.push_str(&formatted_feat);
        }
    }
}

fn add_missing_closing_parentheses(text: &mut String) {
    let mut open_count: usize = 0;
    let mut result = String::new();

    for char in text.chars() {
        match char {
            '(' => {
                if open_count > 0 {
                    result.push_str(") ");
                    open_count -= 1;
                } else {
                    open_count += 1;
                }
            }
            ')' => {
                open_count = open_count.saturating_sub(1);
            }
            _ => {}
        }
        result.push(char);
    }

    for _ in 0..open_count {
        result.push(')');
    }

    *text = result;
}

fn add_missing_opening_parentheses(text: &mut String) {
    let mut open_count: usize = 0;
    let mut result = String::new();

    for char in text.chars().rev() {
        match char {
            ')' => {
                if open_count > 0 {
                    result.push_str(" (");
                    open_count -= 1;
                } else {
                    open_count += 1;
                }
            }
            '(' => {
                open_count = open_count.saturating_sub(1);
            }
            _ => {}
        }
        result.push(char);
    }

    for _ in 0..open_count {
        result.push('(');
    }

    *text = result.chars().rev().collect();
}

fn use_parenthesis_for_mix(title: &mut String) {
    if title.contains(" - ") {
        if let Some(mut index) = title.find(" - ") {
            let new_title = title.replacen(" - ", " (", 1);
            title.clear();
            title.push_str(&new_title);
            index += 2;

            // Check for " (" after the replaced part
            if let Some(insert_index) = title[index..].find(" (").map(|i| i + index) {
                title.insert(insert_index, ')');
            } else {
                // Add a closing parenthesis at the end
                title.push(')');
            }
        }
    }
}

fn fix_nested_parentheses(text: &mut String) {
    // Initialize a stack to keep track of parentheses
    let mut stack = Vec::new();
    let mut result = String::new();

    for char in text.chars() {
        match char {
            '(' => {
                // If the stack is not empty and the top element is also '(', add a closing ')' before the new '('
                if let Some(&last_char) = stack.last() {
                    if last_char == '(' {
                        result.push_str(") ");
                    }
                }
                stack.push(char);
                result.push(char);
            }
            ')' => {
                // If the stack is not empty, pop an element from the stack
                if stack.pop().is_some() {
                    // Add the closing parenthesis only if the stack is empty or the top element is not '('
                    if stack.is_empty() || *stack.last().unwrap() != '(' {
                        result.push(char);
                    }
                }
            }
            _ => {
                // Add any other characters to the result
                result.push(char);
            }
        }
    }

    // If there are any remaining opening parentheses, close them
    while stack.pop().is_some() {
        result.push(')');
    }

    *text = result
        .replace(" )", ")")
        .replace("( ", "(")
        .replace(" ()", "")
        .replace("() ", "");
}

fn extract_feat_from_parentheses(artist: &mut String) {
    let start_pattern = "(feat. ";
    if let Some(start) = artist.find(start_pattern) {
        if let Some(end) = artist[start..].find(')') {
            let feature_part = &artist[start..start + end + 1];
            *artist = artist.replacen(feature_part, &feature_part[1..feature_part.len() - 1], 1);
        }
    }
}

fn remove_bpm_in_parentheses_from_end(text: &mut String) {
    // Skip some valid titles
    let suffixes = [" (4u)", "33rpm)", "45rpm)", " mix)", " dub)", " eq)", " rip)"];
    let text_lower = text.to_lowercase();
    if suffixes.iter().any(|suffix| text_lower.ends_with(suffix)) {
        return;
    }

    let mut result = text.to_string();
    result = RE_BPM_IN_PARENTHESES.replace_all(&result, "").to_string();
    result = RE_BPM_WITH_KEY.replace_all(&result, "").to_string();
    result = RE_BPM_WITH_TEXT_PARENTHESES.replace_all(&result, "").to_string();
    result = RE_BPM_WITH_TEXT.replace_all(&result, "").to_string();

    *text = result;
}

fn wrap_text_after_parentheses(text: &mut String) {
    let (start, rest) = if text.starts_with('(') {
        // If the text starts with a parenthesis,
        // find the end of the first group and start replacing from there
        if let Some(index) = text.find(") ") {
            text.split_at(index + 2)
        } else {
            return;
        }
    } else {
        ("", text.as_str())
    };

    let mut result = RE_TEXT_AFTER_PARENTHESES
        .replace_all(rest, |caps: &Captures| format!(") ({}) (", &caps[1]))
        .to_string();

    if let Some(index) = result.rfind(')') {
        if index < result.len() - 1 {
            result.insert(index + 2, '(');
            result.push(')');
        }
    }

    *text = format!("{}{}", start, result);
}

fn replace_dash_in_parentheses(text: &mut String) {
    *text = RE_DASH_IN_PARENTHESES
        .replace_all(text, |caps: &Captures| format!("({}) ({})", &caps[1], &caps[2]))
        .to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_parenthesis_for_mix() {
        let mut title = "Azn Danza - Myles Club Edit".to_string();
        let correct_title = "Azn Danza (Myles Club Edit)".to_string();
        use_parenthesis_for_mix(&mut title);
        assert_eq!(title, correct_title);

        let mut title = "About Damn Time - Purple Disco Machine (Dirty Intro)".to_string();
        let correct_title = "About Damn Time (Purple Disco Machine) (Dirty Intro)".to_string();
        use_parenthesis_for_mix(&mut title);
        assert_eq!(title, correct_title);
    }

    #[test]
    fn test_extract_feat_from_parentheses() {
        let mut artist = "Major Lazer (feat. Laidback Luke & Ms. Dynamite)".to_string();
        let correct_artist = "Major Lazer feat. Laidback Luke & Ms. Dynamite".to_string();
        extract_feat_from_parentheses(&mut artist);
        assert_eq!(artist, correct_artist);
    }

    #[test]
    fn test_remove_bpm_in_parentheses_from_end() {
        let test_cases = [
            ("Hot (4U)", "Hot (4U)"),
            (
                "Favorite Song (Trayze My Boo Edit) (130 11a)",
                "Favorite Song (Trayze My Boo Edit)",
            ),
            ("Cut (Trayze Acapella Out) (136)", "Cut (Trayze Acapella Out)"),
            (
                "Signed, Sealed, Delivered (Trayze Nola Bounce Flip) (102 4a)",
                "Signed, Sealed, Delivered (Trayze Nola Bounce Flip)",
            ),
            ("Right Now (Facetyme Remix) (132 Ebm)", "Right Now (Facetyme Remix)"),
            (
                "Lift Me Up (Trayze Drop Leaf Edit) (89 11b)",
                "Lift Me Up (Trayze Drop Leaf Edit)",
            ),
            (
                "Lift Me Up (Trayze Drop Leaf Edit) (89 Mix)",
                "Lift Me Up (Trayze Drop Leaf Edit) (89 Mix)",
            ),
        ];

        for (input, expected) in test_cases {
            let mut input_string = input.to_string();
            remove_bpm_in_parentheses_from_end(&mut input_string);
            assert_eq!(input_string, expected);
        }
    }

    #[test]
    fn test_fix_nested_parentheses() {
        let test_cases = vec![
            ("Hello ((World))", "Hello (World)"),
            ("((Hello) World)", "(Hello World)"),
            ("Hello (World)", "Hello (World)"),
            ("(Hello) (World)", "(Hello) (World)"),
        ];

        for (input, expected) in test_cases {
            let mut input_string = input.to_string();
            fix_nested_parentheses(&mut input_string);
            assert_eq!(input_string, expected);
        }
    }

    #[test]
    fn test_wrap_text_after_parentheses() {
        let test_cases = vec![
            ("Hello (World) Test", "Hello (World) (Test)"),
            ("Hello (World) Test (Another)", "Hello (World) (Test) (Another)"),
            (
                "Come And Get Your Love (Nick Bike Extended Mix) (Instrumental) 2.2",
                "Come And Get Your Love (Nick Bike Extended Mix) (Instrumental) (2.2)",
            ),
            (
                "Come And Get Your Love (Nick Bike Remix) Extended Mix (Instrumental) 2.2",
                "Come And Get Your Love (Nick Bike Remix) (Extended Mix) (Instrumental) (2.2)",
            ),
            ("(You Make Me Feel) Mighty Real", "(You Make Me Feel) Mighty Real"),
            (
                "(You Make Me Feel) Mighty Real (Clean) 2.2",
                "(You Make Me Feel) Mighty Real (Clean) (2.2)",
            ),
        ];

        for (input, expected) in test_cases {
            let mut input_string = input.to_string();
            wrap_text_after_parentheses(&mut input_string);
            assert_eq!(input_string, expected);
        }
    }
}
