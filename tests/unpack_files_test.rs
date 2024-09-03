#[cfg(test)]
mod tests {
    use mame_parser::{
        unpack_files, CallbackType, MameDataType, ProgressInfo, SharedProgressCallback,
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
                            "Error during unpacking for {:?}: {}",
                            data_type, progress_info.message
                        );
                    }
                }
            },
        );

        // Unpack the files
        let handles = unpack_files(workspace_path, shared_progress_callback);

        // Wait for all threads to finish and check the results
        for handle in handles {
            match handle.join().unwrap() {
                Ok(path) => {
                    // Assert that the unpacked file path exists
                    assert!(path.exists(), "Unpacked file does not exist: {:?}", path);
                }
                Err(e) => {
                    panic!("Error during unpacking: {}", e);
                }
            }
        }

        Ok(())
    }
}
