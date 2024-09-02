use sevenz_rust::Password;
use zip::ZipArchive;

use crate::core::mame_data_types::get_data_type_details;
use crate::helpers::file_system_helpers::{
    ensure_folder_exists, find_file_with_pattern, WORKSPACE_PATHS,
};
use crate::{CallbackType, MameDataType, ProgressCallback, ProgressInfo, SharedProgressCallback};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::{fs::File, io::Write};

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

    // Check if file already unpacked

    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!(
            "Checking if {} file already unpacked",
            data_type_details.name
        ),
        callback_type: CallbackType::Info,
    });

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

    // Check if zip file is present
    progress_callback(ProgressInfo {
        progress: 0,
        total: 0,
        message: format!("Checking if {} zip file exists", data_type_details.name),
        callback_type: CallbackType::Info,
    });

    let download_folder = workspace_path.join(WORKSPACE_PATHS.download_path);
    let zip_file_path = find_file_with_pattern(
        &download_folder.to_str().unwrap(),
        &data_type_details.zip_file_pattern,
    );

    match zip_file_path {
        // Unpack the file
        Ok(zip_file_path) => {
            let zip_file = zip_file_path.split('/').last().unwrap();
            progress_callback(ProgressInfo {
                progress: 0,
                total: 0,
                message: format!("Unpacking {}", zip_file),
                callback_type: CallbackType::Info,
            });

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
