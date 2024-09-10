use crate::progress::{CallbackType, ProgressInfo};

pub fn get_progress_info(message: &str) -> ProgressInfo {
    ProgressInfo {
        progress: 0,
        total: 0,
        message: message.to_string(),
        callback_type: CallbackType::Info,
    }
}
