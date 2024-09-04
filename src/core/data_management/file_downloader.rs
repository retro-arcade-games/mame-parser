use crate::core::callback_progress::{
    CallbackType, ProgressCallback, ProgressInfo, SharedProgressCallback,
};
use crate::core::mame_data_types::{get_data_type_details, MameDataType};
use crate::helpers::data_source_helper::{get_data_source, get_file_name_from_url};
use crate::helpers::file_system_helpers::{ensure_folder_exists, WORKSPACE_PATHS};
use reqwest::blocking::Client;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

/// Downloads a specific MAME data file based on the provided data type and saves it to the workspace.
///
/// This function handles the entire process of downloading a file: it creates the destination folder if it doesn't exist,
/// retrieves the URL based on the given `MameDataType`, checks if the file already exists, and downloads the file if necessary.
/// Progress updates and messages can be provided via an optional callback function.
///
/// # Parameters
/// - `data_type`: The `MameDataType` that specifies which data file to download (e.g., ROMs, DAT files).
/// - `workspace_path`: A reference to a `Path` representing the base directory where the file will be saved.
/// - `progress_callback`: An optional callback function of type `ProgressCallback` that tracks progress and provides status updates.
///   The callback receives a `ProgressInfo` struct containing `downloaded_bytes`, `total_bytes`, `status_message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<PathBuf, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the path where the downloaded file is saved.
/// - On failure: Contains an error if the download fails, the file already exists, or there are issues accessing the URL or destination folder.
///
/// # Errors
/// This function will return an error if:
/// - The destination folder cannot be created.
/// - The URL cannot be retrieved for the given `MameDataType`.
/// - The file already exists in the destination folder.
/// - The file cannot be downloaded due to network issues or write errors.
///
/// # Callback
/// The optional progress callback function provides real-time updates on the download process and other status information. It receives:
/// - `downloaded_bytes`: The number of bytes downloaded so far.
/// - `total_bytes`: The total size of the file being downloaded (if available).
/// - `status_message`: A status message indicating the current operation (e.g., "Searching URL", "Downloading file").
/// - `callback_type`: The type of callback, typically `CallbackType::Info`, `CallbackType::Error`,`CallbackType::Progress`, or `CallbackType::Finish`.
///
/// # Example
#[doc = docify::embed!("examples/download_file.rs", main)]
///
pub fn download_file(
    data_type: MameDataType,
    workspace_path: &Path,
    progress_callback: ProgressCallback,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    // Creates a folder if it does not exist.
    let destination_folder = workspace_path.join(WORKSPACE_PATHS.download_path);
    let folder_created = ensure_folder_exists(&destination_folder);
    if let Err(err) = folder_created {
        return Err(Box::new(err));
    }

    // Retrieves the details for a given `MameDataType`
    let data_type_details = get_data_type_details(data_type);

    // Retrieves the URL for the data type.
    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Searching URL for {}", data_type_details.name),
        callback_type: CallbackType::Info,
    });

    let download_url =
        match get_data_source(&data_type_details.source, &data_type_details.source_match) {
            Ok(url) => url,
            Err(err) => {
                progress_callback(ProgressInfo {
                    progress: 0,
                    total: 0,
                    message: format!("Couldn't find URL for {}", data_type_details.name),
                    callback_type: CallbackType::Error,
                });

                return Err(err.into());
            }
        };

    // Checks if the file already exists.
    let file_name = get_file_name_from_url(&download_url);
    let file_path = destination_folder.join(file_name.clone());

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Checking if file {} already exists", file_name.clone()),
        callback_type: CallbackType::Info,
    });

    if Path::new(&file_path).exists() {
        progress_callback(ProgressInfo {
            progress: 0,
            total: 0,
            message: format!("{} already exists", file_name),
            callback_type: CallbackType::Finish,
        });

        return Ok(file_path);
    }

    // Downloads the file.
    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Downloading {} file", data_type_details.name),
        callback_type: CallbackType::Info,
    });

    download(&download_url, &destination_folder, progress_callback)
}

