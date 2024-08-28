#[cfg(test)]
mod tests {

    use mame_parser::{download_file, CallbackType, MameDataType};
    use std::error::Error;
    use std::path::Path;

    #[test]
    fn test_download_file() -> Result<(), Box<dyn Error>> {
        // Define the workspace path
        let workspace_path = Path::new("playground");

        // Download the file
        let downloaded_file = download_file(
            MameDataType::NPlayers,
            workspace_path,
            Some(
                |downloaded, total_size, message: String, callback_type: CallbackType| {
                    match callback_type {
                        CallbackType::Progress => {
                            // Check that some progress was made
                            assert!(downloaded > 0);
                            assert!(total_size >= downloaded);
                        }
                        CallbackType::Info => {
                            // Check that we receive info messages
                            assert!(!message.is_empty());
                        }
                        CallbackType::Finish => {
                            assert!(!message.is_empty());
                        }
                        CallbackType::Error => {
                            // If there's an error, fail the test
                            panic!("Error during download: {}", message);
                        }
                    }
                },
            ),
        );

        // Check if the file was downloaded successfully
        match downloaded_file {
            Ok(downloaded_file) => {
                let file_path = downloaded_file.as_path().to_str().unwrap();
                // Assert that the file exists
                assert!(Path::new(file_path).exists());
            }
            Err(e) => {
                panic!("Error during download: {}", e);
            }
        }

        Ok(())
    }
}
