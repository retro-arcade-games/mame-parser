use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{collections::HashMap, error::Error};

use crate::core::callback_progress::{CallbackType, ProgressCallback, ProgressInfo};
use crate::core::models::Machine;

pub fn read_catver_file(
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

fn count_total_elements(file_path: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.contains('=') {
            count += 1;
        }
    }

    Ok(count)
}
