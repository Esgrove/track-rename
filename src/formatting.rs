use std::cmp::Ordering;
use std::sync::LazyLock;

use regex::{Captures, Regex};

static COMMON_SUBSTITUTES: [(&str, &str); 23] = [
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
static TITLE_SUBSTITUTES: [(&str, &str); 18] = [
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
    ("(Clean-Beat Junkie Sound ", "(Clean Beat Junkie Sound "),
    ("(Dirty-Beat Junkie Sound ", "(Dirty Beat Junkie Sound "),
    (" Version/cyberkid ", " Version - Cyberkid "),
    ("/Beat Junkie ", " - Beat Junkie "),
    ("(Clean-", "(Clean "),
    ("(Dirty-", "(Dirty "),
];
static REGEX_SUBSTITUTES: LazyLock<[(Regex, &'static str); 12]> = LazyLock::new(|| {
    [
        // Replace various opening bracket types with "("
        (
            Regex::new(r"[\[{]+").expect("Failed to compile opening brackets regex"),
            "(",
        ),
        // Replace various closing bracket types with ")"
        (
            Regex::new(r"[]}]+").expect("Failed to compile closing brackets regex"),
            ")",
        ),
        // Collapse multiple exclamation marks into one
        (
            Regex::new(r"!{2,}").expect("Failed to compile exclamation marks regex"),
            "!",
        ),
        // Collapse multiple periods into a single period
        (Regex::new(r"\.{2,}").expect("Failed to compile periods regex"), "."),
        // Remove empty parentheses
        (
            Regex::new(r"\(\s*?\)").expect("Failed to compile empty parentheses regex"),
            "",
        ),
        // Ensure a space before an opening parenthesis
        (
            Regex::new(r"(\S)\(").expect("Failed to compile space before parenthesis regex"),
            "$1 (",
        ),
        // Ensure a space after a closing parenthesis
        (
            Regex::new(r"\)([A-Za-z0-9])").expect("Failed to compile space after parenthesis regex"),
            ") $1",
        ),
        // Collapse multiple consecutive opening parentheses into one
        (
            Regex::new(r"\(\s*\){2,}").expect("Failed to compile consecutive opening parentheses regex"),
            "(",
        ),
        // Collapse multiple consecutive closing parentheses into one
        (
            Regex::new(r"\)\s*\){2,}").expect("Failed to compile consecutive closing parentheses regex"),
            ")",
        ),
        // Transforms underscore-wrapped text into single-quoted text
        (
            Regex::new(r"\s_(.*?)_\s").expect("Failed to compile underscore text regex"),
            " '$1' ",
        ),
        // Remove asterisks after a word boundary
        (Regex::new(r"\s\*+\b").expect("Failed to compile asterisks regex"), ""),
        // Collapses multiple spaces into a single space
        (
            Regex::new(r"\s+").expect("Failed to compile multiple spaces regex"),
            " ",
        ),
    ]
});
static REGEX_NAME_SUBSTITUTES: LazyLock<[(Regex, &'static str); 43]> = LazyLock::new(|| {
    [
        // Matches "12 Inch" or "12Inch" with optional space, case-insensitive
        (
            Regex::new(r"(?i)\b12\s?inch\b").expect("Failed to compile 12 inch regex"),
            "12''",
        ),
        // Matches "12in" or "12 in" with optional space, case-insensitive
        (
            Regex::new(r"(?i)\b12\s?in\b").expect("Failed to compile 12 in regex"),
            "12''",
        ),
        // Matches "7 Inch" or "7Inch" with optional space, case-insensitive
        (
            Regex::new(r"(?i)\b7\s?inch\b").expect("Failed to compile 7 inch regex"),
            "7''",
        ),
        // Matches "7in" or "7 in" with optional space, case-insensitive
        (
            Regex::new(r"(?i)\b7\s?in\b").expect("Failed to compile 7 in regex"),
            "7''",
        ),
        // Standardize various forms of "featuring" to "feat."
        (
            Regex::new(r"(?i)\b(?:feat\.?|ft\.?|featuring)\b").expect("Failed to compile featuring regex"),
            "feat.",
        ),
        (
            Regex::new(r"(?i)\(\s*(?:feat\.?|ft\.?|featuring)\b")
                .expect("Failed to compile parentheses featuring regex"),
            "(feat.",
        ),
        // Standardize "w/" to "feat."
        (Regex::new(r"(?i)\sW/").expect("Failed to compile w/ regex"), " feat. "),
        // Standardize Remix
        (
            Regex::new(r"(?i)\(Rmx\)").expect("Failed to compile rmx parentheses regex"),
            "(Remix)",
        ),
        (
            Regex::new(r"(?i)\bRmx\b").expect("Failed to compile rmx regex"),
            "Remix",
        ),
        // Remove trademark symbols
        (Regex::new(r"[®™]").expect("Failed to compile trademark regex"), ""),
        // Correct name for "Missy Elliott"
        (
            Regex::new(r"(?i)\bMissy Elliot\b|\bMissy Elliot$").expect("Failed to compile Missy Elliott regex"),
            "Missy Elliott",
        ),
        // Correct name for "Gang Starr"
        (
            Regex::new(r"(?i)\bGangstarr\b|\bGangstarr$").expect("Failed to compile Gang Starr regex"),
            "Gang Starr",
        ),
        // Fix capitalization for SZA
        (Regex::new(r"(?i)\bSza\b").expect("Failed to compile SZA regex"), "SZA"),
        // Fix spelling for "You're"
        (
            Regex::new(r"(?i)\bYoure\b").expect("Failed to compile You're regex"),
            "You're",
        ),
        // Fix spelling for "I'm"
        (Regex::new(r"(?i)\bIm\b").expect("Failed to compile I'm regex"), "I'm"),
        // Fix spelling for "You've"
        (
            Regex::new(r"(?i)\bYouve\b").expect("Failed to compile You've regex"),
            "You've",
        ),
        // Fix spelling for "Can't"
        (
            Regex::new(r"(?i)\bCant\b").expect("Failed to compile Can't regex"),
            "Can't",
        ),
        // Fix spelling for "Won't"
        (
            Regex::new(r"(?i)\bWont\b").expect("Failed to compile Won't regex"),
            "Won't",
        ),
        // Fix spelling for "Don't"
        (
            Regex::new(r"(?i)\bDont\b").expect("Failed to compile Don't regex"),
            "Don't",
        ),
        // Fix capitalization for "DJ"
        (Regex::new(r"(?i)\bDj\b").expect("Failed to compile DJ regex"), "DJ"),
        // Ensure one whitespace after "feat."
        (
            Regex::new(r"\bfeat\.([A-Za-z0-9])").expect("Failed to compile feat space regex"),
            "feat. $1",
        ),
        (
            Regex::new(r"(?i)\b(dirty!)\b").expect("Failed to compile dirty regex"),
            "(Dirty)",
        ),
        // Removes "Original Mix" with case-insensitivity
        (
            Regex::new(r"(?i)\(Original Mix\)").expect("Failed to compile original mix regex"),
            "",
        ),
        // Removes "DJCity" with case-insensitivity
        (
            Regex::new(r"(?i)\bdjcity\b").expect("Failed to compile djcity regex"),
            "",
        ),
        (
            Regex::new(r"(?i)\bintro - clean\b").expect("Failed to compile intro clean regex"),
            "Clean Intro",
        ),
        (
            Regex::new(r"(?i)\bintro - dirty\b").expect("Failed to compile intro dirty regex"),
            "Dirty Intro",
        ),
        (
            Regex::new(r"(?i)\(clean - intro\)").expect("Failed to compile clean intro parentheses regex"),
            "(Clean Intro)",
        ),
        (
            Regex::new(r"(?i)\(dirty - intro\)").expect("Failed to compile dirty intro parentheses regex"),
            "(Dirty Intro)",
        ),
        (
            Regex::new(r"(?i)\bIntro[:\s/+\-&]*outro\b").expect("Failed to compile intro outro regex"),
            "Intro",
        ),
        (
            Regex::new(r"(?i)\bAca In\b").expect("Failed to compile aca in regex"),
            "Acapella Intro",
        ),
        (
            Regex::new(r"(?i)\bAca intro[:\s/+\-&]*aca outro\b").expect("Failed to compile aca intro outro regex"),
            "Acapella In-Out",
        ),
        (
            Regex::new(r"(?i)\bAcapella Intro[:\s/+\-&]*aca out\b")
                .expect("Failed to compile acapella intro out regex"),
            "Acapella In-Out",
        ),
        (
            Regex::new(r"(?i)\bAca Out\b").expect("Failed to compile aca out regex"),
            "Acapella Out",
        ),
        (
            Regex::new(r"(?i)\bAcap-In\b").expect("Failed to compile acap in regex"),
            "Acapella Intro",
        ),
        (
            Regex::new(r"(?i)\bAcap - diy\b").expect("Failed to compile acap diy regex"),
            "Acapella DIY",
        ),
        (
            Regex::new(r"(?i)\bAcap in[:\s/+\-&]*out\b").expect("Failed to compile acap in out regex"),
            "Acapella In-Out",
        ),
        (
            Regex::new(r"(?i)\bAcap\b").expect("Failed to compile acap regex"),
            "Acapella",
        ),
        (
            Regex::new(r"(?i)\bAcapella[\s/+\-]*In[:\s/+\-&]*Out\b").expect("Failed to compile acapella in out regex"),
            "Acapella In-Out",
        ),
        (
            Regex::new(r"(?i)\bAcapella[\s/+\-]*In\b").expect("Failed to compile acapella in regex"),
            "Acapella Intro",
        ),
        (
            Regex::new(r"(?i)\bAcapella Intro[:\s/+\-&]*Out\b").expect("Failed to compile acapella intro out regex"),
            "Acapella In-Out",
        ),
        (
            Regex::new(r"(?i)\bAcapella-Intro[:\s/+\-&]*Out\b")
                .expect("Failed to compile acapella intro out dash regex"),
            "Acapella In-Out",
        ),
        (
            Regex::new(r"(?i)\bAcapella-Intro\b").expect("Failed to compile acapella intro dash regex"),
            "Acapella Intro",
        ),
        (
            Regex::new(r"(?i)\bAcapella-out\b").expect("Failed to compile acapella out dash regex"),
            "Acapella Out",
        ),
    ]
});
static REGEX_FILENAME_SUBSTITUTES: LazyLock<[(Regex, &str); 2]> = LazyLock::new(|| {
    [
        // Replace characters that are not allowed in filenames with a hyphen
        (
            Regex::new(r"([\\/<>|:*?])").expect("Failed to compile filename chars regex"),
            "-",
        ),
        // Collapse multiple spaces into a single space
        (
            Regex::new(r"\s+").expect("Failed to compile filename spaces regex"),
            " ",
        ),
    ]
});
// Matches "feat." followed by any text until a dash, parenthesis, or end of string
static RE_FEAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bfeat\. .*?( -|\(|\)|$)").expect("Failed to compile feat regex"));

// Matches text after a closing parenthesis until the next opening parenthesis
static RE_TEXT_AFTER_PARENTHESES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\)\s(.*?)\s\(").expect("Failed to compile text after parentheses regex"));

// Matches BPM information inside parentheses at the end of a string,
// with BPM in range 50–180 and an optional decimal.
static RE_BPM_IN_PARENTHESES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r" \(((?:5[0-9]|6[0-9]|7[0-9]|8[0-9]|9[0-9]|1[0-7][0-9]|180)(?:\.\d)?)\)$")
        .expect("Failed to compile BPM in parentheses regex")
});

