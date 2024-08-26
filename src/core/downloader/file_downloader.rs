use reqwest::blocking::Client;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::core::mame_data_types::{get_data_type_details, MameDataType};
use crate::helpers::data_source_helper::get_data_source;

pub fn download_file(
    data_type: MameDataType,
    destination_folder: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    download_file_callback(data_type, destination_folder, None::<fn(u64, u64)>)
}

pub fn download_file_callback(
    data_type: MameDataType,
    destination_folder: &Path,
    progress_callback: Option<impl Fn(u64, u64) + Send + 'static>,
) -> Result<PathBuf, Box<dyn Error>> {
    let details = get_data_type_details(data_type);

    let download_url = get_data_source(&details.source, &details.source_match)?;

    download(&download_url, destination_folder, progress_callback)
}

fn download<F>(
    url: &str,
    destination_folder: &Path,
    progress_callback: Option<F>,
) -> Result<PathBuf, Box<dyn Error>>
where
    F: Fn(u64, u64) + Send + 'static,
{
    let file_name = get_file_name(url);

    let mut response = Client::new().get(url).send()?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut buffer = [0; 4096];

    let file_path = destination_folder.join(file_name);
    let mut file = File::create(&file_path)?;

    loop {
        let bytes_read = response.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;

        if let Some(ref callback) = progress_callback {
            callback(downloaded, total_size);
        }
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
