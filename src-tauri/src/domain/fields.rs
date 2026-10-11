use super::validation;
use crate::error::{AppError, AppResult};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    Boolean,
    Select,
    Number,
    Date,
    Text,
    Rating,
}

impl FieldKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Select => "select",
            Self::Number => "number",
            Self::Date => "date",
            Self::Text => "text",
            Self::Rating => "rating",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FieldOption {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FieldInput {
    pub label: String,
    pub kind: FieldKind,
    pub description: String,
    pub options: Vec<FieldOption>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub unit_label: Option<String>,
    pub sort_order: i64,
}

impl FieldInput {
    pub fn validate(&self) -> AppResult<()> {
        validation::text(&self.label, 120, true)?;
        validation::text(&self.description, 2_000, false)?;
        if let Some(unit) = &self.unit_label {
            validation::text(unit, 100, false)?;
        }
        if self.sort_order < 0 || self.options.len() > 100 {
            return Err(AppError::validation("Invalid field order or options."));
        }
        let mut ids = HashSet::new();
        let mut labels = HashSet::new();
        for option in &self.options {
            validation::id(&option.id)?;
            validation::text(&option.label, 120, true)?;
            if !ids.insert(&option.id) || !labels.insert(validation::normalized(&option.label)) {
                return Err(AppError::validation("Field options must be distinct."));
            }
        }
        if (self.kind != FieldKind::Select && !self.options.is_empty())
            || (self.kind == FieldKind::Select && self.options.is_empty())
        {
            return Err(AppError::validation(
                "Select fields require options; other field kinds cannot have options.",
            ));
        }
        let numeric = matches!(self.kind, FieldKind::Number | FieldKind::Rating);
        if (!numeric
            && (self.minimum.is_some() || self.maximum.is_some() || self.unit_label.is_some()))
            || self.minimum.is_some_and(|v| !v.is_finite())
            || self.maximum.is_some_and(|v| !v.is_finite())
            || matches!((self.minimum,self.maximum),(Some(a),Some(b)) if a>b)
        {
            return Err(AppError::validation("Invalid field bounds."));
        }
        if self.kind == FieldKind::Rating
            && (self.minimum.is_none()
                || self.maximum.is_none()
                || self.minimum.is_some_and(|v| v.fract() != 0.0)
                || self.maximum.is_some_and(|v| v.fract() != 0.0))
        {
            return Err(AppError::validation(
                "Rating fields require integer bounds.",
            ));
        }
        Ok(())
    }

    pub fn validate_value(&self, value: &Option<FieldValue>) -> AppResult<()> {
        let Some(value) = value else {
            return Ok(());
        };
        let valid = match (&self.kind, value) {
            (FieldKind::Boolean, FieldValue::Boolean(_)) => true,
            (FieldKind::Select, FieldValue::Select(id)) => self.options.iter().any(|o| o.id == *id),
            (FieldKind::Number, FieldValue::Number(v))
            | (FieldKind::Rating, FieldValue::Rating(v)) => {
                v.is_finite()
                    && self.minimum.is_none_or(|min| *v >= min)
                    && self.maximum.is_none_or(|max| *v <= max)
                    && (self.kind != FieldKind::Rating || v.fract() == 0.0)
            }
            (FieldKind::Date, FieldValue::Date(v)) => NaiveDate::parse_from_str(v, "%Y-%m-%d")
                .is_ok_and(|d| d.format("%Y-%m-%d").to_string() == *v),
            (FieldKind::Text, FieldValue::Text(v)) => validation::text(v, 10_000, false).is_ok(),
            _ => false,
        };
        if !valid {
            return Err(AppError::validation(
                "The value does not match its custom field definition.",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum FieldValue {
    Boolean(bool),
    Select(String),
    Number(f64),
    Date(String),
    Text(String),
    Rating(f64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDefinition {
    pub id: String,
    #[serde(flatten)]
    pub data: FieldInput,
    pub retired: bool,
}