// Matches BPM with an optional key, formatted within parentheses at the end of a string
static RE_BPM_WITH_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s\(((?:5[0-9]|6[0-9]|7[0-9]|8[0-9]|9[0-9]|1[0-6][0-9]|17[0-9])(?:\s(1[0-2]|[1-9]))?[a-zA-Z])\)$")
        .expect("Failed to compile BPM with key regex")
});

// Matches BPM followed by two or three letters (likely denoting key or mode),
// formatted within parentheses at the end of a string
static RE_BPM_WITH_TEXT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:5[0-9]|6[0-9]|7[0-9]|8[0-9]|9[0-9]|1[0-6][0-9]|17[0-9])bpm\b")
        .expect("Failed to compile BPM with text regex")
});
static RE_BPM_WITH_TEXT_PARENTHESES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s\((?:5[0-9]|6[0-9]|7[0-9]|8[0-9]|9[0-9]|1[0-6][0-9]|17[0-9])\s?[a-zA-Z]{2,3}\)$")
        .expect("Failed to compile BPM with text parentheses regex")
});
static RE_BPM_WITH_EXTRA_TEXT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:5[0-9]|6[0-9]|7[0-9]|8[0-9]|9[0-9]|1[0-6][0-9]|17[0-9])\s?[a-zA-Z]{2,3}$")
        .expect("Failed to compile BPM with extra text regex")
});

