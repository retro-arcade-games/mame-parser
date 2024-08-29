use indicatif::{ProgressBar, ProgressStyle};
use mame_parser::{unpack_file, CallbackType, MameDataType};
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    // Define the workspace path
    let workspace_path = Path::new("playground");

    // Create a progress bar
    let progress_bar = ProgressBar::new(100);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:20.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .progress_chars("#>-"),
    );

    // Unpack the file
    let unpacked_file = unpack_file(
        MameDataType::Series,
        workspace_path,
        Some(
            move |downloaded, total_size, message: String, callback_type: CallbackType| {
                // Update the progress bar
                match callback_type {
                    CallbackType::Progress => {
                        progress_bar.set_length(total_size);
                        progress_bar.set_position(downloaded);
                    }
                    CallbackType::Info => {
                        progress_bar.set_message(message);
                    }
                    CallbackType::Finish => {
                        progress_bar.set_length(total_size);
                        progress_bar.set_position(downloaded);
                        progress_bar.finish_with_message(message);
                    }
                    CallbackType::Error => {
                        progress_bar.finish_with_message(message);
                    }
                }
            },
        ),
    );

    // Print the result
    match unpacked_file {
        Ok(data_file) => {
            println!(
                "Unpacked data file: {}",
                data_file.as_path().to_str().unwrap()
            );
        }
        Err(e) => {
            eprintln!("Error during unpacking: {}", e);
        }
    }

    Ok(())
}
