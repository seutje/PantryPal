//! Pure adapters produce untrusted, reviewable drafts and never persist meals.
mod json_ld;
pub mod sites;
#[cfg(test)]
mod tests;
mod units;
pub mod url;
use crate::{
    domain::{meal_library::MealDraft, validation},
    error::{AppError, AppResult},
};
use serde::{Deserialize, Serialize};
pub const MAX_PAYLOAD: usize = 262_144;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportFormat {
    JsonLd,
    Manual,
    Website,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportInput {
    pub source_url: String,
    pub format: ImportFormat,
    pub payload: String,
    pub permitted: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportedRecipeDraft {
    pub meal: MealDraft,
    pub warnings: Vec<String>,
    pub original_yield: String,
    pub attribution: String,
    pub duplicate_titles: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualRecipe {
    pub title: String,
    pub servings: String,
    pub ingredients: Vec<String>,
    pub instructions: String,
}
pub trait RecipeImporter {
    fn extract(&self, input: &ImportInput) -> AppResult<ImportedRecipeDraft>;
}
pub fn extract(input: &ImportInput) -> AppResult<ImportedRecipeDraft> {
    url::validate(&input.source_url)?;
    if !input.permitted {
        return Err(AppError::validation(
            "Confirm you are permitted to import this supplied content.",
        ));
    }
    if input.payload.len() > MAX_PAYLOAD || input.payload.trim().is_empty() {
        return Err(AppError::validation(
            "Supply recipe text or JSON-LD up to 256 KiB.",
        ));
    }
    match input.format {
        ImportFormat::JsonLd => json_ld::JsonLdImporter.extract(input),
        ImportFormat::Website => sites::WebsiteImporter.extract(input),
        ImportFormat::Manual => {
            let manual: ManualRecipe = serde_json::from_str(&input.payload)
                .map_err(|_| AppError::validation("Invalid manual recipe fields."))?;
            json_ld::draft(
                input,
                manual.title,
                manual.servings,
                manual.ingredients,
                manual.instructions,
                String::new(),
                None,
                None,
                String::new(),
            )
        }
    }
}
pub(crate) fn bounded_text(value: &str, limit: usize) -> AppResult<()> {
    validation::text(value, limit, false)
}

pub(crate) fn parse_line(
    raw: &str,
    mode: &crate::domain::measurements::Mode,
) -> AppResult<crate::domain::meal_library::IngredientDraft> {
    json_ld::line_with_mode(raw, mode)
}
