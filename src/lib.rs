//! # mame-parser
//!
//! `mame-parser` is a Rust library that simplifies the management of files containing MAME (Multiple Arcade Machine Emulator) information.
//! The library provides functionalities to download the latest files, decompress them, read the contents, and save the data in various formats.
//!
//! ## Features
//!
//! - **File Downloading**: Download the latest MAME-related files and manage them easily.
//! - **File Decompression**: Automatically decompress downloaded files for further processing.
//! - **Data Parsing and Management**: Read and parse MAME data files, providing utilities for handling and processing the information.
//! - **Multi-format Exporting**: Save the parsed data in multiple formats, such as JSON, CSV, or SQLite.
//!
//! ## Crate Contents
//!
//! * **File Download**
//!   * [`download_file`](fn.download_file.html) - Downloads a single file and saves it to a specified location.
//!   * [`download_files`](fn.download_files.html) - Downloads multiple files concurrently with progress tracking.
//!

mod core;
mod helpers;

pub use core::fetch_unpack::file_downloader::{download_file, download_files, CallbackType};
pub use core::fetch_unpack::file_unpacker::{unpack_file, unpack_files};
pub use core::mame_data_types::MameDataType;
