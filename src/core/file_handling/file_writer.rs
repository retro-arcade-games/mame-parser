use crate::{
    core::writers::{csv_writer, json_writer, sqlite_writer},
    helpers::file_system_helpers::{ensure_folder_exists, WORKSPACE_PATHS},
    models::Machine,
    progress::ProgressCallback,
};
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

/// Writes machine data to the specified export file type.
///
/// This function handles the export of machine data to the chosen format (`SQLite`, `JSON`, or `CSV`)
/// by creating the necessary export folder in the workspace path and invoking the appropriate writer function.
/// It ensures that the target directory exists, then delegates the writing task to the relevant module
/// based on the selected `ExportFileType`. Progress updates and messages are provided via a callback function.
///
/// # Parameters
/// - `export_file_type`: An `ExportFileType` enum specifying the format for data export. Supported types are:
///   - `ExportFileType::Sqlite`: Exports data to a SQLite database file.
///   - `ExportFileType::Json`: Exports data to a JSON file.
///   - `ExportFileType::Csv`: Exports data to a CSV file.
/// - `workspace_path`: A reference to a `Path` representing the base directory where the exported files will be stored.
/// - `machines`: A reference to a `HashMap` where keys are machine names and values are `Machine` structs containing
///   detailed information about each MAME machine.
/// - `progress_callback`: A callback function of type `ProgressCallback` that provides status updates and progress
///   information during the export process. The callback receives a `ProgressInfo` struct containing `progress`, `total`,
///   `message`, and `callback_type`.
///
/// # Returns
/// Returns a `Result<PathBuf, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains a `PathBuf` representing the path to the folder where the export files are stored.
/// - On failure: Contains an error if the export folder cannot be created or if there is an issue during the writing process.
///
/// # Errors
/// This function will return an error if:
/// - The export folder cannot be created due to permission issues or file system errors.
/// - The writing process fails for the selected export file type due to data formatting issues or I/O errors.
///
/// # Callback
/// The progress callback function provides real-time updates on the export process. It receives:
/// - `progress`: The current progress of the export operation (e.g., number of records processed).
/// - `total`: The total number of items to be exported.
/// - `message`: A status message indicating the current operation (e.g., "Creating export folder", "Writing to file").
/// - `callback_type`: The type of callback, such as `CallbackType::Info`, `CallbackType::Error`, `CallbackType::Progress`, or `CallbackType::Finish`.
///
/// # Example Sqlite
#[doc = docify::embed!("examples/write_sqlite.rs", main)]
///
/// # Example Csv
#[doc = docify::embed!("examples/write_csv.rs", main)]
///
/// # Example Json
#[doc = docify::embed!("examples/write_json.rs", main)]
///
pub fn write_files(
    export_file_type: ExportFileType,
    workspace_path: &Path,
    machines: &HashMap<String, Machine>,
    progress_callback: ProgressCallback,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let export_folder = workspace_path
        .join(WORKSPACE_PATHS.export_path)
        .join(export_file_type.to_string().to_lowercase());

    let folder_created = ensure_folder_exists(&export_folder);
    if let Err(err) = folder_created {
        return Err(Box::new(err));
    }

    match export_file_type {
        ExportFileType::Sqlite => {
            let data_base_path = export_folder.join("machines.db");
            sqlite_writer::write_sqlite(
                &data_base_path.to_string_lossy(),
                &machines,
                progress_callback,
            )?;
        }
        ExportFileType::Json => {
            json_writer::write_json(
                &export_folder.to_string_lossy(),
                &machines,
                progress_callback,
            )?;
        }
        ExportFileType::Csv => {
            csv_writer::write_csv(
                &export_folder.to_string_lossy(),
                &machines,
                progress_callback,
            )?;
        }
    }

    Ok(export_folder)
}

/// Represents the file type to be used for data export.
///
/// The `ExportFileType` enum defines the different formats supported for exporting data,
/// allowing the caller to choose between database, structured data, and tabular formats.
/// This is particularly useful in scenarios where data needs to be shared, stored, or analyzed
/// in various ways.
///
/// # Variants
/// - `Sqlite`: Exports the data to a SQLite database file, suitable for structured storage and complex queries.
/// - `Json`: Exports the data to a JSON (JavaScript Object Notation) file, ideal for web applications and data interchange.
/// - `Csv`: Exports the data to a CSV (Comma-Separated Values) file, useful for spreadsheet applications and basic data analysis.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFileType {
    /// Exports data to a SQLite database file.
    Sqlite,
    /// Exports data to a JSON file.
    Json,
    /// Exports data to a CSV file.
    Csv,
}

/// Implements the `fmt::Display` trait for `ExportFileType`.
///
/// This allows instances of `ExportFileType` to be formatted as strings,
/// making it easy to display or print the enum values in a user-friendly way.
impl fmt::Display for ExportFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Match each variant to its corresponding string representation
        let as_str = match self {
            ExportFileType::Sqlite => "sqlite",
            ExportFileType::Json => "json",
            ExportFileType::Csv => "csv",
        };
        // Write the string representation to the formatter
        write!(f, "{}", as_str)
    }
}
