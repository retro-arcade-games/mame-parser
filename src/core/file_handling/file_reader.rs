use crate::helpers::file_system_helpers::{find_file_with_pattern, WORKSPACE_PATHS};
use crate::{
    core::models::{
        callback_progress::{CallbackType, ProgressCallback, ProgressInfo, SharedProgressCallback},
        core_models::Machine,
        mame_data_types::{get_data_type_details, MameDataType},
    },
    helpers::callback_progress_helper::get_progress_info,
};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::thread;

/// Reads and processes a specific MAME data file based on the provided data type.
///
/// This function handles the retrieval of a MAME data file from the extracted folder and processes it
/// by reading its contents and returning a map of machine details. It first checks if the required
/// data file is present in the expected location and then reads the file using a specialized function
/// for the provided `MameDataType`. Progress updates and messages are provided via a callback function.
///
/// # Parameters
/// - `data_type`: The `MameDataType` that specifies which type of MAME data file to read (e.g., ROMs, DAT files).
/// - `workspace_path`: A reference to a `Path` representing the base directory where the data file is located.
/// - `progress_callback`: A callback function of type `ProgressCallback` that tracks progress and provides status updates.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains a `HashMap` where the keys are machine names and the values are `Machine` structs representing detailed information about each MAME machine.
/// - On failure: Contains an error if the data file is not found, cannot be read, or if there are issues accessing the file system.
///
/// # Errors
/// This function will return an error if:
/// - The data file is not found in the expected location.
/// - The data file cannot be read due to permission issues or file corruption.
/// - There are errors in processing the data file content using the corresponding read function.
///
/// # Callback
/// The progress callback function provides real-time updates on the reading process and other status information. It receives:
/// - `progress`: The current progress of the operation (e.g., number of entries).
/// - `total`: The total number of items to be processed.
/// - `message`: A status message indicating the current operation (e.g., "Checking if data file is present", "Reading file").
/// — `callback_type`: The type of callback, such as `CallbackType::Info`, `CallbackType::Error`, `CallbackType::Progress`, or `CallbackType::Finish`.
///
/// # Example
#[doc = docify::embed!("examples/read_file.rs", main)]
///
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
    progress_callback(get_progress_info(
        format!(
            "Checking if data file for {} is present",
            data_type_details.name
        )
        .as_str(),
    ));

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

/// Reads and processes all MAME data files available for the specified workspace path.
///
/// This function manages the concurrent reading of multiple MAME data files. For each `MameDataType`,
/// it spawns a separate thread to handle the file reading process. The function waits for all threads
/// to complete and then combines the results into a single `HashMap` of machine details. Progress updates
/// and messages are provided via a shared callback function.
///
/// # Parameters
/// - `workspace_path`: A reference to a `Path` representing the base directory where all data files are located.
/// - `progress_callback`: A shared callback function of type `SharedProgressCallback` that tracks progress and provides status updates.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<HashMap<String, Machine>, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains a `HashMap` where the keys are machine names and the values are `Machine` structs representing detailed information about each MAME machine.
/// - On failure: Contains an error if any data file cannot be read, or if there are issues joining the threads.
///
/// # Errors
/// This function will return an error if:
/// - Any thread fails to complete successfully or panics.
/// - There are issues reading any data file due to permission problems, file corruption, or missing files.
/// - There are errors during the merging of machine data, such as data inconsistencies.
///
/// # Concurrency
/// This function uses multiple threads to read MAME data files concurrently. Each thread handles the reading of a specific
/// data type file (`MameDataType`). The function waits for all threads to complete using `join()`, and any errors encountered
/// are captured and logged. The shared progress callback is used to provide real-time updates across all threads.
///
/// # Callback
/// The shared progress callback function provides real-time updates on the reading process for each data type and other status information. It receives:
/// - `progress`: The current progress of the operation for a specific data type (e.g., number of files processed).
/// - `total`: The total number of items to be processed (if available).
/// - `message`: A status message indicating the current operation (e.g., "Reading file", "Processing data").
/// - `callback_type`: The type of callback, such as `CallbackType::Info`, `CallbackType::Error`, `CallbackType::Progress`, or `CallbackType::Finish`.
///
/// # Example
#[doc = docify::embed!("examples/read_files.rs", main)]
///
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
