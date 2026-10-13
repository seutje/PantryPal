use crate::{
    domain::{
        catalog::{Catalog, Ingredient, IngredientInput, Vocabulary, VocabularyKind},
        fields::{FieldDefinition, FieldInput},
        meals::{Meal, MealInput},
    },
    error::AppResult,
    persistence::{StorageState, StorageStatus},
};
use serde::Serialize;
mod data;
mod imports;
pub use data::*;
mod meals;
pub use imports::*;
mod planning;
mod shopping;
pub use planning::*;
pub use shopping::*;
mod measurements;
pub use meals::*;
pub use measurements::*;
use tauri::State;

#[tauri::command]
pub async fn get_storage_status(state: State<'_, StorageState>) -> AppResult<StorageStatus> {
    state.database()?.status().await
}
#[tauri::command]
pub async fn get_catalog(state: State<'_, StorageState>) -> AppResult<Catalog> {
    state.database()?.catalog().await
}
#[tauri::command]
pub async fn create_ingredient(
    state: State<'_, StorageState>,
    input: IngredientInput,
) -> AppResult<Ingredient> {
    state.database()?.create_ingredient(input).await
}
#[tauri::command]
pub async fn update_ingredient(
    state: State<'_, StorageState>,
    id: String,
    input: IngredientInput,
) -> AppResult<Ingredient> {
    state.database()?.update_ingredient(&id, input).await
}
#[tauri::command]
pub async fn retire_ingredient(
    state: State<'_, StorageState>,
    id: String,
    confirmed: bool,
) -> AppResult<()> {
    state.database()?.retire_ingredient(&id, confirmed).await
}
#[tauri::command]
pub async fn list_meals(state: State<'_, StorageState>) -> AppResult<Vec<Meal>> {
    state.database()?.list_meals().await
}
#[tauri::command]
pub async fn get_meal(state: State<'_, StorageState>, id: String) -> AppResult<Meal> {
    state.database()?.meal(&id).await
}
#[tauri::command]
pub async fn create_meal(state: State<'_, StorageState>, input: MealInput) -> AppResult<Meal> {
    state.database()?.create_meal(input).await
}
#[tauri::command]
pub async fn update_meal(
    state: State<'_, StorageState>,
    id: String,
    expected_revision: i64,
    input: MealInput,
) -> AppResult<Meal> {
    state
        .database()?
        .update_meal(&id, expected_revision, input)
        .await
}
#[tauri::command]
pub async fn retire_meal(
    state: State<'_, StorageState>,
    id: String,
    confirmed: bool,
) -> AppResult<()> {
    state.database()?.retire_meal(&id, confirmed).await
}
#[tauri::command]
pub async fn list_fields(state: State<'_, StorageState>) -> AppResult<Vec<FieldDefinition>> {
    state.database()?.list_fields().await
}
#[tauri::command]
pub async fn create_field(
    state: State<'_, StorageState>,
    input: FieldInput,
) -> AppResult<FieldDefinition> {
    state.database()?.create_field(input).await
}
#[tauri::command]
pub async fn update_field(
    state: State<'_, StorageState>,
    id: String,
    input: FieldInput,
) -> AppResult<FieldDefinition> {
    state.database()?.update_field(&id, input).await
}
#[tauri::command]
pub async fn retire_field(
    state: State<'_, StorageState>,
    id: String,
    confirmed: bool,
) -> AppResult<()> {
    state.database()?.retire_field(&id, confirmed).await
}
#[tauri::command]
pub async fn create_vocabulary(
    state: State<'_, StorageState>,
    kind: VocabularyKind,
    name: String,
    sort_order: i64,
) -> AppResult<Vocabulary> {
    state
        .database()?
        .create_vocabulary(kind, &name, sort_order)
        .await
}
#[tauri::command]
pub async fn update_vocabulary(
    state: State<'_, StorageState>,
    kind: VocabularyKind,
    id: String,
    name: String,
    sort_order: i64,
) -> AppResult<Vocabulary> {
    state
        .database()?
        .update_vocabulary(kind, &id, &name, sort_order)
        .await
}
#[tauri::command]
pub async fn retire_vocabulary(
    state: State<'_, StorageState>,
    kind: VocabularyKind,
    id: String,
    confirmed: bool,
) -> AppResult<()> {
    state
        .database()?
        .retire_vocabulary(kind, &id, confirmed)
        .await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "PantryPal",
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_identity_matches_package() {
        let info = get_app_info();
        assert_eq!(info.name, "PantryPal");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        let payload = serde_json::to_value(info).unwrap();
        assert_eq!(payload["name"], "PantryPal");
        assert!(payload["version"].is_string());
    }
}
mod updates;
pub use updates::*;
