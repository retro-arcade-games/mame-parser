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
pub fn normalize_name(description: &Option<String>) -> String {
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