/// Downloads multiple files concurrently, with progress updates for each file.
///
/// This function spawns a new thread for each file to be downloaded, allowing for concurrent downloads.
/// Progress for each download is reported via a provided callback function. The function returns a list of
/// thread handles, each of which can be used to join and retrieve the result of the download operation.
///
/// # Parameters
/// - `workspace_path`: A reference to a `Path` representing the base directory where the files will be saved.
/// - `progress_callback`: A callback function of type `SharedProgressCallback` that tracks the progress of each file download. The callback
///   receives the following parameters: `(data_type, downloaded_bytes, total_bytes, status_message, callback_type)`.
///
/// # Returns
/// Returns a `Vec<thread::JoinHandle<Result<PathBuf, Box<dyn Error + Send + Sync>>>>`:
/// - Each handle represents a thread responsible for downloading a specific file. The result of the download can be accessed
///   by joining the thread handle.
/// - On success: Each thread handle contains the path where the downloaded file is saved.
/// - On failure: Each thread handle contains an error if the download fails or if there are issues saving the file.
///
/// # Callback
/// The progress callback function allows monitoring of the download process for each file. It receives:
/// - `data_type`: An enum value of `MameDataType`, indicating the type of data being downloaded.
/// - `downloaded_bytes`: The number of bytes downloaded so far.
/// - `total_bytes`: The total size of the file being downloaded (if available).
/// - `status_message`: A status message (e.g., progress or completion status).
/// - `callback_type`: The type of callback, typically `CallbackType::Progress` in this context.
///
/// # Example
#[doc = docify::embed!("examples/download_files.rs", main)]
///
pub fn download_files(
    workspace_path: &Path,
    progress_callback: SharedProgressCallback,
) -> Vec<thread::JoinHandle<Result<PathBuf, Box<dyn Error + Send + Sync>>>> {
    let progress_callback = Arc::clone(&progress_callback);

    MameDataType::all_variants()
        .iter()
        .map(|&data_type| {
            let workspace_path = workspace_path.to_path_buf();
            let progress_callback = Arc::clone(&progress_callback);

            thread::spawn(move || {
                download_file(
                    data_type,
                    &workspace_path,
                    Box::new(move |progress_info| {
                        progress_callback(data_type, progress_info);
                    }),
                )
            })
        })
        .collect()
}

/// Downloads a file from the given URL and saves it to the specified destination folder.
///
/// This function fetches the content from the provided URL, saves it to the given destination folder,
/// and optionally provides progress updates via a callback function. The function is designed to handle
/// large files by streaming the data in chunks and supports tracking download progress.
///
/// # Parameters
/// - `url`: A string slice (`&str`) representing the URL of the file to download. For example:
///   `https://example.com/file.zip`.
/// - `destination_folder`: A reference to a `Path` representing the folder where the downloaded file will be saved.
/// - `progress_callback`: A callback function of type `ProgressCallback` that tracks the progress of the download.
///   The callback receives a `ProgressInfo` struct containing `downloaded_bytes`, `total_bytes`, `status_message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<PathBuf, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the path where the downloaded file is saved.
/// - On failure: Contains an error if the download fails, the file cannot be created, or if there are issues writing to the file.
///
/// # Errors
/// This function will return an error if:
/// - The URL cannot be accessed or the download fails.
/// - The destination folder is invalid or the file cannot be created.
/// - There is an error during the reading or writing process.
///
/// # Callback
/// The progress callback function can be used to monitor the download progress in real-time. It receives:
/// - `downloaded_bytes`: The number of bytes downloaded so far.
/// - `total_bytes`: The total size of the file being downloaded (if available).
/// - `status_message`: A status message, which is currently set to an empty string during the download process and updated upon completion.
/// - `callback_type`: The type of callback, typically `CallbackType::Progress` to indicate ongoing progress, or other variants like `CallbackType::Finish` to signal completion or `CallbackType::Error` for errors.

fn download(
    url: &str,
    destination_folder: &Path,
    progress_callback: ProgressCallback,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let file_name = get_file_name_from_url(url);

    let mut response = Client::new().get(url).send()?;
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut buffer = [0; 4096];

    let file_path = destination_folder.join(file_name.clone());
    let mut file = File::create(&file_path)?;

    while let Ok(bytes_read) = response.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;

        progress_callback(ProgressInfo {
            progress: downloaded,
            total: total_size,
            message: String::from(""),
            callback_type: CallbackType::Progress,
        });
    }

    progress_callback(ProgressInfo {
        progress: downloaded,
        total: downloaded,
        message: format!("{} downloaded successfully", file_name),
        callback_type: CallbackType::Progress,
    });

    Ok(file_path)
}
