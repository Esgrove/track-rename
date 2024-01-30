use regex::Regex;

pub struct TrackFormatter {
    common_substitutes: Vec<(&'static str, &'static str)>,
    title_substitutes: Vec<(&'static str, &'static str)>,
    regex_substitutes: Vec<(Regex, &'static str)>,
    filename_regex_substitutes: Vec<(Regex, &'static str)>,
}

impl TrackFormatter {
    pub fn new() -> TrackFormatter {
        TrackFormatter {
            common_substitutes: vec![
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
            ],
            title_substitutes: vec![
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
            ],
            regex_substitutes: vec![
                (Regex::new(r"(?i)\b(?:feat\.?|ft\.?|featuring)\b").unwrap(), "feat."),
                (Regex::new(r"(?i)\(\s*(?:feat\.?|ft\.?|featuring)\b").unwrap(), "(feat."),
                (Regex::new(r"[\[{]+").unwrap(), "("),
                (Regex::new(r"[\]}]+").unwrap(), ")"),
                (Regex::new(r"!{2,}").unwrap(), "!"),
                (Regex::new(r"\s+").unwrap(), " "),
                (Regex::new(r"\s{2,}").unwrap(), " "),
                (Regex::new(r"\.{2,}").unwrap(), "."),
                (Regex::new(r"\(\s*?\)").unwrap(), ""),
                (Regex::new(r"(\S)\(").unwrap(), "$1 ("),
                (Regex::new(r"\(\s*\){2,}").unwrap(), "("),
                (Regex::new(r"\)\s*\){2,}").unwrap(), ")"),
                (
                    Regex::new(r"(?i)\bMissy Elliot\b|\bMissy Elliot$").unwrap(),
                    "Missy Elliott",
                ),
                (Regex::new(r"(?i)\bGangstarr\b|\bGangstarr$").unwrap(), "Gang Starr"),
                (Regex::new(r"(?i)\sW/").unwrap(), " feat. "),
            ],
            filename_regex_substitutes: vec![
                (Regex::new("\"").unwrap(), "''"),
                (Regex::new(r"([\\/<>|:\*\?])+").unwrap(), "-"),
                (Regex::new(r"\s+").unwrap(), " "),
            ],
        }
    }

    /// Return formatted artist and title string.
    pub(crate) fn format_tags(&self, artist: &str, title: &str) -> (String, String) {
        let mut formatted_artist = artist.to_string();
        let mut formatted_title = title.to_string();

        let extensions = [".mp3", ".flac", ".aif", ".aiff", ".m4a"];
        for ext in &extensions {
            if formatted_artist.to_lowercase().ends_with(ext) {
                formatted_artist = formatted_artist[0..formatted_artist.len() - ext.len()].to_string();
            }
            if formatted_title.to_lowercase().ends_with(ext) {
                formatted_title = formatted_title[0..formatted_title.len() - ext.len()].to_string();
            }
        }

        for (pattern, replacement) in &self.common_substitutes {
            formatted_artist = formatted_artist.replace(pattern, replacement);
            formatted_title = formatted_title.replace(pattern, replacement);
        }

        for (pattern, replacement) in &self.title_substitutes {
            formatted_title = formatted_title.replace(pattern, replacement);
        }

        for (regex, replacement) in &self.regex_substitutes {
            formatted_artist = regex.replace_all(&formatted_artist, *replacement).to_string();
            formatted_title = regex.replace_all(&formatted_title, *replacement).to_string();
        }

        TrackFormatter::use_parenthesis_for_mix(&mut formatted_title);

        (formatted_artist, formatted_title) =
            TrackFormatter::move_feat_from_title_to_artist(&formatted_artist, &formatted_title);

        formatted_title = TrackFormatter::replace_dash_in_parentheses(&formatted_title);
        formatted_title = TrackFormatter::fix_nested_parentheses(&formatted_title);
        formatted_title = TrackFormatter::wrap_text_after_parentheses(&formatted_title);
        formatted_title = TrackFormatter::remove_bpm_in_parentheses_from_end(&formatted_title);
        formatted_title = formatted_title.replace("((", "(").replace("))", ")");

        for (regex, replacement) in &self.regex_substitutes {
            formatted_artist = regex.replace_all(&formatted_artist, *replacement).to_string();
            formatted_title = regex.replace_all(&formatted_title, *replacement).to_string();
        }

        for (pattern, replacement) in &self.common_substitutes {
            formatted_artist = formatted_artist.replace(pattern, replacement);
            formatted_title = formatted_title.replace(pattern, replacement);
        }

        (formatted_artist.trim().to_string(), formatted_title.trim().to_string())
    }

    pub(crate) fn format_filename(&self, artist: &str, title: &str) -> (String, String) {
        let mut formatted_artist = artist.to_string();
        let mut formatted_title = title.to_string();

        for (regex, replacement) in &self.filename_regex_substitutes {
            formatted_artist = regex.replace_all(&formatted_artist, *replacement).to_string();
            formatted_title = regex.replace_all(&formatted_title, *replacement).to_string();
        }

        (formatted_artist.trim().to_string(), formatted_title.trim().to_string())
    }

    fn move_feat_from_title_to_artist(artist: &str, title: &str) -> (String, String) {
        let mut formatted_artist = artist.to_string();
        let mut formatted_title = title.to_string();

        if title.contains(" feat. ") || title.contains("(feat. ") {
            let feat_regex = Regex::new(r"feat\. .*?( -|\(|\)|$)").unwrap();
            if let Some(feat_match) = feat_regex.find(&title) {
                let feat = feat_match
                    .as_str()
                    .trim_end_matches(|c| c == '(' || c == ')' || c == '-');

                formatted_title = title.replace(feat, "").replace("()", "");

                let feat_artist = feat
                    .replacen("feat. ", "", 1)
                    .replace(", and ", " & ")
                    .replace(" and ", " & ")
                    .trim()
                    .to_string();

                for delimiter in [", ", " & ", " and ", " + "] {
                    formatted_artist = formatted_artist
                        .replace(&format!("{}{}", delimiter, feat_artist), "")
                        .replace(&format!("{}{}", feat_artist, delimiter), "");
                }

                let new_feat = format!(" feat. {}", feat_artist);
                if !formatted_artist.contains(&new_feat) {
                    formatted_artist.push_str(&new_feat);
                }
            }
        }

        (formatted_artist.trim().to_string(), formatted_title.trim().to_string())
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
                    title.insert_str(insert_index, ")");
                } else {
                    // Add a closing parenthesis at the end
                    title.push(')');
                }
            }
        }
    }

    fn fix_nested_parentheses(text: &str) -> String {
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
                    if let Some(_) = stack.pop() {
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
        while let Some(_) = stack.pop() {
            result.push(')');
        }

        result
            .replace(" )", ")")
            .replace("( ", "(")
            .replace(" ()", "")
            .replace("() ", "")
    }

    fn remove_bpm_in_parentheses_from_end(text: &str) -> String {
        // Special case to skip one valid title
        if text.ends_with(" (4U)") {
            return text.to_string();
        }

        let mut result = text.to_string();

        let re = Regex::new(r" \((\d{2,3}(\.\d)?|\d{2,3} \d{1,2}a)\)$").unwrap();
        result = re.replace_all(&result, "").to_string();

        let re = Regex::new(r"\s\(\d{1,2}(?:\s\d{1,2})?\s?[a-zA-Z]\)$").unwrap();
        result = re.replace_all(&result, "").to_string();

        if !result.to_lowercase().ends_with(" mix)") {
            let re = Regex::new(r"\s\(\d{2,3}\s?[a-zA-Z]{2,3}\)$").unwrap();
            result = re.replace_all(&result, "").to_string();
        }

        result
    }

    fn wrap_text_after_parentheses(text: &str) -> String {
        let re = Regex::new(r"\)\s(.*?)\s\(").unwrap();

        let (start, rest) = if text.starts_with('(') {
            // If the text starts with a parenthesis, find the end of the first group and start replacing from there
            if let Some(index) = text.find(") ") {
                text.split_at(index + 2)
            } else {
                return text.to_string();
            }
        } else {
            ("", text)
        };

        let mut result = re
            .replace_all(rest, |caps: &regex::Captures| format!(") ({}) (", &caps[1]))
            .to_string();

        if let Some(index) = result.rfind(')') {
            if index < result.len() - 1 {
                result.insert_str(index + 2, "(");
                result.push_str(")");
            }
        }

        format!("{}{}", start, result)
    }

    fn replace_dash_in_parentheses(text: &str) -> String {
        let re = Regex::new(r"\((.*?) - (.*?)\)").unwrap();
        let result = re.replace_all(text, |caps: &regex::Captures| format!("({}) ({})", &caps[1], &caps[2]));
        result.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_parenthesis_for_mix() {
        let mut title = "Azn Danza - Myles Club Edit".to_string();
        let correct_title = "Azn Danza (Myles Club Edit)".to_string();
        TrackFormatter::use_parenthesis_for_mix(&mut title);
        assert_eq!(title, correct_title);

        let mut title = "About Damn Time - Purple Disco Machine (Dirty Intro)".to_string();
        let correct_title = "About Damn Time (Purple Disco Machine) (Dirty Intro)".to_string();
        TrackFormatter::use_parenthesis_for_mix(&mut title);
        assert_eq!(title, correct_title);
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
            let result = TrackFormatter::remove_bpm_in_parentheses_from_end(input);
            assert_eq!(result, expected);
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
            let result = TrackFormatter::fix_nested_parentheses(input);
            assert_eq!(result, expected);
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
            let result = TrackFormatter::wrap_text_after_parentheses(input);
            assert_eq!(result, expected);
        }
    }
}
