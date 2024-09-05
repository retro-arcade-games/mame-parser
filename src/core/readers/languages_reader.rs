use crate::core::callback_progress::{CallbackType, ProgressCallback, ProgressInfo};
use crate::core::models::Machine;
use anyhow::Context;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{collections::HashMap, error::Error};

/// Reads and processes a "languages" file to extract machine language information.
///
/// This function reads a specified "languages" file line by line, extracts machine names
/// and their associated languages, and populates a `HashMap` with machine names as keys
/// and their corresponding `Machine` structs as values. It tracks progress through a callback function.
///
/// # Parameters
/// - `file_path`: A `&str` representing the path to the "languages" file to be read and processed.
/// - `progress_callback`: A callback function of type `ProgressCallback` that tracks progress and provides status updates.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains a `HashMap` where the keys are machine names and the values are `Machine` structs
///   with their associated languages.
/// - On failure: Contains an error if the file cannot be opened, read, or if there are issues processing its content.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be opened due to permission issues or if it does not exist.
/// - There are I/O errors while reading the file.
/// - The total number of elements in the file cannot be determined.
///
/// # File structure
/// The `languages.ini` file format represents configurations and data related to different languages in the system.
/// The file is organized into sections, where each section corresponds to a specific language.
/// Within each language section, entries represent names of ROMs associated with that language.
///
/// - `[FOLDER_SETTINGS]`: A section for folder settings.
///   - `RootFolderIcon`: Specifies the icon for the root folder.
///   - `SubFolderIcon`: Specifies the icon for sub-folders.
///
/// - `[ROOT_FOLDER]`: A placeholder section for root folder configurations (may be empty).
///
/// - `[<Language>]`: Sections where each section header is a language identifier.
///   - Entries: Each entry is a ROM name associated with the specific language.
///
/// Note: Sections are labeled by language names, and the entries under each section are ROM names associated with that language.
pub fn read_languages_file(
    file_path: &str,
    progress_callback: ProgressCallback,
) -> Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>> {
    let mut machines: HashMap<String, Machine> = HashMap::new();
    let data_file_name = file_path.split('/').last().unwrap();

    // Get total elements
    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Getting total entries for {}", data_file_name),
        callback_type: CallbackType::Info,
    });

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

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Reading {}", data_file_name),
        callback_type: CallbackType::Info,
    });

    // Open the file and create a buffered reader
    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
    let reader = BufReader::new(file);
    let mut current_language: Option<String> = None;

    // Define lines to ignore
    let to_ignore = vec![";", "", " ", "", "[FOLDER_SETTINGS]", "[ROOT_FOLDER]"];

    let mut processed_count = 0;
    let batch = total_elements / 10;

    // Process each line of the file
    for line in reader.lines() {
        let line = line?;
        let first_char = line.chars().next().unwrap_or(' ');

        if !to_ignore.contains(&first_char.to_string().as_str())
            && !to_ignore.contains(&line.as_str())
        {
            if first_char == '[' {
                // Set the current language when a new language section starts
                current_language = Some(line.replace("[", "").replace("]", ""));
            } else if let Some(language) = &current_language {
                // If the current language has a slash don't add it to the machine
                if !language.contains("/") {
                    // Get or insert machine
                    let machine_name = line;
                    let machine = machines
                        .entry(machine_name.to_owned())
                        .or_insert_with(|| Machine::new(machine_name.to_owned()));

                    machine.languages.push(language.clone());

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

/// Counts the total number of relevant elements in a file, ignoring specific lines.
///
/// This function reads a specified file line by line and counts the number of lines
/// that are considered relevant entries, based on the criteria defined in the function.
/// Lines that match specific criteria, such as being empty, containing certain keywords,
/// or starting with specific characters, are ignored in the count.
///
/// # Parameters
/// - `file_path`: A `&str` representing the path to the file to be read and analyzed.
///
/// # Returns
/// Returns a `Result<usize, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the total number of relevant lines found in the file.
/// - On failure: Contains an error if the file cannot be opened or read due to I/O issues.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be opened due to permission issues or if it does not exist.
/// - There are I/O errors while reading the file.
///
fn count_total_elements(file_path: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let to_ignore = vec![
        ";",
        "",
        " ",
        "",
        "[FOLDER_SETTINGS]",
        "[ROOT_FOLDER]",
        "[",
        "RootFolderIcon mame",
        "SubFolderIcon folder",
    ];

    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
    let reader = BufReader::new(file);

    let count = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| {
            let first_char = line.chars().next().unwrap_or(' ');
            !to_ignore.contains(&line.as_str())
                && !to_ignore.contains(&first_char.to_string().as_str())
        })
        .count();

    Ok(count)
}
