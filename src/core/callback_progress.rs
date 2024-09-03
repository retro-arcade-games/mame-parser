use std::sync::Arc;

use crate::MameDataType;

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

/// Represents information about the progress of an ongoing operation.
///
/// This struct is used to convey the current state of progress, including the amount of work completed,
/// the total amount of work, a message describing the current status, and the type of progress update
/// being reported.
///
/// # Fields
/// - `progress`: A `u64` representing the current progress value or the amount of work completed so far.
/// - `total`: A `u64` representing the total progress value or the total amount of work expected.
/// - `message`: A `String` containing a message associated with the progress update, typically used for
///   providing additional information or context about the current operation.
/// - `callback_type`: An enum of type `CallbackType` that indicates the nature of the progress update, such as
///   `CallbackType::Progress`, `CallbackType::Info`, `CallbackType::Finish`, or `CallbackType::Error`.
///
/// # Usage
/// `ProgressInfo` is typically used in callback functions to report the status of an operation in real-time,
/// allowing the caller to monitor progress, handle errors, or perform additional actions based on the state
/// of the ongoing process.
pub struct ProgressInfo {
    /// The current progress value.
    pub progress: u64,
    /// The total progress value.
    pub total: u64,
    /// The message associated with the progress update.
    pub message: String,
    /// The type of callback being invoked.
    pub callback_type: CallbackType,
}

/// Type alias for a progress callback function used to report progress updates during long-running operations.
///
/// `ProgressCallback` is a boxed function trait object that accepts a `ProgressInfo` struct and is used to provide
/// real-time updates on the status of an ongoing operation. It allows monitoring progress, handling errors, or
/// executing additional actions based on the state of the process.
///
/// # Type
/// `ProgressCallback` is defined as:
/// ```text
/// Box<dyn Fn(ProgressInfo) + Send + 'static>
/// ```
///
/// - `ProgressInfo`: The struct containing details about the progress of the operation (current progress, total, message, callback type).
/// - `Send`: Ensures that the callback can be safely transferred across thread boundaries.
/// - `'static`: Indicates that the callback does not contain any non-static references, making it suitable for long-lived operations.
///
/// # Usage
/// This type is typically used in functions that perform asynchronous or lengthy tasks (like downloading or unpacking files)
/// and need to provide progress feedback to the caller. The callback function can be customized to handle different types of progress updates.
pub type ProgressCallback = Box<dyn Fn(ProgressInfo) + Send + 'static>;

/// Type alias for a shared progress callback function used to report progress updates across multiple threads.
///
/// `SharedProgressCallback` is a thread-safe, reference-counted function trait object that accepts a `MameDataType`
/// and a `ProgressInfo` struct. It is designed to provide real-time updates on the status of an ongoing operation
/// from multiple threads, allowing concurrent tasks to share a single callback for progress reporting.
///
/// # Type
/// `SharedProgressCallback` is defined as:
/// ```text
/// Arc<dyn Fn(MameDataType, ProgressInfo) + Send + Sync + 'static>
/// ```
///
/// - `MameDataType`: An enum indicating the type of MAME data being processed (e.g., ROMs, DAT files).
/// - `ProgressInfo`: A struct containing details about the progress of the operation (current progress, total, message, callback type).
/// - `Send`: Ensures that the callback can be safely transferred across thread boundaries.
/// - `Sync`: Ensures that the callback can be safely shared and called from multiple threads simultaneously.
/// - `'static`: Indicates that the callback does not contain any non-static references, making it suitable for long-lived and shared operations.
///
/// # Usage
/// This type is typically used in scenarios where multiple threads perform concurrent tasks (like unpacking or downloading files),
/// and a single, shared callback is needed to handle progress updates. The `Arc` wrapper allows multiple ownership of the callback,
/// ensuring it remains valid and accessible across all threads involved in the operation.
pub type SharedProgressCallback = Arc<dyn Fn(MameDataType, ProgressInfo) + Send + Sync + 'static>;
