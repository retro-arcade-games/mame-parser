use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

// Regular expressions used for cleaning and normalizing manufacturer names.
lazy_static! {
    static ref RE_COMMON: Regex = Regex::new(r"(?i)\b(Games|Corp|Inc|Ltd|Co|Corporation|Industries|Elc|S\.R\.L|S\.A|inc|of America|Japan|UK|USA|Europe|do Brasil|du Canada|Canada|America|Austria|of)\b\.?").unwrap();
    static ref RE_PUNCTUATION: Regex = Regex::new(r"[.,?]+$|-$").unwrap();
    static ref NEEDS_CLEANING: Regex = Regex::new(r"[\(/,?]|(Games|Corp|Inc|Ltd|Co|Corporation|Industries|Elc|S\.R\.L|S\.A|inc|of America|Japan|UK|USA|Europe|do Brasil|du Canada|Canada|America|Austria|of)").unwrap();
}

/// Substitutions for normalizing the number of players description.
const SUBSTITUTIONS_ARRAY: &[(&str, &str)] = &[
    ("1P", "Single-player game"),
    ("2P alt", "Alternate two-player mode"),
    ("2P sim", "Simultaneous two-player mode"),
    ("3P alt", "Alternate three-player mode"),
    ("3P sim", "Simultaneous three-player mode"),
    ("4P alt", "Alternate four-player mode"),
    ("4P sim", "Simultaneous four-player mode"),
    ("5P alt", "Alternate five-player mode"),
    ("6P alt", "Alternate six-player mode"),
    ("6P sim", "Simultaneous six-player mode"),
    ("8P alt", "Alternate eight-player mode"),
    ("8P sim", "Simultaneous eight-player mode"),
    ("9P alt", "Alternate nine-player mode"),
    ("???", "Unknown or unspecified number of players"),
    ("BIOS", "BIOS"),
    ("Device", "Non-playable device"),
    ("Non-arcade", "Non-arcade game"),
];

