use super::{
    fields::FieldValue,
    validation::{self, Amount},
};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MealIngredient {
    pub id: String,
    pub ingredient_id: Option<String>,
    pub quantity: Option<Amount>,
    #[serde(default)]
    pub amount_text: Option<String>,
    pub unit_id: Option<String>,
    pub raw_text: String,
    pub source_convention: Option<String>,
    pub note: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MealFieldValue {
    pub field_id: String,
    pub value: Option<FieldValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MealInput {
    pub title: String,
    pub description: String,
    pub base_servings: Amount,
    #[serde(default)]
    pub base_servings_text: Option<String>,
    pub prep_minutes: Option<i64>,
    pub cook_minutes: Option<i64>,
    pub instructions: String,
    pub notes: String,
    pub source_url: Option<String>,
    pub source_site: Option<String>,
    pub tags: Vec<String>,
    pub ingredients: Vec<MealIngredient>,
    pub custom_fields: Vec<MealFieldValue>,
}

impl MealInput {
    pub fn validate(&self) -> AppResult<()> {
        validation::text(&self.title, 200, true)?;
        validation::text(&self.description, 10_000, false)?;
        validation::text(&self.instructions, 100_000, false)?;
        validation::text(&self.notes, 10_000, false)?;
        self.base_servings.validate(true)?;
        validate_amount_text(&self.base_servings_text, Some(&self.base_servings))?;
        for minutes in [self.prep_minutes, self.cook_minutes].into_iter().flatten() {
            if !(0..=100_000).contains(&minutes) {
                return Err(AppError::validation("Invalid cooking time."));
            }
        }
        if let Some(url) = &self.source_url {
            validation::text(url, 2_048, true)?;
            // Stored as inert provenance, never fetched or opened in Phase 1.
            if !url.starts_with("https://") || url.chars().any(char::is_whitespace) {
                return Err(AppError::validation(
                    "Source links must start with https://.",
                ));
            }
        }
        if let Some(site) = &self.source_site {
            validation::text(site, 200, false)?;
        }
        if self.ingredients.len() > 300 || self.custom_fields.len() > 100 || self.tags.len() > 50 {
            return Err(AppError::validation("Too many meal lines, fields or tags."));
        }
        let mut ids = HashSet::new();
        for line in &self.ingredients {
            validation::id(&line.id)?;
            if !ids.insert(&line.id) {
                return Err(AppError::validation(
                    "Ingredient line identifiers must be distinct.",
                ));
            }
            validation::optional_id(&line.ingredient_id)?;
            validation::optional_id(&line.unit_id)?;
            if let Some(amount) = &line.quantity {
                amount.validate(false)?;
            }
            validate_amount_text(&line.amount_text, line.quantity.as_ref())?;
            validation::text(&line.raw_text, 2_000, false)?;
            validation::text(&line.note, 2_000, false)?;
            if let Some(convention) = &line.source_convention {
                validation::text(convention, 100, false)?;
            }
        }
        let mut tags = HashSet::new();
        for tag in &self.tags {
            validation::text(tag, 100, true)?;
            if !tags.insert(validation::normalized(tag)) {
                return Err(AppError::validation("Tags must be distinct."));
            }
        }
        let mut fields = HashSet::new();
        for value in &self.custom_fields {
            validation::id(&value.field_id)?;
            if !fields.insert(&value.field_id) {
                return Err(AppError::validation("Meal fields must be distinct."));
            }
        }
        Ok(())
    }
}

fn validate_amount_text(text: &Option<String>, amount: Option<&Amount>) -> AppResult<()> {
    use super::measurements::rational::{self, Rational};
    if let Some(text) = text {
        validation::text(text, 120, false)?;
        let parsed = rational::parse(text)?;
        let expected = amount
            .map(|a| Rational::new(a.numerator.into(), a.denominator.into()))
            .transpose()?;
        if parsed != expected {
            return Err(AppError::validation(
                "Entered amount does not match its exact quantity.",
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Meal {
    pub id: String,
    #[serde(flatten)]
    pub data: MealInput,
    pub retired: bool,
    pub revision: i64,
    pub created_at: String,
    pub updated_at: String,
}
