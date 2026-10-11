use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

pub fn normalized(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn text(value: &str, maximum: usize, required: bool) -> AppResult<()> {
    if value.chars().count() > maximum
        || value.contains('\0')
        || (required && value.trim().is_empty())
    {
        return Err(AppError::validation(
            "Check the text length and required fields.",
        ));
    }
    Ok(())
}

pub fn id(value: &str) -> AppResult<()> {
    if value.is_empty()
        || value.len() > 100
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    {
        return Err(AppError::validation("Invalid record identifier."));
    }
    Ok(())
}

pub fn optional_id(value: &Option<String>) -> AppResult<()> {
    if let Some(value) = value {
        id(value)?;
    }
    Ok(())
}

/// Exact stored amount, not a conversion engine. Keep the entered pair intact.
/// Bounds keep every IPC integer exactly representable in JavaScript and leave
/// headroom for checked arithmetic in the Phase 2 engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Amount {
    pub numerator: i64,
    pub denominator: i64,
}

impl Amount {
    pub fn validate(&self, positive: bool) -> AppResult<()> {
        if self.numerator < i64::from(positive)
            || self.numerator > 1_000_000_000
            || self.denominator < 1
            || self.denominator > 1_000_000_000
        {
            return Err(AppError::validation(
                "Enter a valid amount with a positive denominator.",
            ));
        }
        Ok(())
    }
}
