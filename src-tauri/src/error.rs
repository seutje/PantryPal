use serde::Serialize;
use std::fmt;

/// IPC errors contain a stable code and a safe user-facing message. Do not put
/// database paths, source errors, credentials or imported content in messages.
#[derive(Debug, Clone, Serialize)]
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

impl AppError {
    pub fn validation(message: &str) -> Self {
        Self {
            code: "validation",
            message: message.into(),
        }
    }
    pub fn not_found() -> Self {
        Self {
            code: "not_found",
            message: "This record is no longer available.".into(),
        }
    }
    pub fn conflict() -> Self {
        Self {
            code: "conflict",
            message:
                "This change conflicts with existing data. Refresh and check names and references."
                    .into(),
        }
    }
    pub fn storage() -> Self {
        Self { code: "storage_unavailable", message: "Local data is unavailable. Close PantryPal and retry. Your data has not been reset.".into() }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::not_found(),
            sqlx::Error::Database(ref db)
                if db.is_unique_violation()
                    || db.is_foreign_key_violation()
                    || db.is_check_violation()
                    || db.code().is_some_and(|code| code == "1811") =>
            {
                Self::conflict()
            }
            _ => Self::storage(),
        }
    }
}

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
