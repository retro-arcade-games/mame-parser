use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::thread;

use crate::core::callback_progress::{
    CallbackType, ProgressCallback, ProgressInfo, SharedProgressCallback,
};
use crate::core::mame_data_types::{get_data_type_details, MameDataType};
use crate::core::models::Machine;
use crate::helpers::file_system_helpers::{find_file_with_pattern, WORKSPACE_PATHS};

pub fn read_file(
    data_type: MameDataType,
    workspace_path: &Path,
    progress_callback: ProgressCallback,
) -> Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>> {
    // Retrieves the details for a given `MameDataType`
    let data_type_details = get_data_type_details(data_type);
    // Set path where data file is located
    let extract_folder = workspace_path
        .join(WORKSPACE_PATHS.extract_path)
        .join(data_type_details.name.to_lowercase());

    // Checks if data file is present in the extract folder
    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!(
            "Checking if data file for {} is present",
            data_type_details.name
        ),
        callback_type: CallbackType::Info,
    });

    let existing_data_file = find_file_with_pattern(
        &extract_folder.to_str().unwrap(),
        &data_type_details.data_file_pattern,
    );

    if let Err(err) = existing_data_file {
        progress_callback(ProgressInfo {
            progress: 0,
            total: 0,
            message: format!("Data file for {} not found", data_type_details.name),
            callback_type: CallbackType::Error,
        });

        return Err(err.into());
    }

    let file_path = existing_data_file.unwrap();

    let machines = (data_type_details.read_function)(&file_path, progress_callback)?;

    Ok(machines)
}

pub fn read_files(
    workspace_path: &Path,
    progress_callback: SharedProgressCallback,
) -> Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>> {
    let progress_callback = Arc::clone(&progress_callback);

    let handles: Vec<_> = MameDataType::all_variants()
        .iter()
        .map(|&data_type| {
            let workspace_path = workspace_path.to_path_buf();
            let progress_callback = Arc::clone(&progress_callback);

            thread::spawn(move || {
                read_file(
                    data_type,
                    &workspace_path,
                    Box::new(move |progress_info| {
                        progress_callback(data_type, progress_info);
                    }),
                )
            })
        })
        .collect();

    let mut combined_machines = HashMap::new();

    for handle in handles {
        match handle.join() {
            Ok(Ok(machines)) => {
                for (key, new_machine) in machines {
                    combined_machines
                        .entry(key)
                        .and_modify(|existing_machine: &mut Machine| {
                            existing_machine.combine(&new_machine)
                        })
                        .or_insert(new_machine);
                }
            }
            Ok(Err(err)) => {
                eprintln!("Error reading file: {:?}", err);
            }
            Err(err) => {
                eprintln!("Error joining thread: {:?}", err);
            }
        }
    }

    Ok(combined_machines)
}
