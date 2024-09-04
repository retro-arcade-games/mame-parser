use crate::core::callback_progress::{CallbackType, ProgressCallback, ProgressInfo};
use crate::core::models::Machine;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{collections::HashMap, error::Error};

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
    let file = File::open(file_path)?;
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

    let file = File::open(file_path)?;
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
