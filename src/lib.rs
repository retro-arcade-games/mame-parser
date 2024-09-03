//! # mame-parser
//!
//! `mame-parser` is a Rust library that simplifies the management and processing of files containing MAME (Multiple Arcade Machine Emulator) information.
//! The library provides a suite of tools to automate the download, decompression, parsing, and exporting of MAME data,
//! making it easier to handle and manipulate this data in various formats.
//!
//! ## Features
//!
//! - **File Downloading**: Download the latest MAME-related files and store them in a specified location.
//! - **File Decompression**: Decompress downloaded files, supporting multiple archive formats like ZIP and 7z.
//! - **Data Parsing and Management**: Parse MAME data files with utilities for reading and handling the information in-memory.
//! - **Multi-format Exporting**: Export parsed data to multiple formats, such as JSON, CSV, or SQLite.
//! - **Progress Tracking**: Monitor the progress of operations.
//!
//! ## Crate Contents
//!
//! * **File Download**
//!   * [`download_file`](fn.download_file.html) - Downloads a single MAME data file to a specified location.
//!   * [`download_files`](fn.download_files.html) - Downloads multiple MAME data files concurrently, supporting progress tracking across multiple threads.
//!
//! * **File Unpacking**
//!   * [`unpack_file`](fn.unpack_file.html) - Unpacks a single downloaded file from its archive format, such as ZIP or 7z, to a specified folder.
//!   * [`unpack_files`](fn.unpack_files.html) - Unpacks multiple files concurrently, allowing efficient decompression with progress tracking.
//!
//! * **Progress Handling**
//!   * [`ProgressInfo`](struct.ProgressInfo.html) - Represents detailed progress information for ongoing operations.
//!   * [`CallbackType`](enum.CallbackType.html) - Enum defining the types of progress updates that can be reported (e.g., progress, info, finish, error).
//!   * [`ProgressCallback`](type.ProgressCallback.html) - Type alias for a callback function that reports progress updates for a single operation.
//!   * [`SharedProgressCallback`](type.SharedProgressCallback.html) - Type alias for a thread-safe callback function used across multiple operations.
//!
//! * **MAME Data Types**
//!   * [`MameDataType`](enum.MameDataType.html) - Enum representing the different types of MAME data files supported by the library.
//!

mod core;
mod helpers;

pub use core::callback_progress::{
    CallbackType, ProgressCallback, ProgressInfo, SharedProgressCallback,
};
pub use core::fetch_unpack::file_downloader::{download_file, download_files};
pub use core::fetch_unpack::file_unpacker::{unpack_file, unpack_files};
pub use core::mame_data_types::MameDataType;
