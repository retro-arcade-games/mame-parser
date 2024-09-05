use crate::core::callback_progress::{CallbackType, ProgressCallback, ProgressInfo};
use crate::core::filters::nplayers_normalization;
use crate::core::models::Machine;
use anyhow::Context;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Reads and processes the "nplayers.ini" file to extract the number of players for each machine.
///
/// This function reads a specified "nplayers.ini" file line by line, extracts machine information,
/// and populates a `HashMap` with machine names as keys and their corresponding `Machine` structs as values.
/// It identifies the number of players for each machine, normalizes the player count, and stores it in the `extended_data`.
/// Progress updates are provided via a callback function.
///
/// # Parameters
/// - `file_path`: A `&str` representing the path to the "nplayers.ini" file to be read and processed.
/// - `progress_callback`: A callback function of type `ProgressCallback` that tracks progress and provides status updates.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains a `HashMap` where the keys are machine names and the values are `Machine` structs
///   with their associated number of players and normalized player counts.
/// - On failure: Contains an error if the file cannot be opened, read, or if there are issues processing its content.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be opened due to permission issues or if it does not exist.
/// - There are I/O errors while reading the file.
/// - The total number of elements in the file cannot be determined.
///
/// # File structure
/// The `nplayers.ini` file format represents configurations related to the number of players and game types for various ROMs.
/// The file is organized into a single section `[NPlayers]`, where each entry corresponds to a specific ROM and its associated player count or game type.
/// Each line follows the format:
///
/// - `ROM_Name=Player_Count_Or_Game_Type`
///
/// Where:
/// - `ROM_Name`: The name of the ROM file.
/// - `Player_Count_Or_Game_Type`: Describes the number of players or the type of game associated with the ROM.
///
/// Possible values for `Player_Count_Or_Game_Type` include:
///
/// - `1P`: Single-player game.
/// - `2P alt`: Alternate two-player mode.
/// - `2P sim`: Simultaneous two-player mode.
/// - `3P sim`: Simultaneous three-player mode.
/// - `3P alt`: Alternate three-player mode.
/// - `4P alt`: Alternate four-player mode.
/// - `4P sim`: Simultaneous four-player mode.
/// - `4P alt / 2P sim`: Alternate four-player mode or simultaneous two-player mode.
/// - `5P alt`: Alternate five-player mode.
/// - `6P alt`: Alternate six-player mode.
/// - `6P sim`: Simultaneous six-player mode.
/// - `6P alt / 2P sim`: Alternate six-player mode or simultaneous two-player mode.
/// - `8P alt`: Alternate eight-player mode.
/// - `8P alt / 2P sim`: Alternate eight-player mode or simultaneous two-player mode.
/// - `9P alt`: Alternate nine-player mode.
/// - `Pinball`: Pinball game.
/// - `BIOS`: BIOS or system ROM.
/// - `Device`: Non-playable device.
/// - `Non-arcade`: Non-arcade game.
/// - `???`: Unknown or unspecified number of players.
///
/// Lines that start with `[` or `;`, or are empty, are considered comments or section headers and are ignored.
pub fn read_nplayers_file(
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

    let to_ignore = ["[", ";", "", " "];

    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
    let reader = BufReader::new(file);

    let mut processed_count = 0;
    let batch = total_elements / 10;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        let first_char = trimmed.chars().next().unwrap_or(' ');

        // Skip lines that start with any of the ignore characters or patterns
        if to_ignore.contains(&first_char.to_string().as_str()) {
            continue;
        }

        // Process lines with '=' sign
        if let Some(equal_pos) = trimmed.find('=') {
            let (machine_name, value) = trimmed.split_at(equal_pos);
            let machine_name = machine_name.trim();
            let value = &value[1..].trim(); // Skip the '=' and trim the value

            let machine = machines
                .entry(machine_name.to_owned())
                .or_insert_with(|| Machine::new(machine_name.to_owned()));
            // Update machine.players with the value from the file
            machine.players = Some(value.to_string());
            // Add normalized player count to the extended data
            let normalized_name = nplayers_normalization::normalize_nplayer(&machine.players);
            machine.extended_data.as_mut().unwrap().players = Some(normalized_name.clone());

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
/// This function reads the specified file line by line and counts the number of lines
/// that contain an equal sign (`=`), which is used to identify relevant entries. The count
/// represents the total number of elements or entries in the file.
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
fn count_total_elements(file_path: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let file =
        File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
    let reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        let line = line?;
        if line.contains('=') {
            count += 1;
        }
    }

    Ok(count)
}
