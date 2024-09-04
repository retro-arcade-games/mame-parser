use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mame_parser::{
    download_files, CallbackType, MameDataType, ProgressInfo, SharedProgressCallback,
};
use std::error::Error;
use std::path::Path;
use std::sync::Arc;

#[docify::export]
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
                        .template(&format!("{{spinner:.green}} [{{elapsed_precise}}] [{{bar:20.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{eta}}) {{msg}}"))
                        .progress_chars("#>-"),
                );
                (data_type, progress_bar)
            })
            .collect::<Vec<_>>(),
    );

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

    // Download the files
    let handles = download_files(workspace_path, shared_progress_callback);

    // Wait for all threads to finish
    multi_progress.join().unwrap();

    // Print the result
    for handle in handles {
        match handle.join().unwrap() {
            Ok(path) => {
                println!("Downloaded file: {}", path.display());
            }
            Err(e) => {
                eprintln!("Error during download: {}", e);
            }
        }
    }

    Ok(())
}
