use super::validation;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngredientInput {
    pub name: String,
    pub category_id: Option<String>,
    pub section_id: Option<String>,
    pub preferred_unit_id: Option<String>,
    pub aliases: Vec<String>,
    /// Empty is unspecified; all twelve months means explicitly year-round.
    pub season_months: Vec<u8>,
    pub notes: String,
}

impl IngredientInput {
    pub fn validate(&self) -> AppResult<()> {
        validation::text(&self.name, 120, true)?;
        validation::text(&self.notes, 10_000, false)?;
        validation::optional_id(&self.category_id)?;
        validation::optional_id(&self.section_id)?;
        validation::optional_id(&self.preferred_unit_id)?;
        if self.aliases.len() > 50 || self.season_months.len() > 12 {
            return Err(AppError::validation("Too many aliases or season months."));
        }
        let mut names = HashSet::from([validation::normalized(&self.name)]);
        for alias in &self.aliases {
            validation::text(alias, 120, true)?;
            if !names.insert(validation::normalized(alias)) {
                return Err(AppError::validation("Names and aliases must be distinct."));
            }
        }
        let mut months = HashSet::new();
        for month in &self.season_months {
            if !(1..=12).contains(month) || !months.insert(month) {
                return Err(AppError::validation(
                    "Season months must be distinct months from 1 to 12.",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Ingredient {
    pub id: String,
    #[serde(flatten)]
    pub data: IngredientInput,
    pub retired: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Vocabulary {
    pub id: String,
    pub name: String,
    pub sort_order: i64,
    pub retired: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VocabularyKind {
    Category,
    Section,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    pub id: String,
    pub label: String,
    pub symbol: String,
    pub dimension: String,
    pub system: String,
    pub factor_numerator: Option<i64>,
    pub factor_denominator: Option<i64>,
    pub offset_numerator: Option<i64>,
    pub offset_denominator: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub ingredients: Vec<Ingredient>,
    pub categories: Vec<Vocabulary>,
    pub sections: Vec<Vocabulary>,
    pub units: Vec<Unit>,
}
