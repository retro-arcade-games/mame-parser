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

/// Represents a callback function that can be invoked during an operation.
pub type ProgressCallback = Box<dyn Fn(ProgressInfo) + Send + 'static>;

/// Represents a shared callback function that can be invoked during an operation.
pub type SharedProgressCallback = Arc<dyn Fn(MameDataType, ProgressInfo) + Send + Sync + 'static>;
