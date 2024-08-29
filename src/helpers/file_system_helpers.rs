use std::error::Error;
use std::fs::{self};
use std::io::{self};
use std::path::Path;

/// Ensures that the specified folder exists, creating it if necessary.
///
/// This function checks whether the provided path exists, and if it does not, attempts to create
/// the folder and any necessary parent directories. It is a utility function to guarantee that
/// a folder is available for file operations, such as downloads or data storage.
///
/// # Parameters
/// - `path`: A reference to a `Path` representing the folder path to check or create. For example:
///   `/path/to/folder`.
///
/// # Returns
/// Returns an `io::Result<()>`:
/// - On success: Returns `Ok(())` indicating that the folder exists or was successfully created.
/// - On failure: Returns an `io::Error` if the folder could not be created due to issues such as
///   insufficient permissions or an invalid path.
///
/// # Errors
/// This function will return an error if:
/// - The path cannot be created due to filesystem issues (e.g., permission denied).
/// - The provided path is invalid or contains unsupported characters.
///
pub(crate) fn ensure_folder_exists(path: &Path) -> io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub(crate) fn find_file_with_pattern(
    folder: &str,
    pattern: &regex::Regex,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    for entry in walkdir::WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                if pattern.is_match(file_name) {
                    return Ok(path.to_string_lossy().into_owned());
                }
            }
        }
    }
    let error_message = format!(
        "No matching file with pattern {} found in {}",
        pattern.as_str(),
        folder
    );
    Err(error_message.into())
}

pub(crate) struct WorkspacePaths {
    pub download_path: &'static str,
    pub extract_path: &'static str,
}

pub(crate) const WORKSPACE_PATHS: WorkspacePaths = WorkspacePaths {
    download_path: "downloads",
    extract_path: "extracted",
};
