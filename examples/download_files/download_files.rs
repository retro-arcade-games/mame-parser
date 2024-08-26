use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mame_parser::core::downloader::file_downloader::{download_files, CallbackType};
use mame_parser::core::mame_data_types::MameDataType;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let destination_folder = Path::new("playground/downloads");
    std::fs::create_dir_all(destination_folder)?;

    let multi_progress = MultiProgress::new();

    let progress_bars = Arc::new(
        MameDataType::all_variants()
            .iter()
            .map(|&data_type| {
                let label = format!("{:?}", data_type);
                let progress_bar = multi_progress.add(ProgressBar::new(100));
                progress_bar.set_style(
                    ProgressStyle::default_bar()
                        .template(&format!(
                            "{{spinner:.green}} [{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{eta}}) {{msg}} {}",
                            label
                        ))
                        .progress_chars("#>-"),
                );
                (data_type, progress_bar)
            })
            .collect::<Vec<_>>(),
    );

    let handles = download_files(
        destination_folder,
        move |data_type, downloaded, total_size, message: String, callback_type: CallbackType| {
            if let Some((_, progress_bar)) = progress_bars.iter().find(|(dt, _)| *dt == data_type) {
                match callback_type {
                    CallbackType::Progress => {
                        progress_bar.set_length(total_size);
                        progress_bar.set_position(downloaded);
                        if downloaded == total_size {
                            progress_bar.finish_with_message(format!("Download completed"));
                        }
                    }
                    CallbackType::Info => {
                        progress_bar.set_message(message);
                    }
                    CallbackType::Error => {
                        progress_bar.finish_with_message(message);
                    }
                }
            }
        },
    );

    multi_progress.join().unwrap();

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
