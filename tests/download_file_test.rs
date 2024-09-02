#[cfg(test)]
mod tests {

    use mame_parser::{download_file, CallbackType, MameDataType, ProgressCallback, ProgressInfo};
    use std::error::Error;
    use std::path::Path;

    #[test]
    fn test_download_file() -> Result<(), Box<dyn Error>> {
        // Define the workspace path
        let workspace_path = Path::new("playground");

        let progress_callback: ProgressCallback = Box::new(move |progress_info: ProgressInfo| {
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
                    assert!(!progress_info.message.is_empty());
                    assert!(progress_info.progress == progress_info.total);
                }
                CallbackType::Error => {
                    panic!("Error during download: {}", progress_info.message);
                }
            }
        });

        // Download the file
        let downloaded_file =
            download_file(MameDataType::NPlayers, workspace_path, progress_callback);

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
