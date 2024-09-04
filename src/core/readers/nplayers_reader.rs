use crate::core::callback_progress::{CallbackType, ProgressCallback, ProgressInfo};
use crate::core::filters::nplayers_normalization;
use crate::core::models::Machine;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

    let file = File::open(file_path)?;
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

fn count_total_elements(file_path: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let file = File::open(file_path)?;
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