// Matches any text within parentheses that contains a dash, separating it into two groups
static RE_DASH_IN_PARENTHESES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\((.*?) - (.*?)\)").expect("Failed to compile dash in parentheses regex"));

// Matches variations on "and" in feat artist names
static RE_FEAT_AND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i),?\s+and\s+").expect("Failed to compile feat and regex"));

// Collapse multiple spaces into a single space
static RE_MULTIPLE_SPACES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s{2,}").expect("Failed to compile multiple spaces regex"));

static RE_WWW: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^www\.").expect("Failed to compile www regex"));

static RE_CHARS_AND_DOTS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^([a-z]\.)+([a-z])?$").expect("Failed to compile chars and dots regex"));
const FILE_EXTENSIONS: [&str; 5] = [".mp3", ".flac", ".aif", ".aiff", ".m4a"];

/// Return formatted artist and title string.
pub fn format_tags_for_artist_and_title(artist: &str, title: &str) -> (String, String) {
    let mut formatted_artist = artist.to_string();
    let mut formatted_title = title.to_string();

    // Remove an extra file extension from the end

    for ext in &FILE_EXTENSIONS {
        if formatted_artist.to_lowercase().ends_with(ext) {
            formatted_artist = formatted_artist[0..formatted_artist.len() - ext.len()].to_string();
        }
        if formatted_title.to_lowercase().ends_with(ext) {
            formatted_title = formatted_title[0..formatted_title.len() - ext.len()].to_string();
        }
    }

    for (pattern, replacement) in &COMMON_SUBSTITUTES {
        formatted_artist = formatted_artist.replace(pattern, replacement);
        formatted_title = formatted_title.replace(pattern, replacement);
    }

    for (pattern, replacement) in &TITLE_SUBSTITUTES {
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

    formatted_artist = formatted_artist.replace(" / ", ", ");
    if formatted_artist.eq_ignore_ascii_case("Various Artists") {
        let (artist, title) = match formatted_title.splitn(2, " - ").collect::<Vec<&str>>().as_slice() {
            [artist, title] => (*artist, *title),
            [no_split] => ("", *no_split),
            _ => ("", ""),
        };
        formatted_artist = artist.to_string();
        formatted_title = title.to_string();
    } else {
        formatted_artist = formatted_artist.trim_start_matches("Various Artists - ").to_string();
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

    for (pattern, replacement) in &COMMON_SUBSTITUTES {
        formatted_artist = formatted_artist.replace(pattern, replacement);
        formatted_title = formatted_title.replace(pattern, replacement);
    }

    if formatted_title == formatted_title.to_uppercase()
        && formatted_title.chars().count() > 10
        && !RE_CHARS_AND_DOTS.is_match(&formatted_title)
    {
        formatted_title = titlecase::titlecase(&formatted_title);
        if formatted_artist == formatted_artist.to_uppercase() && formatted_artist.chars().count() > 8 {
            formatted_artist = titlecase::titlecase(&formatted_artist);
        }
    } else if RE_CHARS_AND_DOTS.is_match(&formatted_title) {
        formatted_title = formatted_title.to_uppercase();
    }

    (formatted_artist.trim().to_string(), formatted_title.trim().to_string())
}

/// Apply filename formatting.
pub fn format_filename(artist: &str, title: &str) -> (String, String) {
    // Replace double quotes with two single quotes
    let mut formatted_artist = artist.replace('"', "''");
    let mut formatted_title = title.replace('"', "''");

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
        Ordering::Equal => {}
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
        let feat = feat_match.as_str().trim_end_matches(['(', ')', '-']);

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
            .map(ToString::to_string)
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
    if title.contains(" - ")
        && let Some(mut index) = title.find(" - ")
    {
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

fn fix_nested_parentheses(text: &mut String) {
    // Initialize a stack to keep track of parentheses
    let mut stack = Vec::new();
    let mut result = String::new();

    for char in text.chars() {
        match char {
            '(' => {
                // If the stack is not empty and the top element is also '(', add a closing ')' before the new '('
                if let Some(&last_char) = stack.last()
                    && last_char == '('
                {
                    result.push_str(") ");
                }
                stack.push(char);
                result.push(char);
            }
            ')' => {
                // If the stack is not empty, pop an element from the stack
                if stack.pop().is_some() {
                    // Add the closing parenthesis only if the stack is empty or the top element is not '('
                    if stack.is_empty() || stack.last().is_none_or(|&c| c != '(') {
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
    if let Some(start) = artist.find(start_pattern)
        && let Some(end) = artist[start..].find(')')
    {
        let feature_part = &artist[start..=(start + end)];
        *artist = artist.replacen(feature_part, &feature_part[1..feature_part.len() - 1], 1);
    }
}

fn remove_bpm_in_parentheses_from_end(text: &mut String) {
    // Skip some valid titles
    let suffixes = [" (4u)", "33rpm)", "45rpm)", " mix)", " dub)", " eq)", " rip)"];
    let text_lower = text.to_lowercase();
    if suffixes.iter().any(|suffix| text_lower.ends_with(suffix)) {
        return;
    }

    let mut result = (*text).clone();
    let regexes = [
        &RE_BPM_IN_PARENTHESES,
        &RE_BPM_WITH_TEXT,
        &RE_BPM_WITH_KEY,
        &RE_BPM_WITH_TEXT_PARENTHESES,
        &RE_BPM_WITH_EXTRA_TEXT,
    ];
    for re in regexes {
        if re.is_match(&result) {
            result = re.replace_all(&result, "").to_string();
            break;
        }
    }
    result = result.trim().to_string();
    if !result.is_empty() {
        *text = result;
    }
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

    if let Some(index) = result.rfind(')')
        && index < result.len() - 1
    {
        result.insert(index + 2, '(');
        result.push(')');
    }

    *text = format!("{start}{result}");
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

#[cfg(test)]
mod bpm_tests {
    use super::RE_BPM_WITH_KEY;

    #[test]
    fn valid_bpm_with_key() {
        let valid_cases = [" (50 1A)", "Song (99 12b)", "Something (120 5A)", "Test (100 11a)"];
        for case in valid_cases {
            assert!(RE_BPM_WITH_KEY.is_match(case), "Should match: {case}");
        }
    }

    #[test]
    fn invalid_bpm_with_key() {
        let invalid_cases = [
            " (1 A)",      // below 50
            " (200 A)",    // above 179
            " (100 5 A)",  // extra number
            " (abc G)",    // not a number
            " (60)",       // missing letters
            " (60  )",     // only number and space
            " (100 ABCD)", // too many letters
            " (100)",      // no key
        ];
        for case in invalid_cases {
            assert!(!RE_BPM_WITH_KEY.is_match(case), "Should not match: {case}");
        }
    }
}
