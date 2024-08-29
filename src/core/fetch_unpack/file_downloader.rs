use reqwest::blocking::Client;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use crate::core::mame_data_types::{get_data_type_details, MameDataType};
use crate::helpers::data_source_helper::{get_data_source, get_file_name_from_url};
use crate::helpers::file_system_helpers::{ensure_folder_exists, WORKSPACE_PATHS};

/// Represents the type of callback being invoked during an operation.
///
/// The `CallbackType` enum is used to categorize the nature of the callback, allowing the caller
/// to differentiate between informational messages, progress updates, and errors. This is particularly
/// useful in scenarios where different types of feedback need to be handled in distinct ways.
///
/// # Variants
/// - `Info`: Indicates a general informational message, such as status updates or non-critical notifications.
/// - `Progress`: Indicates that the callback is providing progress updates, typically involving downloaded bytes or percentages.
/// - `Finish`: Indicates that an operation has completed successfully, providing a final status message.
/// - `Error`: Indicates that an error has occurred and provides details related to the issue.
///
#[derive(Debug)]
pub enum CallbackType {
    /// Conveys a general informational message.
    Info,
    /// Indicates that progress information is being reported (e.g., download progress).
    Progress,
    /// Indicates that an operation has finished successfully.
    Finish,
    /// Signals that an error has occurred and provides error details.
    Error,
}

/// Downloads a specific MAME data file based on the provided data type and saves it to the workspace.
///
/// This function handles the entire process of downloading a file: it creates the destination folder if it doesn't exist,
/// retrieves the URL based on the given `MameDataType`, checks if the file already exists, and downloads the file if necessary.
/// Progress updates and messages can be provided via an optional callback function.
///
/// # Parameters
/// - `data_type`: The `MameDataType` that specifies which data file to download (e.g., ROMs, DAT files).
/// - `workspace_path`: A reference to a `Path` representing the base directory where the file will be saved.
/// - `progress_callback`: An optional callback function of type `F` that tracks progress and provides status updates. The callback
///   receives the following parameters: `(downloaded_bytes, total_bytes, status_message, callback_type)`.
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
/// ```rust
/// use mame_parser::{download_file, CallbackType, MameDataType};
/// use std::path::Path;
///
/// fn progress_callback(downloaded: u64, total: u64, message: String, callback_type: CallbackType) {
///     println!("{} ({} / {}) - {:?}", message, downloaded, total, callback_type);
/// }
///
/// let workspace_path = Path::new("playground");
/// let result = download_file(MameDataType::Series, workspace_path, Some(progress_callback));
///
/// match result {
///     Ok(path) => println!("File saved to: {:?}", path),
///     Err(e) => println!("Download failed: {}", e),
/// }
/// ```
///
pub fn download_file<F>(
    data_type: MameDataType,
    workspace_path: &Path,
    progress_callback: Option<F>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>>
where
    F: Fn(u64, u64, String, CallbackType) + Send + 'static,
{
    // Creates a folder if it does not exist.
    let destination_folder = workspace_path.join(WORKSPACE_PATHS.download_path);
    let folder_created = ensure_folder_exists(&destination_folder);
    if let Err(err) = folder_created {
        return Err(Box::new(err));
    }

    // Retrieves the details for a given `MameDataType`
    let data_type_details = get_data_type_details(data_type);

    // Retrieves the URL for the data type.
    if let Some(ref callback) = progress_callback {
        let message = format!("Searching URL for {}", data_type_details.name);
        callback(0, 0, message, CallbackType::Info);
    }
    let download_url =
        match get_data_source(&data_type_details.source, &data_type_details.source_match) {
            Ok(url) => url,
            Err(err) => {
                if let Some(ref callback) = progress_callback {
                    let message = format!("Couldn't find URL for {}", data_type_details.name);
                    callback(0, 0, message, CallbackType::Error);
                }
                return Err(err.into());
            }
        };

    // Checks if the file already exists.
    let file_name = get_file_name_from_url(&download_url);
    let file_path = destination_folder.join(file_name.clone());

    if let Some(ref callback) = progress_callback {
        let message = format!("Checking if file {} already exists", file_name.clone());
        callback(0, 0, message, CallbackType::Info);
    }

    if Path::new(&file_path).exists() {
        let message = format!("{} already exists", file_name);

        if let Some(ref callback) = progress_callback {
            callback(0, 0, message.clone(), CallbackType::Finish);
        }
        return Ok(file_path);
    }

    // Downloads the file.
    if let Some(ref callback) = progress_callback {
        let message = format!("Downloading {} file", data_type_details.name);
        callback(0, 0, message, CallbackType::Info);
    }
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
/// - `progress_callback`: A callback function of type `F` that tracks the progress of each file download. The callback
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
/// ```rust
/// use mame_parser::{download_files, CallbackType, MameDataType};
/// use std::path::Path;
/// use std::thread;
///
/// fn progress_callback(
///     data_type: MameDataType,
///     downloaded: u64,
///     total: u64,
///     _status: String,
///     _callback_type: CallbackType,
/// ) {
///     println!(
///         "Downloading {:?}: {} of {} bytes",
///         data_type, downloaded, total
///     );
/// }
///
/// let workspace_path = Path::new("playground");
/// let handles = download_files(workspace_path, progress_callback);
///
/// for handle in handles {
///     match handle.join() {
///         Ok(result) => match result {
///             Ok(path) => println!("File saved to: {:?}", path),
///             Err(e) => println!("Download failed: {}", e),
///         },
///         Err(_) => println!("Thread panicked during download"),
///     }
/// }
/// ```
///
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
/// - `progress_callback`: An optional callback function of type `F` that tracks the progress of the download. The callback
///   receives the following parameters: `(downloaded_bytes, total_bytes, status_message, callback_type)`.
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
/// - `status_message`: A status message, which is currently set to an empty string.
/// - `callback_type`: The type of callback, typically `CallbackType::Progress` in this context.
///
fn download<F>(
    url: &str,
    destination_folder: &Path,
    progress_callback: Option<F>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>>
where
    F: Fn(u64, u64, String, CallbackType) + Send + 'static,
{
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
        let message = format!("{} downloaded successfully", file_name);
        callback(downloaded, downloaded, message, CallbackType::Finish);
    }

    Ok(file_path)
}
