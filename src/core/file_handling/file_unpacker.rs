use crate::helpers::file_system_helpers::{
    ensure_folder_exists, find_file_with_pattern, WORKSPACE_PATHS,
};
use crate::{
    core::models::{
        callback_progress::{CallbackType, ProgressCallback, ProgressInfo, SharedProgressCallback},
        mame_data_types::{get_data_type_details, MameDataType},
    },
    helpers::callback_progress_helper::get_progress_info,
};
use sevenz_rust::Password;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::{fs::File, io::Write};
use zip::ZipArchive;

/// Unpacks a data file for a specific `MameDataType` into a designated workspace folder.
///
/// This function checks if the required data file for the specified `MameDataType` is already unpacked.
/// If not, it searches for the corresponding ZIP file in the download directory, and if found,
/// unpacks it into the appropriate folder. Progress updates during the process can be provided via a callback function.
///
/// # Parameters
/// - `data_type`: The `MameDataType` that specifies the type of data file to unpack (e.g., Series, Categories).
/// - `workspace_path`: A reference to a `Path` representing the base directory where the data file will be unpacked.
/// - `progress_callback`: A callback function of type `ProgressCallback` that provides progress updates during the unpacking process.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<PathBuf, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the path where the unpacked file is located.
/// - On failure: Contains an error if the file cannot be unpacked, if the ZIP file is not found,
///   or if there are issues creating the destination folder.
///
/// # Errors
/// This function will return an error if:
/// - The destination folder cannot be created.
/// - The required ZIP file is not found in the download folder.
/// - The unpacking process fails due to reading or writing errors.
///
/// # Callback
/// The progress callback function can be used to monitor the unpacking process in real-time. It receives:
/// - `progress`: The number of entries processed so far.
/// - `total`: The total entries of the file being unpacked.
/// - `message`: A status message indicating the current operation (e.g., "Unpacking file", "Checking if file already unpacked").
/// - `callback_type`: The type of callback, typically `CallbackType::Progress` for ongoing updates, `CallbackType::Info` for informational messages, `CallbackType::Finish` for completion, or `CallbackType::Error` for errors.
///
/// # Example
#[doc = docify::embed!("examples/unpack_file.rs", main)]
///
pub fn unpack_file(
    data_type: MameDataType,
    workspace_path: &Path,
    progress_callback: ProgressCallback,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    // Retrieves the details for a given `MameDataType`
    let data_type_details = get_data_type_details(data_type);

    // Creates a folder if it does not exist.
    let extract_folder = workspace_path
        .join(WORKSPACE_PATHS.extract_path)
        .join(data_type_details.name.to_lowercase());

    let folder_created = ensure_folder_exists(&extract_folder);
    if let Err(err) = folder_created {
        return Err(Box::new(err));
    }

    // Checks if file already unpacked
    progress_callback(get_progress_info(
        format!(
            "Checking if {} file already unpacked",
            data_type_details.name
        )
        .as_str(),
    ));

    if let Ok(existing_data_file) = find_file_with_pattern(
        &extract_folder.to_str().unwrap(),
        &data_type_details.data_file_pattern,
    ) {
        progress_callback(ProgressInfo {
            progress: 0,
            total: 0,
            message: format!("{} file already unpacked", data_type_details.name),
            callback_type: CallbackType::Finish,
        });

        return Ok(existing_data_file.into());
    }

    // Checks if zip file is present
    progress_callback(get_progress_info(
        format!("Checking if {} zip file exists", data_type_details.name).as_str(),
    ));

    let download_folder = workspace_path.join(WORKSPACE_PATHS.download_path);
    let zip_file_path = find_file_with_pattern(
        &download_folder.to_str().unwrap(),
        &data_type_details.zip_file_pattern,
    );

    match zip_file_path {
        // Unpack the file
        Ok(zip_file_path) => {
            let zip_file = zip_file_path.split('/').last().unwrap();

            progress_callback(get_progress_info(
                format!("Unpacking {}", zip_file).as_str(),
            ));

            let unpack_result = unpack(&zip_file_path, &extract_folder, &progress_callback);

            // Check if unpacking was successful
            match unpack_result {
                Ok(_) => {
                    if let Ok(existing_data_file) = find_file_with_pattern(
                        &extract_folder.to_str().unwrap(),
                        &data_type_details.data_file_pattern,
                    ) {
                        return Ok(existing_data_file.into());
                    } else {
                        let message = format!(
                            "{} data file not present after unpacking",
                            data_type_details.name
                        );
                        progress_callback(ProgressInfo {
                            progress: 0,
                            total: 0,
                            message: message.clone(),
                            callback_type: CallbackType::Error,
                        });

                        return Err(message.into());
                    }
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        }
        Err(err) => {
            let message = format!("{} zip file not found", data_type_details.name);

            progress_callback(ProgressInfo {
                progress: 0,
                total: 0,
                message: message.clone(),
                callback_type: CallbackType::Error,
            });

            return Err(err.into());
        }
    }
}

/// Unpacks multiple data files concurrently for all `MameDataType` variants into a designated workspace folder.
///
/// This function spawns a new thread for each data type to unpack its respective file, allowing for concurrent unpacking operations.
/// Progress for each unpacking operation is reported via a provided shared callback function. The function returns a list of
/// thread handles, each of which can be used to join and retrieve the result of the unpacking operation.
///
/// # Parameters
/// - `workspace_path`: A reference to a `Path` representing the base directory where the data files will be unpacked.
/// - `progress_callback`: A shared callback function of type `SharedProgressCallback` that tracks the progress of each file unpacking operation.
///   The callback receives the `data_type` and a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Vec<thread::JoinHandle<Result<PathBuf, Box<dyn Error + Send + Sync>>>>`:
/// - Each handle represents a thread responsible for unpacking a specific file. The result of the unpacking can be accessed by joining the thread handle.
/// - On success: Each thread handle contains the path where the unpacked file is located.
/// - On failure: Each thread handle contains an error if the unpacking fails or if there are issues accessing or creating the destination folder.
///
/// # Errors
/// This function does not directly return errors, but errors may be encountered and reported through the thread handles.
/// The following errors might occur during the unpacking process:
/// - The destination folder cannot be created.
/// - The required ZIP file is not found in the download folder.
/// - The unpacking process fails due to reading or writing errors.
///
/// # Callback
/// The shared progress callback function can be used to monitor the unpacking process of each file in real-time. It receives:
/// - `data_type`: An enum value of `MameDataType`, indicating the type of data being unpacked.
/// - `progress`: The number of entries processed so far.
/// - `total`: The total entries of the file being unpacked (if available).
/// - `message`: A status message indicating the current operation (e.g., "Unpacking file", "Checking if file already unpacked").
/// - `callback_type`: The type of callback, typically `CallbackType::Progress` for ongoing updates, `CallbackType::Info` for informational messages, `CallbackType::Finish` for completion, or `CallbackType::Error` for errors.
///
/// # Example
#[doc = docify::embed!("examples/unpack_files.rs", main)]
///
pub fn unpack_files(
    workspace_path: &Path,
    progress_callback: SharedProgressCallback,
) -> Vec<thread::JoinHandle<Result<PathBuf, Box<dyn Error + Send + Sync>>>> {
    let progress_callback = Arc::new(progress_callback);

    MameDataType::all_variants()
        .iter()
        .map(|&data_type| {
            let workspace_path = workspace_path.to_path_buf();
            let progress_callback = Arc::clone(&progress_callback);

            thread::spawn(move || {
                unpack_file(
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

/// Unpacks an archive file (ZIP or 7z) to the specified destination folder.
///
/// This function determines the type of archive file based on its extension (`.zip` or `.7z`)
/// and calls the appropriate extraction function to unpack its contents into the provided folder.
/// Progress updates during the unpacking process can be provided via a callback function.
///
/// # Parameters
/// - `zip_file_path`: A string slice (`&str`) representing the path to the archive file to be unpacked.
///   The file must have a `.zip` or `.7z` extension.
/// - `extract_folder`: A reference to a `Path` representing the destination folder where the contents of the archive will be extracted.
/// - `progress_callback`: A reference to a callback function of type `ProgressCallback` that provides progress updates during the unpacking process.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<PathBuf, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the path to the extracted contents in the destination folder.
/// - On failure: Contains an error if the file cannot be unpacked due to an unsupported format, reading or writing errors, or issues accessing the destination folder.
///
/// # Errors
/// This function will return an error if:
/// - The archive format is unsupported (i.e., the file does not have a `.zip` or `.7z` extension).
/// - The destination folder is invalid or inaccessible.
/// - The extraction process fails due to reading or writing errors.
fn unpack(
    zip_file_path: &str,
    extract_folder: &Path,
    progress_callback: &ProgressCallback,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    match zip_file_path {
        path if path.ends_with(".zip") => {
            return extract_zip(
                zip_file_path,
                extract_folder.to_str().unwrap(),
                progress_callback,
            );
        }
        path if path.ends_with(".7z") => {
            return extract_7zip(
                zip_file_path,
                extract_folder.to_str().unwrap(),
                progress_callback,
            );
        }
        _ => return Err("Unsupported archive format".into()),
    }
}

/// Extracts the contents of a ZIP archive to the specified destination folder.
///
/// This function opens a ZIP file, iterates over its contents, and extracts each file or directory
/// to the specified destination folder. It provides real-time progress updates via a callback function
/// during the extraction process.
///
/// # Parameters
/// - `archive_path`: A string slice (`&str`) representing the path to the ZIP archive file to be extracted.
/// - `destination_folder`: A string slice (`&str`) representing the destination folder where the contents of the archive will be extracted.
/// - `progress_callback`: A reference to a callback function of type `ProgressCallback` that provides progress updates during the extraction process.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<PathBuf, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the path to the folder where the contents were extracted.
/// - On failure: Contains an error if the extraction process fails due to reading errors, writing errors, or issues accessing the destination folder.
///
/// # Errors
/// This function will return an error if:
/// - The ZIP archive cannot be opened or read.
/// - The destination folder cannot be created or is invalid.
/// - There are errors during file extraction, such as reading from the archive or writing to the disk.
fn extract_zip(
    archive_path: &str,
    destination_folder: &str,
    progress_callback: &ProgressCallback,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    let total_files = archive.len() as u64;
    let mut progress: u64 = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let output_path = Path::new(destination_folder).join(file.name());

        if (file.name()).ends_with('/') {
            std::fs::create_dir_all(&output_path)?;
        } else {
            if let Some(p) = output_path.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
            }
            let mut output_file = File::create(&output_path)?;
            std::io::copy(&mut file, &mut output_file)?;
        }

        progress += 1;

        progress_callback(ProgressInfo {
            progress,
            total: total_files,
            message: String::from(""),
            callback_type: CallbackType::Progress,
        });
    }

    let zip_file = archive_path.split('/').last().unwrap();
    progress_callback(ProgressInfo {
        progress,
        total: progress,
        message: format!("{} unpacked successfully", zip_file),
        callback_type: CallbackType::Finish,
    });

    Ok(destination_folder.into())
}

/// Extracts the contents of a 7z archive to the specified destination folder.
///
/// This function opens a 7z archive file, iterates over its contents, and extracts each file or directory
/// to the specified destination folder. Progress updates during the extraction process can be provided
/// via a callback function.
///
/// # Parameters
/// - `archive_path`: A string slice (`&str`) representing the path to the 7z archive file to be extracted.
/// - `destination_folder`: A string slice (`&str`) representing the destination folder where the contents of the archive will be extracted.
/// - `progress_callback`: A reference to a callback function of type `ProgressCallback` that provides progress updates during the extraction process.
///   The callback receives a `ProgressInfo` struct containing `progress`, `total`, `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<PathBuf, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the path to the folder where the contents were extracted.
/// - On failure: Contains an error if the extraction process fails due to reading errors, writing errors, or issues accessing the destination folder.
///
/// # Errors
/// This function will return an error if:
/// - The 7z archive cannot be opened or read.
/// - The destination folder cannot be created or is invalid.
/// - There are errors during file extraction, such as reading from the archive or writing to the disk.
/// - The provided 7z archive format is unsupported or corrupted.
fn extract_7zip(
    archive_path: &str,
    destination_folder: &str,
    progress_callback: &ProgressCallback,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let mut sz = sevenz_rust::SevenZReader::open(archive_path, Password::empty()).unwrap();

    let total_files = sz.archive().files.len();
    let mut progress_entries: u64 = 0;

    let dest = PathBuf::from(destination_folder);

    sz.for_each_entries(|entry, reader| {
        let mut buf = [0u8; 1024];
        let path = dest.join(entry.name());
        if entry.is_directory() {
            std::fs::create_dir_all(path).unwrap();
            return Ok(true);
        }
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = File::create(path).unwrap();
        loop {
            let read_size = reader.read(&mut buf)?;
            if read_size == 0 {
                progress_entries += 1;

                progress_callback(ProgressInfo {
                    progress: progress_entries,
                    total: total_files as u64,
                    message: String::from(""),
                    callback_type: CallbackType::Progress,
                });

                break Ok(true);
            }
            file.write_all(&buf[..read_size])?;
        }
    })
    .unwrap();

    let zip_file = archive_path.split('/').last().unwrap();
    progress_callback(ProgressInfo {
        progress: progress_entries,
        total: progress_entries,
        message: format!("{} unpacked successfully", zip_file),
        callback_type: CallbackType::Finish,
    });

    Ok(destination_folder.into())
}
