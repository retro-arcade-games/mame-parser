#[cfg(test)]
mod tests {

    use mame_parser::file_handling::download_files;
    use mame_parser::models::MameDataType;
    use mame_parser::progress::{CallbackType, SharedProgressCallback};
    use std::collections::HashMap;
    use std::error::Error;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_download_files() -> Result<(), Box<dyn Error + Send + Sync>> {
        // Define the workspace path
        let workspace_path = Path::new("playground");

        // Use a thread-safe structure to track progress for each data type
        let progress_data = Arc::new(Mutex::new(HashMap::new()));

        let shared_progress_callback: SharedProgressCallback = Arc::new({
            let progress_data = Arc::clone(&progress_data);
            move |data_type, progress_info| {
                let mut progress = progress_data.lock().unwrap();

                match progress_info.callback_type {
                    CallbackType::Progress => {
                        progress.insert(data_type, (progress_info.progress, progress_info.total));
                    }
                    CallbackType::Info => {
                        progress.insert(data_type, (progress_info.progress, progress_info.total));
                    }
                    CallbackType::Finish => {
                        progress.insert(data_type, (progress_info.progress, progress_info.total));
                    }
                    CallbackType::Error => {
                        panic!(
                            "Error during download of {:?}: {}",
                            data_type, progress_info.message
                        );
                    }
                }
            }
        });

        // Download the files
        let handles = download_files(workspace_path, shared_progress_callback);

        // Wait for all threads to finish and check results
        for handle in handles {
            match handle.join().unwrap() {
                Ok(path) => {
                    let file_path = path.as_path().to_str().unwrap();
                    // Assert that the file exists
                    assert!(Path::new(file_path).exists());
                }
                Err(e) => {
                    panic!("Error during download: {}", e);
                }
            }
        }

        // Check the progress data to ensure all downloads completed
        let progress = progress_data.lock().unwrap();
        for data_type in MameDataType::all_variants() {
            if let Some((downloaded, total_size)) = progress.get(data_type) {
                assert!(total_size >= downloaded);
            } else {
                panic!("No progress recorded for {:?}", data_type);
            }
        }

        Ok(())
    }
}
