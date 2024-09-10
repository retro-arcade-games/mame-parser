use crate::{
    core::models::{
        callback_progress::{CallbackType, ProgressCallback, ProgressInfo},
        core_models::Machine,
    },
    helpers::callback_progress_helper::get_progress_info,
};
use anyhow::Context;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{collections::HashMap, error::Error};

/// Reads and processes a catver.ini file to extract machine categories and subcategories.
///
/// This function reads a specified catver.ini file line by line, extracts machine information,
/// and populates a `HashMap` with machine names as keys and their corresponding `Machine` structs as values.
/// It identifies categories, subcategories, and flags machines marked as "Mature".
/// Progress updates are provided via a callback function.
///
/// # Parameters
/// - `file_path`: A `&str` representing the path to the catver.ini file to be read and processed.
/// - `progress_callback`: A callback function of type `ProgressCallback` that tracks progress and provides status updates.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains a `HashMap` where the keys are machine names and the values are `Machine` structs
///   with their associated categories, subcategories, and maturity flags.
/// - On failure: Contains an error if the file cannot be opened, read, or if there are issues processing its content.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be opened due to permission issues or if it does not exist.
/// - There are I/O errors while reading the file.
/// - The total number of elements in the file cannot be determined.
///
/// # File structure
/// The `catver.ini` file represents configurations and data related to game classification in the MAME system.
/// The file is organized into lines, where each line corresponds to a game entry with its category and subcategory.
///
/// The file structure is as follows:
///
/// - `[FOLDER_SETTINGS]`: An optional section for folder settings.
///   - `RootFolderIcon`: Specifies the icon for the root folder.
///   - `SubFolderIcon`: Specifies the icon for sub-folders.
///
/// - `[ROOT_FOLDER]`: A placeholder section for root folder configurations (may be empty).
///
/// - `<ROM Name>=<Category> / <Subcategory> * Mature *`
///   - `<ROM Name>`: The name of the ROM being configured.
///   - `<Category>`: The category of the game.
///   - `<Subcategory>`: The subcategory of the game, which may be followed by `* Mature *` if the game is marked as mature.
///
/// Note: The `category` and `subcategory` are separated by ` / `, and the subcategory may or may not end with the `* Mature *` marker.
///
pub fn read_catver_file(
    file_path: &str,
    progress_callback: ProgressCallback,
) -> Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>> {
    let mut machines: HashMap<String, Machine> = HashMap::new();

    let data_file_name = file_path.split('/').last().unwrap();

    // Get total elements
    progress_callback(get_progress_info(
        format!("Getting total entries for {}", data_file_name).as_str(),
    ));

    let total_elements = match count_total_elements(file_path) {
        Ok(total_elements) => total_elements,
        Err(err) => {
            progress_callback(ProgressInfo {
                progress: 0,
                total: 0,
                message: format!("Couldn't get total entries for {}", data_file_name),
                callback_type: CallbackType::Error,
            });

            return Err(err.into());
        }
    };

    progress_callback(get_progress_info(
        format!("Reading {}", data_file_name).as_str(),
    ));

    let to_ignore = ["[", ";", "", " "];

    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
    let reader = BufReader::new(file);

    let mut processed_count = 0;
    let batch = total_elements / 10;

    for line in reader.lines() {
        let line = line.with_context(|| format!("Failed to read line in file: {}", file_path))?;
        let trimmed = line.trim();
        let first_char = trimmed.chars().next().unwrap_or(' ');

        if to_ignore.contains(&first_char.to_string().as_str()) {
            continue;
        }

        if let Some(equal_pos) = trimmed.find('=') {
            let (machine_name, value) = trimmed.split_at(equal_pos);
            let machine_name = machine_name.trim();
            let value = &value[1..].trim(); // Skip the '=' and trim the value

            let parts: Vec<&str> = value.split(" / ").collect();
            if parts.len() >= 2 {
                let category = parts[0].to_string();
                let mut subcategory = parts[1].to_string();
                let is_mature = subcategory.ends_with(" * Mature *");

                if is_mature {
                    subcategory = subcategory
                        .trim_end_matches(" * Mature *")
                        .trim()
                        .to_string();
                }
                // Get or insert machine
                let machine = machines
                    .entry(machine_name.to_owned())
                    .or_insert_with(|| Machine::new(machine_name.to_owned()));

                machine.category = Some(category);
                machine.subcategory = Some(subcategory);
                machine.is_mature = Some(is_mature);
            }
            // Increase processed count
            processed_count += 1;
            // Progress callback
            if processed_count % batch == 0 {
                progress_callback(ProgressInfo {
                    progress: processed_count as u64,
                    total: total_elements as u64,
                    message: String::from(""),
                    callback_type: CallbackType::Progress,
                });
            }
        }
    }

    progress_callback(ProgressInfo {
        progress: processed_count as u64,
        total: total_elements as u64,
        message: format!("{} loaded successfully", data_file_name),
        callback_type: CallbackType::Finish,
    });

    Ok(machines)
}

/// Counts the total number of elements in a file based on the presence of an equal sign (`=`).
///
/// This function reads a specified file line by line and counts the number of lines
/// that contain an equal sign (`=`), which is used to identify relevant entries.
/// The count represents the total number of elements or entries in the file.
///
/// # Parameters
/// - `file_path`: A `&str` representing the path to the file to be read and analyzed.
///
/// # Returns
/// Returns a `Result<usize, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the total number of lines with an equal sign, representing the total entries found in the file.
/// - On failure: Contains an error if the file cannot be opened or read due to I/O issues.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be opened due to permission issues or if it does not exist.
/// - There are I/O errors while reading the file.
///
fn count_total_elements(file_path: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
    let reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        let line = line.with_context(|| format!("Failed to read a line in file: {}", file_path))?;
        if line.trim().contains('=') {
            count += 1;
        }
    }

    Ok(count)
}
