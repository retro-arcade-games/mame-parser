//! `mame-parser` is a Rust library that simplifies the management and processing of files containing MAME (Multiple Arcade Machine Emulator) information.
//! The library provides a suite of tools to automate the download, decompression, parsing, and exporting of MAME data,
//! making it easier to handle and manipulate this data in various formats.
//!
//! # Features
//!
//! - **File Downloading**: Download the latest MAME-related files and store them in a specified location.
//! - **File Decompression**: Decompress downloaded files, supporting multiple archive formats like ZIP and 7z.
//! - **Data Parsing and Management**: Parse MAME data files with utilities for reading and handling the information in-memory.
//! - **Multi-format Exporting**: Export parsed data to multiple formats, such as JSON, CSV, or SQLite.
//! - **Progress Tracking**: Monitor the progress of operations.
//!
//! # Crate Contents
//!
//! * [`File handling`](file_handling) - Provides functions and utilities for downloading, unpacking, reading and writing MAME data files.
//! * [`Progress tracking`](progress) - Contains tools and types for tracking and managing progress updates during operations.
//! * [`Crate models`](models) - Defines data types and models used for representing MAME data.
//! * [`Mame files readers`](readers) - Contains functions for reading and parsing different MAME data file formats.
//!
//! # Examples
//!
//! You can find examples of how to use the library in the `examples` directory of the repository.
//! Also each function in the file_handling module has its own documentation with examples.
//!
mod core;
mod helpers;

/// Module to handle the callback functions used for progress tracking.
pub use core::models::callback_progress as progress;
/// Management of MAME data files, including downloading, reading, and unpacking.
pub mod file_handling {
    pub use crate::core::data_cleanup::machine_filtering::{
        remove_machines_by_category, remove_machines_by_filter,
    };
    pub use crate::core::file_handling::file_downloader::{download_file, download_files};
    pub use crate::core::file_handling::file_reader::{read_file, read_files};
    pub use crate::core::file_handling::file_unpacker::{unpack_file, unpack_files};
    pub use crate::core::file_handling::file_writer::write_files;
}
/// Data models and types used for MAME data processing.
pub mod models {
    pub use crate::core::data_cleanup::machine_filtering::Category;
    pub use crate::core::data_cleanup::machine_filtering::MachineFilter;
    pub use crate::core::file_handling::file_writer::ExportFileType;
    pub use crate::core::models::core_models::*;
    pub use crate::core::models::mame_data_types::MameDataType;

    pub mod collections {
        pub use crate::core::models::collections_helper::get_categories_list;
        pub use crate::core::models::collections_helper::get_languages_list;
        pub use crate::core::models::collections_helper::get_manufacturers_list;
        pub use crate::core::models::collections_helper::get_players_list;
        pub use crate::core::models::collections_helper::get_series_list;
        pub use crate::core::models::collections_helper::get_subcategories_list;
    }
}

/// Module for reading and parsing MAME data files.
pub mod readers {
    pub use crate::core::readers::catver_reader::read_catver_file;
    pub use crate::core::readers::history_reader::read_history_file;
    pub use crate::core::readers::languages_reader::read_languages_file;
    pub use crate::core::readers::mame_reader::read_mame_file;
    pub use crate::core::readers::nplayers_reader::read_nplayers_file;
    pub use crate::core::readers::resources_reader::read_resources_file;
    pub use crate::core::readers::series_reader::read_series_file;
}
