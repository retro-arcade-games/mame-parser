use indicatif::{ProgressBar, ProgressStyle};
use mame_parser::core::downloader::file_downloader::download_file_callback;
use mame_parser::core::mame_data_types::MameDataType;
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let destination_folder = Path::new("playground/downloads");

    std::fs::create_dir_all(destination_folder)?;

    let progress_bar = ProgressBar::new(100);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-"),
    );

    let downloaded_file = download_file_callback(
        MameDataType::Mame,
        destination_folder,
        Some(move |downloaded, total_size| {
            if total_size > 0 {
                progress_bar.set_length(total_size);
            }
            progress_bar.set_position(downloaded);

            if downloaded == total_size {
                progress_bar.finish_with_message("Download completed");
            }
        }),
    );

    match downloaded_file {
        Ok(downloaded_file) => println!(
            "File {} downloaded",
            downloaded_file.as_path().to_str().unwrap()
        ),
        Err(error) => eprintln!("Error downloading file: {}", error),
    }

    Ok(())
}
