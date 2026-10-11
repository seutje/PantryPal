use super::{
    catalog::Catalog,
    fields::{FieldDefinition, FieldValue},
    meals::{Meal, MealFieldValue},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngredientDraft {
    pub id: String,
    pub ingredient_id: Option<String>,
    pub amount: String,
    pub unit_id: Option<String>,
    pub raw_text: String,
    pub source_convention: Option<String>,
    pub note: String,
    pub optional: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MealDraft {
    pub title: String,
    pub description: String,
    pub servings: String,
    pub prep_minutes: Option<i64>,
    pub cook_minutes: Option<i64>,
    pub instructions: String,
    pub notes: String,
    pub source_url: Option<String>,
    pub source_site: Option<String>,
    pub tags: Vec<String>,
    pub ingredients: Vec<IngredientDraft>,
    pub custom_fields: Vec<MealFieldValue>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MealSearch {
    pub text: String,
    pub ingredient_id: Option<String>,
    pub tag: String,
    pub field: Option<FieldFilter>,
    pub cooked: CookedFilter,
    pub cooked_from: Option<String>,
    pub cooked_to: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CookedFilter {
    Any,
    Never,
    Cooked,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FieldFilter {
    pub field_id: String,
    pub operator: FieldOperator,
    pub value: Option<FieldValue>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldOperator {
    Equals,
    Unknown,
    Known,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MealRecord {
    pub meal: Meal,
    pub last_cooked: Option<String>,
    pub servings_display: String,
    pub quantity_displays: Vec<Option<String>>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MealLibrary {
    pub meals: Vec<MealRecord>,
    pub catalog: Catalog,
    pub fields: Vec<FieldDefinition>,
}
