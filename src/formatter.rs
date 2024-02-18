use lazy_static::lazy_static;
use regex::{Captures, Regex};

use std::cmp::Ordering;

lazy_static! {
    static ref COMMON_SUBSTITUTES: [(&'static str, &'static str); 18] = [
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
        (" ***", ""),
        (" **", ""),
        (" *", ""),
        ("*** ", ""),
        ("** ", ""),
        ("* ", ""),
    ];
    static ref TITLE_SUBSTITUTES: [(&'static str, &'static str); 31] = [
        (" (Dirty!)", " (Dirty)"),
        (" (Original Mix)", ""),
        (" DJcity", ""),
        (" DJCity", ""),
        ("(DJcity - ", "("),
        ("(DJcity ", "("),
        ("DJcity ", ""),
        ("DJCity ", ""),
        ("12\"", "12''"),
        ("Intro - Dirty", "Dirty Intro"),
        ("Intro - Clean", "Clean Intro"),
        ("(Dirty - Intro)", "(Dirty Intro)"),
        ("(Clean - Intro)", "(Clean Intro)"),
        ("Acap - DIY", "Acapella DIY"),
        ("(Acap)", "(Acapella)"),
        ("Acap ", "Acapella "),
        ("(Inst)", "(Instrumental)"),
        (" 12 Inch ", " 12'' "),
        ("(12 Inch ", "(12'' "),
        (" 12in ", " 12'' "),
        ("(12in ", "(12'' "),
        ("(7in ", "(7'' "),
        (" 7in ", " 7'' "),
        ("Intro/Outro", "Intro-Outro"),
        (" In/Out", " Intro-Outro"),
        ("In/Out ", "Intro-Outro "),
        ("Aca In/Aca Out", "Acapella In-Out"),
        ("Intro/Outro", "Intro"),
        ("Intro-Outro", "Intro"),
        ("In+Out", "In-Out"),
        ("In+out", "In-Out"),
    ];
    static ref REGEX_SUBSTITUTES: [(Regex, &'static str); 18] = [
        (Regex::new(r"(?i)\b(?:feat\.?|ft\.?|featuring)\b").unwrap(), "feat."),
        (Regex::new(r"(?i)\(\s*(?:feat\.?|ft\.?|featuring)\b").unwrap(), "(feat."),
        (Regex::new(r"[\[{]+").unwrap(), "("),
        (Regex::new(r"[\]}]+").unwrap(), ")"),
        (Regex::new(r"!{2,}").unwrap(), "!"),
        (Regex::new(r"\s{2,}").unwrap(), " "),
        (Regex::new(r"\.{2,}").unwrap(), "."),
        (Regex::new(r"\(\s*?\)").unwrap(), ""),
        (Regex::new(r"(\S)\(").unwrap(), "$1 ("),
        (Regex::new(r"\)(\S)").unwrap(), ") $1"),
        (Regex::new(r"\(\s*\){2,}").unwrap(), "("),
        (Regex::new(r"\)\s*\){2,}").unwrap(), ")"),
        (Regex::new(r"\({2,}").unwrap(), "("),
        (Regex::new(r"\){2,}").unwrap(), ")"),
        (
            Regex::new(r"(?i)\bMissy Elliot\b|\bMissy Elliot$").unwrap(),
            "Missy Elliott",
        ),
        (Regex::new(r"(?i)\bGangstarr\b|\bGangstarr$").unwrap(), "Gang Starr"),
        (Regex::new(r"(?i)\sW/").unwrap(), " feat. "),
        (Regex::new(r"\s+").unwrap(), " "),
    ];
    static ref FILENAME_REGEX_SUBSTITUTES: [(Regex, &'static str); 3] = [
        (Regex::new("\"").unwrap(), "''"),
        (Regex::new(r"([\\/<>|:\*\?])+").unwrap(), "-"),
        (Regex::new(r"\s+").unwrap(), " "),
    ];
    static ref FEAT_REGEX: Regex = Regex::new(r"feat\. .*?( -|\(|\)|$)").unwrap();
    static ref TEXT_AFTER_PARENTHESES: Regex = Regex::new(r"\)\s(.*?)\s\(").unwrap();
    static ref BPM_IN_PARENTHESES: Regex = Regex::new(r" \((\d{2,3}(\.\d)?|\d{2,3} \d{1,2}a)\)$").unwrap();
    static ref BPM_WITH_KEY: Regex = Regex::new(r"\s\(\d{1,2}(?:\s\d{1,2})?\s?[a-zA-Z]\)$").unwrap();
    static ref BPM_WITH_LETTERS: Regex = Regex::new(r"\s\(\d{2,3}\s?[a-zA-Z]{2,3}\)$").unwrap();
    static ref DASH_IN_PARENTHESES: Regex = Regex::new(r"\((.*?) - (.*?)\)").unwrap();
}

/// Return formatted artist and title string.
pub fn format_tags(artist: &str, title: &str) -> (String, String) {
    let mut formatted_artist = artist.to_string();
    let mut formatted_title = title.to_string();

    // Remove extra extension from end
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

    for (regex, replacement) in REGEX_SUBSTITUTES.iter() {
        formatted_artist = regex.replace_all(&formatted_artist, *replacement).to_string();
        formatted_title = regex.replace_all(&formatted_title, *replacement).to_string();
    }

    use_parenthesis_for_mix(&mut formatted_title);
    move_feat_from_title_to_artist(&mut formatted_artist, &mut formatted_title);
    replace_dash_in_parentheses(&mut formatted_title);
    fix_nested_parentheses(&mut formatted_title);
    wrap_text_after_parentheses(&mut formatted_title);
    remove_bpm_in_parentheses_from_end(&mut formatted_title);

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

/// Apply filename formatting
pub fn format_filename(artist: &str, title: &str) -> (String, String) {
    let mut formatted_artist = artist.to_string();
    let mut formatted_title = title.to_string();

    for (regex, replacement) in FILENAME_REGEX_SUBSTITUTES.iter() {
        formatted_artist = regex.replace_all(&formatted_artist, *replacement).to_string();
        formatted_title = regex.replace_all(&formatted_title, *replacement).to_string();
    }

    (formatted_artist.trim().to_string(), formatted_title.trim().to_string())
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

fn move_feat_from_title_to_artist(artist: &mut String, title: &mut String) {
    if title.contains(" feat. ") || title.contains("(feat. ") {
        if let Some(feat_match) = FEAT_REGEX.find(&title.clone()) {
            let feat = feat_match
                .as_str()
                .trim_end_matches(|c| c == '(' || c == ')' || c == '-');

            // Remove the feat from the title
            *title = title
                .replace(feat, "")
                .replace("()", "")
                .replace("  ", " ")
                .trim()
                .to_string();

            // Format feat string
            let feat = feat
                .replacen("feat. ", "", 1)
                .replace(", and ", " & ")
                .replace(" and ", " & ")
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
    // Special case to skip one valid title
    if text.ends_with(" (4U)") {
        return;
    }

    let mut result = text.to_string();
    result = BPM_IN_PARENTHESES.replace_all(&result, "").to_string();
    result = BPM_WITH_KEY.replace_all(&result, "").to_string();
    if !result.to_lowercase().ends_with(" mix)") {
        result = BPM_WITH_LETTERS.replace_all(&result, "").to_string();
    }

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

    let mut result = TEXT_AFTER_PARENTHESES
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
    *text = DASH_IN_PARENTHESES
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
