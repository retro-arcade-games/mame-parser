#[cfg(test)]
mod tests {
    use mame_parser::file_handling::{read_files, remove_machines_by_category, write_files};
    use mame_parser::models::{Category, ExportFileType, MameDataType};
    use mame_parser::progress::{
        CallbackType, ProgressCallback, ProgressInfo, SharedProgressCallback,
    };
    use std::error::Error;
    use std::path::Path;
    use std::sync::Arc;

    #[test]
    fn test_unpack_files() -> Result<(), Box<dyn Error + Send + Sync>> {
        // Define the workspace path
        let workspace_path = Path::new("playground");

        // Define the shared progress callback without using a progress bar
        let shared_progress_callback: SharedProgressCallback = Arc::new(
            move |data_type: MameDataType, progress_info: ProgressInfo| {
                // Handle progress updates
                match progress_info.callback_type {
                    CallbackType::Progress => {
                        // Check that some progress was made
                        assert!(progress_info.progress > 0);
                        assert!(progress_info.total >= progress_info.progress);
                    }
                    CallbackType::Info => {
                        // Check that we receive info messages
                        assert!(!progress_info.message.is_empty());
                    }
                    CallbackType::Finish => {
                        // Verify that the process finished correctly
                        assert!(!progress_info.message.is_empty());
                        assert!(progress_info.progress == progress_info.total);
                    }
                    CallbackType::Error => {
                        panic!(
                            "Error during reading for {:?}: {}",
                            data_type, progress_info.message
                        );
                    }
                }
            },
        );

        // Read the files
        let machines = read_files(workspace_path, shared_progress_callback);

        // Verify that the files were read successfully
        match machines {
            Ok(machines) => {
                // Filter machines to reduce writing time
                let categories_to_remove = vec![
                    Category::Sports,
                    Category::Puzzle,
                    Category::SlotMachine,
                    Category::Handheld,
                    Category::Electromechanical,
                ];
                let machines = remove_machines_by_category(&machines, &categories_to_remove);
                // Create a progress callback
                let progress_callback: ProgressCallback =
                    Box::new(move |progress_info: ProgressInfo| {
                        // Update the progress bar
                        match progress_info.callback_type {
                            CallbackType::Progress => {
                                // Check that some progress was made
                                assert!(progress_info.progress > 0);
                                assert!(progress_info.total >= progress_info.progress);
                            }
                            CallbackType::Info => {
                                // Check that we receive info messages
                                assert!(!progress_info.message.is_empty());
                            }
                            CallbackType::Finish => {
                                // Verify that the process finished correctly
                                assert!(!progress_info.message.is_empty());
                                assert!(progress_info.progress == progress_info.total);
                            }
                            CallbackType::Error => {
                                panic!("Error during writing csv");
                            }
                        }
                    });
                // Write the machines to JSON files
                let result = write_files(
                    ExportFileType::Json,
                    workspace_path,
                    &machines.unwrap(),
                    progress_callback,
                );
                match result {
                    Ok(result) => {
                        let file_path = result.join("machines.json");
                        // Assert that the file exists
                        assert!(Path::new(&file_path).exists());
                    }
                    Err(e) => {
                        eprintln!("Error writing data files: {}", e);
                    }
                }
            }
            Err(e) => {
                panic!("Error during unpacking: {}", e);
            }
        }

        Ok(())
    }
}
