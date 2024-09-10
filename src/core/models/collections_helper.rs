use crate::models::Machine;
use std::collections::HashMap;

/// Gets a list of unique manufacturers from the provided machines, counting their occurrences.
///
/// # Parameters
/// - `machines`: A reference to a `HashMap<String, Machine>` representing the collection of machines.
///
/// # Returns
/// A `HashMap<String, usize>` where keys are manufacturer names and values are their counts.
pub fn get_manufacturers_list(machines: &HashMap<String, Machine>) -> HashMap<String, usize> {
    let mut manufacturers: HashMap<String, usize> = HashMap::new();

    machines
        .values()
        .filter_map(|machine| machine.extended_data.as_ref()?.manufacturer.as_ref()) // Filters and maps to manufacturers
        .for_each(|manufacturer| add_item_to_list(&mut manufacturers, manufacturer.clone())); // Adds each manufacturer to the list

    manufacturers
}

/// Gets a list of unique languages from the provided machines, counting their occurrences.
///
/// # Parameters
/// - `machines`: A reference to a `HashMap<String, Machine>` representing the collection of machines.
///
/// # Returns
/// A `HashMap<String, usize>` where keys are language names and values are their counts.
pub fn get_languages_list(machines: &HashMap<String, Machine>) -> HashMap<String, usize> {
    let mut languages: HashMap<String, usize> = HashMap::new();

    machines
        .values()
        .flat_map(|machine| machine.languages.iter()) // Iterates over each language in the machine
        .for_each(|language| add_item_to_list(&mut languages, language.clone())); // Adds each language to the list

    languages
}

/// Gets a list of unique players from the provided machines, counting their occurrences.
///
/// # Parameters
/// - `machines`: A reference to a `HashMap<String, Machine>` representing the collection of machines.
///
/// # Returns
/// A `HashMap<String, usize>` where keys are player names and values are their counts.
pub fn get_players_list(machines: &HashMap<String, Machine>) -> HashMap<String, usize> {
    let mut players: HashMap<String, usize> = HashMap::new();

    machines
        .values()
        .filter_map(|machine| machine.extended_data.as_ref()?.players.as_ref()) // Filter and map to players string
        .flat_map(|players| players.split(',').map(|s| s.trim().to_string())) // Split players by comma and trim whitespace
        .for_each(|player| add_item_to_list(&mut players, player)); // Add each player to the list

    players
}

/// Gets a list of unique series from the provided machines, counting their occurrences.
///
/// # Parameters
/// - `machines`: A mutable reference to a `HashMap<String, Machine>` representing the collection of machines.
///
/// # Returns
/// A `HashMap<String, usize>` where keys are series names and values are their counts.
pub fn get_series_list(machines: &HashMap<String, Machine>) -> HashMap<String, usize> {
    let mut series: HashMap<String, usize> = HashMap::new();

    machines.values().for_each(|machine| {
        if let Some(series_name) = &machine.series {
            add_item_to_list(&mut series, series_name.clone());
        }
    });

    series
}

/// Gets a list of unique categories from the provided machines, counting their occurrences.
///
/// # Parameters
/// - `machines`: A reference to a `HashMap<String, Machine>` representing the collection of machines.
///
/// # Returns
/// A `HashMap<String, usize>` where keys are category names and values are their counts.
pub fn get_categories_list(machines: &HashMap<String, Machine>) -> HashMap<String, usize> {
    let mut categories: HashMap<String, usize> = HashMap::new();

    machines.values().for_each(|machine| {
        if let Some(category) = &machine.category {
            // Add the category to the categories list
            add_item_to_list(&mut categories, category.clone());
        }
    });

    categories
}

/// Gets a list of unique subcategories from the provided machines, counting their occurrences.
/// The subcategories are formatted as "category - subcategory".
///
/// # Parameters
/// - `machines`: A reference to a `HashMap<String, Machine>` representing the collection of machines.
///
/// # Returns
/// A `HashMap<String, usize>` where keys are subcategory names formatted as "category - subcategory" and values are their counts.
pub fn get_subcategories_list(machines: &HashMap<String, Machine>) -> HashMap<String, usize> {
    let mut subcategories: HashMap<String, usize> = HashMap::new();

    machines.values().for_each(|machine| {
        if let Some(category) = &machine.category {
            if let Some(subcategory) = &machine.subcategory {
                // Combine category and subcategory into a single key
                let key = format!("{} - {}", category, subcategory);
                add_item_to_list(&mut subcategories, key);
            }
        }
    });

    subcategories
}

/// Adds an item to a list stored in a `HashMap`, incrementing its count.
///
/// # Parameters
/// - `map`: A mutable reference to a `HashMap<String, usize>` where keys are item names and values are counts.
/// - `name`: The name of the item to be added.
fn add_item_to_list(map: &mut HashMap<String, usize>, name: String) {
    let counter = map.entry(name).or_insert(0);
    *counter += 1;
}
