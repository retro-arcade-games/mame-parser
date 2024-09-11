#[cfg(test)]
mod tests {
    use mame_parser::file_handling::{read_file, remove_machines_by_category};
    use mame_parser::models::{Category, MameDataType};
    use mame_parser::progress::{CallbackType, ProgressCallback, ProgressInfo};
    use std::error::Error;
    use std::path::Path;

    #[test]
    fn test_read_file() -> Result<(), Box<dyn Error>> {
        // Define the workspace path
        let workspace_path = Path::new("playground");

        // Define the progress callback without using a progress bar
        let progress_callback: ProgressCallback = Box::new(move |progress_info: ProgressInfo| {
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
                    panic!("Error during reading: {}", progress_info.message);
                }
            }
        });

        // Attempt to unpack the file
        let machines = read_file(MameDataType::Catver, workspace_path, progress_callback);

        // Verify that the file was read successfully
        match machines {
            Ok(machines) => {
                // Categories to remove
                let categories_to_remove =
                    vec![Category::Sports, Category::Puzzle, Category::SlotMachine];

                // Filter the machines
                let filtered_machines =
                    remove_machines_by_category(&machines, &categories_to_remove);

                assert!(
                    machines.len() > filtered_machines.unwrap().len(),
                    "Machine count is the same"
                );
            }
            Err(e) => {
                panic!("Error during unpacking: {}", e);
            }
        }

        Ok(())
    }
}
