use indicatif::{ProgressBar, ProgressStyle};
use mame_parser::core::downloader::file_downloader::CallbackType;
use mame_parser::core::mame_data_types::MameDataType;
use mame_parser::download_file;
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let destination_folder = Path::new("playground/downloads");

    std::fs::create_dir_all(destination_folder)?;

    let progress_bar = ProgressBar::new(100);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
            .progress_chars("#>-"),
    );

    let downloaded_file = download_file(
        MameDataType::NPlayers,
        destination_folder,
        Some(
            move |downloaded, total_size, message: String, callback_type: CallbackType| {
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
            },
        ),
    );

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
