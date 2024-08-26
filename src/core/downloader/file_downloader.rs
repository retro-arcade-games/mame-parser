use reqwest::blocking::Client;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use crate::core::mame_data_types::{get_data_type_details, MameDataType};
use crate::helpers::data_source_helper::get_data_source;

pub enum CallbackType {
    Info,
    Progress,
    Error,
}

/// Downloads a file for the given MameDataType.
///
/// This function initiates the download of a file specified by the `MameDataType`
/// and saves it to the specified destination folder. The function is a simplified
/// wrapper around `download_file_callback`, without providing a progress callback.
///
/// # Parameters
/// - `data_type`: The type of data to download, represented by `MameDataType`.
/// - `destination_folder`: The path where the downloaded file should be saved.
///
/// # Returns
/// A `Result` containing the path to the downloaded file (`PathBuf`) if successful,
/// or an error (`Box<dyn Error + Send + Sync>`) if the download fails.
///
/// # Errors
/// This function will return an error if the download fails, if the destination
/// folder cannot be created, or if the file cannot be written.
///
/// # Examples
/// ```
/// use mame_parser::core::mame_data_types::MameDataType;
/// use mame_parser::download_file;
/// use std::path::Path;
///
/// let destination_folder = Path::new("downloads");
/// let result = download_file(MameDataType::NPlayers, destination_folder);
///
/// match result {
///     Ok(path) => println!("File downloaded to: {:?}", path),
///     Err(e) => eprintln!("Failed to download file: {}", e),
/// }
/// ```

pub fn download_file<F>(
    data_type: MameDataType,
    destination_folder: &Path,
    progress_callback: Option<F>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>>
where
    F: Fn(u64, u64, String, CallbackType) + Send + 'static,
{
    let details = get_data_type_details(data_type);

    if let Some(ref callback) = progress_callback {
        callback(0, 0, String::from("Searching url"), CallbackType::Info);
    }

    let download_url = get_data_source(&details.source, &details.source_match);

    match download_url {
        Ok(url) => {
            if let Some(ref callback) = progress_callback {
                callback(0, 0, String::from("Downloading"), CallbackType::Info);
            }
            download(&url, destination_folder, progress_callback)
        }
        Err(err) => {
            if let Some(ref callback) = progress_callback {
                callback(0, 0, String::from("Couldn't find url"), CallbackType::Error);
            }
            Err(err)
        }
    }
}

pub fn download_files<F>(
    destination_folder: &Path,
    progress_callback: F,
) -> Vec<thread::JoinHandle<Result<PathBuf, Box<dyn Error + Send + Sync>>>>
where
    F: Fn(MameDataType, u64, u64, String, CallbackType) + Send + Sync + 'static,
{
    let progress_callback = Arc::new(progress_callback);

    MameDataType::all_variants()
        .iter()
        .map(|&data_type| {
            let destination_folder = destination_folder.to_path_buf();
            let progress_callback = Arc::clone(&progress_callback);

            thread::spawn(move || {
                download_file(
                    data_type,
                    &destination_folder,
                    Some(move |downloaded, total_size, message, callback_type| {
                        progress_callback(
                            data_type,
                            downloaded,
                            total_size,
                            message,
                            callback_type,
                        );
                    }),
                )
            })
        })
        .collect()
}

fn download<F>(
    url: &str,
    destination_folder: &Path,
    progress_callback: Option<F>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>>
where
    F: Fn(u64, u64, String, CallbackType) + Send + 'static,
{
    let file_name = get_file_name(url);

    let mut response = Client::new().get(url).send()?;
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut buffer = [0; 4096];

    let file_path = destination_folder.join(file_name);
    let mut file = File::create(&file_path)?;

    while let Ok(bytes_read) = response.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;

        if let Some(ref callback) = progress_callback {
            callback(
                downloaded,
                total_size,
                String::from(""),
                CallbackType::Progress,
            );
        }
    }

    if let Some(ref callback) = progress_callback {
        callback(
            downloaded,
            downloaded,
            String::from(""),
            CallbackType::Progress,
        );
    }

    Ok(file_path)
}

/// Extracts the file name from a given URL.
///
/// The function parses the URL and returns the last segment, which is typically the file name.
/// If the URL contains a query parameter in the format `?file=name.ext`, it returns the value of that parameter.
///
/// /// # Parameters
/// - `url`: A string slice (`&str`) representing the URL from which the file name should be extracted.
///
/// # Returns
/// Returns a `String` containing the extracted file name from the URL. If no valid file name is found,
/// it returns an empty string.
///
/// # Examples
/// ```
/// use mame_parser::core::downloader::file_downloader::get_file_name;
///
/// let url = "https://example.com/download?file=example.zip";
/// let file_name = get_file_name(url);
/// assert_eq!(file_name, "example.zip");
///
/// let url = "https://example.com/files/example.zip";
/// let file_name = get_file_name(url);
/// assert_eq!(file_name, "example.zip");
/// ```
///
pub fn get_file_name(url: &str) -> String {
    let last_param = url.split('/').last().unwrap_or("");
    let file_name = last_param.split('=').last().unwrap_or("");
    file_name.to_string()
}
