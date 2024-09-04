use std::collections::HashMap;

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
pub fn normalize_nplayer(nplayers: &Option<String>) -> String {
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
