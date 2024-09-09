use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mame_parser::file_handling::{read_files, write_files};
use mame_parser::models::{ExportFileType, MameDataType};
use mame_parser::progress::{CallbackType, ProgressCallback, ProgressInfo, SharedProgressCallback};
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::thread;

#[docify::export]
fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Define the workspace path
    let workspace_path = Path::new("playground");

    // Create a multi progress bar
    let multi_progress: Arc<MultiProgress> = Arc::new(MultiProgress::new());

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

    // Create a shared progress callback
    let shared_progress_callback: SharedProgressCallback = Arc::new(
        move |data_type: MameDataType, progress_info: ProgressInfo| {
            if let Some((_, progress_bar)) = progress_bars.iter().find(|(dt, _)| *dt == data_type) {
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
            }
        },
    );

    let handle = thread::spawn(move || {
        multi_progress.join().unwrap();
    });

    // Read the files
    let machines = read_files(workspace_path, shared_progress_callback);

    handle.join().unwrap();

    // Print the result
    match machines {
        Ok(machines) => {
            let progress_bar = ProgressBar::new(100);
            progress_bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:20.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                    .progress_chars("#>-"),
            );

            // Create a progress callback
            let progress_callback: ProgressCallback =
                Box::new(move |progress_info: ProgressInfo| {
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
            // Write the machines to CSV files
            let result = write_files(
                ExportFileType::Csv,
                workspace_path,
                &machines,
                progress_callback,
            );
            match result {
                Ok(result) => {
                    println!("Machines written to {}.", result.to_string_lossy());
                }
                Err(e) => {
                    eprintln!("Error writing data files: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading data files: {}", e);
        }
    }

    Ok(())
}
