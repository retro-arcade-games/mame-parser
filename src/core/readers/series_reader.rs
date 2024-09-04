use crate::core::callback_progress::{CallbackType, ProgressCallback, ProgressInfo};
use crate::core::models::Machine;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn read_series_file(
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

    let to_ignore = [";", "", " ", "", "[FOLDER_SETTINGS]", "[ROOT_FOLDER]"];

    let file = File::open(file_path)?;
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

    let file = File::open(file_path)?;
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
