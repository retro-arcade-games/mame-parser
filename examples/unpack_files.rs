use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mame_parser::{unpack_files, CallbackType, MameDataType};
use std::error::Error;
use std::path::Path;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Define the workspace path
    let workspace_path = Path::new("playground");

    // Create a multi progress bar
    let multi_progress = MultiProgress::new();

    // Create progress bars for each data type
    let progress_bars = Arc::new(
        MameDataType::all_variants()
            .iter()
            .map(|&data_type| {
                let progress_bar = multi_progress.add(ProgressBar::new(100));
                progress_bar.set_style(
                    ProgressStyle::default_bar()
                        .template(&format!("{{spinner:.green}} [{{elapsed_precise}}] [{{bar:20.cyan/blue}}] {{pos}}/{{len}} ({{eta}}) {{msg}}"))
                        .progress_chars("#>-"),
                );
                (data_type, progress_bar)
            })
            .collect::<Vec<_>>(),
    );

    // Unpack the files
    let handles = unpack_files(
        workspace_path,
        move |data_type,
              unpacked_files,
              total_files,
              message: String,
              callback_type: CallbackType| {
            if let Some((_, progress_bar)) = progress_bars.iter().find(|(dt, _)| *dt == data_type) {
                // Update the progress bar
                match callback_type {
                    CallbackType::Progress => {
                        progress_bar.set_length(total_files);
                        progress_bar.set_position(unpacked_files);
                    }
                    CallbackType::Info => {
                        progress_bar.set_message(message);
                    }
                    CallbackType::Finish => {
                        progress_bar.set_length(total_files);
                        progress_bar.set_position(unpacked_files);
                        progress_bar.finish_with_message(message);
                    }
                    CallbackType::Error => {
                        progress_bar.finish_with_message(message);
                    }
                }
            }
        },
    );

    // Wait for all threads to finish
    multi_progress.join().unwrap();

    // Print the result
    for handle in handles {
        match handle.join().unwrap() {
            Ok(path) => {
                println!("Unpacked data file: {}", path.display());
            }
            Err(e) => {
                eprintln!("Error during unpacking: {}", e);
            }
        }
    }

    Ok(())
}
