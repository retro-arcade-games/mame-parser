use crate::{
    core::models::{
        callback_progress::{CallbackType, ProgressCallback, ProgressInfo},
        core_models::Machine,
    },
    helpers::callback_progress_helper::get_progress_info,
};
use anyhow::Context;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Reads and processes a "series.ini" file to extract machine series information.
///
/// This function reads a specified "series.ini" file line by line, extracts machine names,
/// and associates them with their corresponding series. It updates a `HashMap` where the
/// keys are machine names and the values are `Machine` structs containing the series information.
/// Progress updates are provided via a callback function.
///
/// # Parameters
/// - `file_path`: A `&str` representing the path to the "series.ini" file to be read and processed.
/// - `progress_callback`: A callback function of type `ProgressCallback` that tracks progress and provides status updates.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains a `HashMap` where the keys are machine names and the values are `Machine` structs
///   with their associated series information.
/// - On failure: Contains an error if the file cannot be opened, read, or if there are issues processing its content.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be opened due to permission issues or if it does not exist.
/// - There are I/O errors while reading the file.
/// - The total number of elements in the file cannot be determined.
///
/// # File structure
/// The `series.ini` file format represents configurations and data related to different game series in the system.
/// The file is organized into sections, where each section corresponds to a specific game series.
/// Within each series section, entries represent names of ROMs associated with that series.
///
/// - `[FOLDER_SETTINGS]`: A section for folder settings.
///   - `RootFolderIcon`: Specifies the icon for the root folder.
///   - `SubFolderIcon`: Specifies the icon for sub-folders.
///
/// - `[ROOT_FOLDER]`: A placeholder section for root folder configurations (may be empty).
///
/// - `[<Series>]`: Sections where each section header is a game series identifier.
///   - Entries: Each entry is a ROM name associated with the specific game series.
///
/// Note: Sections are labeled by series names, and the entries under each section are ROM names associated with that series.
pub fn read_series_file(
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

    let to_ignore = [";", "", " ", "", "[FOLDER_SETTINGS]", "[ROOT_FOLDER]"];

    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
    let reader = BufReader::new(file);

    let mut current_series: Option<String> = None;

    let mut processed_count = 0;
    let batch = total_elements / 10;

    for line in reader.lines() {
        let line = line?;

        let first_char = line.chars().next().unwrap_or(' ');

        if !to_ignore.contains(&line.as_str())
            && !to_ignore.contains(&first_char.to_string().as_str())
        {
            if first_char == '[' {
                current_series = Some(line.trim_matches(|c| c == '[' || c == ']').to_string());
            } else if let Some(series) = &current_series {
                // Get or insert machine
                let machine_name = line;
                let machine = machines
                    .entry(machine_name.clone())
                    .or_insert_with(|| Machine::new(machine_name));
                // Add the series to the machine
                machine.series = Some(series.clone());
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

    progress_callback(ProgressInfo {
        progress: processed_count as u64,
        total: total_elements as u64,
        message: format!("{} loaded successfully", data_file_name),
        callback_type: CallbackType::Finish,
    });

    Ok(machines)
}

/// Counts the total number of elements in a file, ignoring certain lines based on specific patterns.
///
/// This function reads a specified file line by line and counts the number of lines that are not in a predefined list of
/// patterns to ignore. The lines to ignore include comments, empty lines, and specific configuration sections or icons.
///
/// # Parameters
/// - `file_path`: A `&str` representing the path to the file to be read and analyzed.
///
/// # Returns
/// Returns a `Result<usize, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the total number of lines that do not match any of the ignored patterns.
/// - On failure: Contains an error if the file cannot be opened or read due to I/O issues.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be opened due to permission issues or if it does not exist.
/// - There are I/O errors while reading the file.
fn count_total_elements(file_path: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let to_ignore = [
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
        .filter_map(Result::ok)
        .filter(|line| {
            !to_ignore.contains(&line.as_str())
                && !to_ignore.contains(&line.get(0..1).unwrap_or(""))
        })
        .count();

    Ok(count)
}
