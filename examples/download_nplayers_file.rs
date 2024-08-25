use mame_parser::core::mame_data_types::MameDataType;
use mame_parser::download_file;
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let destination_folder = Path::new("playground/downloads");

    std::fs::create_dir_all(destination_folder)?;

    let downloaded_file = download_file(MameDataType::NPlayers, destination_folder);

    match downloaded_file {
        Ok(downloaded_file) => println!(
            "File {} downloaded",
            downloaded_file.as_path().to_str().unwrap()
        ),
        Err(error) => eprintln!("Error downloading file: {}", error),
    }

    Ok(())
}
