use serde::Serialize;
use std::fmt;

/// IPC errors contain a stable code and a safe user-facing message. Do not put
/// database paths, source errors, credentials or imported content in messages.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_error_has_stable_code_and_safe_message() {
        let error = AppError {
            code: "validation",
            message: "Check the entered value.".into(),
        };
        assert_eq!(
            serde_json::to_value(&error).unwrap(),
            serde_json::json!({"code": "validation", "message": "Check the entered value."})
        );
        assert_eq!(error.to_string(), "Check the entered value.");
    }
}
