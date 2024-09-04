use indicatif::{ProgressBar, ProgressStyle};
use mame_parser::{download_file, CallbackType, MameDataType, ProgressCallback, ProgressInfo};
use std::error::Error;
use std::path::Path;

#[docify::export]
fn main() -> Result<(), Box<dyn Error>> {
    // Define the workspace path
    let workspace_path = Path::new("playground");

    // Create a progress bar
    let progress_bar = ProgressBar::new(100);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:20.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
            .progress_chars("#>-"),
    );

    let progress_callback: ProgressCallback = Box::new(move |progress_info: ProgressInfo| {
        // Update the progress bar
        match progress_info.callback_type {
            CallbackType::Progress => {
                progress_bar.set_length(progress_info.total);
                progress_bar.set_position(progress_info.progress);
            }
            CallbackType::Info => {
                progress_bar.set_message(progress_info.message);
            }
            CallbackType::Finish => {
                progress_bar.set_length(progress_info.total);
                progress_bar.set_position(progress_info.progress);
                progress_bar.finish_with_message(progress_info.message);
            }
            CallbackType::Error => {
                progress_bar.finish_with_message(progress_info.message);
            }
        }
    });

    // Download the file
    let downloaded_file = download_file(MameDataType::Series, workspace_path, progress_callback);

    // Print the result
    match downloaded_file {
        Ok(downloaded_file) => {
            println!(
                "Downloaded file: {}",
                downloaded_file.as_path().to_str().unwrap()
            );
        }
        Err(e) => {
            eprintln!("Error during download: {}", e);
        }
    }

    Ok(())
}
