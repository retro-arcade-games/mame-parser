use std::{
    collections::HashMap,
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use crate::{
    core::writers::sqlite_writer,
    helpers::file_system_helpers::{ensure_folder_exists, WORKSPACE_PATHS},
    models::Machine,
    progress::ProgressCallback,
};

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
                &data_base_path.as_path().to_string_lossy(),
                &machines,
                progress_callback,
            )?;
        }
        ExportFileType::Json => {}
        ExportFileType::Csv => {}
    }

    Ok(export_folder)
}

pub enum ExportFileType {
    Sqlite,
    Json,
    Csv,
}

impl fmt::Display for ExportFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let as_str = match self {
            ExportFileType::Sqlite => "sqlite",
            ExportFileType::Json => "json",
            ExportFileType::Csv => "csv",
        };
        write!(f, "{}", as_str)
    }
}
