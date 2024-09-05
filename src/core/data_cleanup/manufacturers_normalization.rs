use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_COMMON: Regex = Regex::new(r"(?i)\b(Games|Corp|Inc|Ltd|Co|Corporation|Industries|Elc|S\.R\.L|S\.A|inc|of America|Japan|UK|USA|Europe|do Brasil|du Canada|Canada|America|Austria|of)\b\.?").unwrap();
    static ref RE_PUNCTUATION: Regex = Regex::new(r"[.,?]+$|-$").unwrap();
    static ref NEEDS_CLEANING: Regex = Regex::new(r"[\(/,?]|(Games|Corp|Inc|Ltd|Co|Corporation|Industries|Elc|S\.R\.L|S\.A|inc|of America|Japan|UK|USA|Europe|do Brasil|du Canada|Canada|America|Austria|of)").unwrap();
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

pub(crate) fn normalize_manufacturer(manufacturer: &Option<String>) -> String {
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