/// Normalizes a machine's name based on its description.
///
/// This function takes an optional description of a machine and returns a normalized version of the name.
/// It performs several transformations, including removing special characters, handling HTML entities,
/// and capitalizing the first letter of each word while preserving whitespace.
///
/// # Parameters
/// - `description`: An `Option<String>` that contains the description of the machine to be normalized.
///   If `None`, the function returns an empty string.
///
/// # Returns
/// Returns a `String` representing the normalized name:
/// - If `description` is `Some`, the function processes the string by removing certain characters and adjusting capitalization.
/// - If `description` is `None`, the function returns an empty string.
///
/// # Processing Steps
/// - Replaces specific characters (`'?'` and `"&amp;"`) with their desired substitutes (empty string and `"&"`, respectively).
/// - Extracts the portion of the description before the first occurrence of `'('` to remove any additional information.
/// - Capitalizes the first letter of each word while preserving whitespace and maintains the rest of the characters as they are.
///
/// # Errors
/// This function does not return errors. It always returns a `String`, either processed or empty.
pub(crate) fn normalize_machine_name(description: &Option<String>) -> String {
    if description.is_none() {
        return String::new();
    }

    let step1 = description
        .as_ref()
        .unwrap()
        .replace('?', "")
        .replace("&amp;", "&");
    let step2: String = step1.split('(').next().unwrap_or("").to_string();

    let mut result = String::new();
    let mut capitalize_next = true;
    for c in step2.chars() {
        if c.is_whitespace() {
            capitalize_next = true;
            result.push(c);
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Normalizes a manufacturer's name by cleaning and formatting it.
///
/// This function processes an optional manufacturer's name, removing unwanted characters, words,
/// and formatting it to ensure a standardized representation. It removes common suffixes like "Inc",
/// "Ltd", and others, handles punctuation, and corrects special cases to provide a clean and consistent output.
///
/// # Parameters
/// - `manufacturer`: An `Option<String>` that contains the manufacturer's name to be normalized.
///   If `None`, the function will return an empty string.
///
/// # Returns
/// Returns a `String` representing the normalized manufacturer name:
/// - If `manufacturer` is `Some`, the function processes the string by removing unwanted parts and cleaning the name.
/// - If `manufacturer` is `None`, the function returns an empty string.
///
/// # Processing Steps
/// - Splits the manufacturer name at specific characters like `(` or `/` to keep only the first part.
/// - Handles edge cases where the first part might be empty by using the second part if available.
/// - Cleans the name using regular expressions (`Regex`) to remove common terms (e.g., "Inc", "Corp") and punctuation.
/// - Replaces specific unwanted characters (`?`, `,`) and adjusts certain terms (`"<unknown>"` to `"Unknown"`).
/// - Trims any leading or trailing whitespace to produce the final result.
///
/// # Regular Expressions
/// The function utilizes the following pre-compiled regular expressions for efficiency:
/// - `RE_COMMON`: Matches common suffixes and terms often found in manufacturer names.
/// - `RE_PUNCTUATION`: Matches trailing punctuation that should be removed.
/// - `NEEDS_CLEANING`: Matches terms or patterns that indicate the name requires cleaning.
pub(crate) fn normalize_manufacturer_name(manufacturer: &Option<String>) -> String {
    // Keep only the first part of the manufacturer removing anything after (, /
    let parts: Vec<&str> = manufacturer
        .as_ref()
        .unwrap()
        .trim()
        .split(&['(', '/'][..])
        .collect();
    let mut result = parts[0].to_string();

    // Fix for edge case where the first part is empty
    if result.is_empty() && parts.len() > 1 {
        result = parts[1].to_string();
    }

    // Check if needs cleaning
    if NEEDS_CLEANING.is_match(&result) {
        result = RE_COMMON.replace_all(&result, "").to_string();
        result = RE_PUNCTUATION.replace_all(&result, "").to_string();
    }

    result = result.replace('?', "").replace(',', "");
    result = result.replace("<unknown>", "Unknown");
    result = result.trim().to_string();

    result
}

/// Normalizes a software list name by cleaning and formatting it.
fn get_substitutions() -> HashMap<&'static str, &'static str> {
    SUBSTITUTIONS_ARRAY.iter().cloned().collect()
}

/// Normalizes the number of players description based on predefined substitutions.
///
/// This function takes an optional string that describes the number of players for a game or machine
/// and returns a normalized version of the description. It replaces specific terms with more readable
/// or standardized equivalents using a predefined set of substitutions.
///
/// # Parameters
/// - `nplayers`: An `Option<String>` that contains the description of the number of players to be normalized.
///   If `None`, the function returns "Unknown".
///
/// # Returns
/// Returns a `String` representing the normalized description of the number of players:
/// - If `nplayers` is `Some`, the function processes the string by substituting recognized terms with standardized equivalents.
/// - If `nplayers` is `None`, the function returns "Unknown".
///
/// # Processing Steps
/// - Retrieves a map of substitutions from the `get_substitutions` function.
/// - Splits the input string on '/' to handle multiple descriptions.
/// - For each part, trims whitespace and replaces it using the substitution map.
/// - Joins all parts back together with a comma separator to form the final normalized description.
///
/// # Substitution Mapping
/// The function uses a set of predefined substitutions defined in `SUBSTITUTIONS_ARRAY` to map specific terms to their standardized descriptions.
pub(crate) fn normalize_nplayer_name(nplayers: &Option<String>) -> String {
    let substitutions = get_substitutions();
    nplayers
        .as_ref()
        .unwrap_or(&"Unknown".to_string())
        .split('/')
        .map(|part| {
            let part = part.trim();
            substitutions.get(part).unwrap_or(&part).to_string()
        })
        .collect::<Vec<_>>()
        .join(", ")
}
