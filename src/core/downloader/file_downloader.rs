use reqwest::blocking::Client;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use crate::core::mame_data_types::{get_data_type_details, MameDataType};
use crate::helpers::data_source_helper::get_data_source;

const DOWNLOAD_PATH: &str = "downloads";

pub enum CallbackType {
    Info,
    Progress,
    Error,
}

fn ensure_folder_exists(path: &Path) -> io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn download_file<F>(
    data_type: MameDataType,
    workspace_path: &Path,
    progress_callback: Option<F>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>>
where
    F: Fn(u64, u64, String, CallbackType) + Send + 'static,
{
    // Creates a folder if it does not exist.
    let destination_folder = workspace_path.join(DOWNLOAD_PATH);
    let folder_created = ensure_folder_exists(&destination_folder);
    if let Err(err) = folder_created {
        return Err(Box::new(err));
    }

    // Retrieves the details for a given `MameDataType`
    let details = get_data_type_details(data_type);

    // Retrieves the URL for the data type.
    if let Some(ref callback) = progress_callback {
        let message = format!("Searching URL for {}", details.name);
        callback(0, 0, message, CallbackType::Info);
    }
    let download_url = match get_data_source(&details.source, &details.source_match) {
        Ok(url) => url,
        Err(err) => {
            if let Some(ref callback) = progress_callback {
                let message = format!("Couldn't find URL for {}", details.name);
                callback(0, 0, message, CallbackType::Error);
            }
            return Err(err.into());
        }
    };

    // Checks if the file already exists.
    let file_name = get_file_name(&download_url);
    let file_path = destination_folder.join(file_name.clone());

    if let Some(ref callback) = progress_callback {
        let message = format!("Checking if file {} already exists", file_name.clone());
        callback(0, 0, message, CallbackType::Info);
    }

    if Path::new(&file_path).exists() {
        let message = format!("File {} already exists", file_name);

        if let Some(ref callback) = progress_callback {
            callback(0, 0, message.clone(), CallbackType::Error);
        }
        return Err(Box::new(io::Error::new(
            io::ErrorKind::AlreadyExists,
            message,
        )));
    }

    // Downloads the file.
    if let Some(ref callback) = progress_callback {
        let message = format!("Downloading {} file", details.name);
        callback(0, 0, message, CallbackType::Info);
    }
    download(&download_url, &destination_folder, progress_callback)
}

pub fn download_files<F>(
    workspace_path: &Path,
    progress_callback: F,
) -> Vec<thread::JoinHandle<Result<PathBuf, Box<dyn Error + Send + Sync>>>>
where
    F: Fn(MameDataType, u64, u64, String, CallbackType) + Send + Sync + 'static,
{
    let progress_callback = Arc::new(progress_callback);

    MameDataType::all_variants()
        .iter()
        .map(|&data_type| {
            let workspace_path = workspace_path.to_path_buf();
            let progress_callback = Arc::clone(&progress_callback);

            thread::spawn(move || {
                download_file(
                    data_type,
                    &workspace_path,
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

pub fn get_file_name(url: &str) -> String {
    let last_param = url.split('/').last().unwrap_or("");
    let file_name = last_param.split('=').last().unwrap_or("");
    file_name.to_string()
}
