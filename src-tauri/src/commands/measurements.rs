use crate::{
    domain::{catalog::Ingredient, measurements::*},
    error::AppResult,
    persistence::StorageState,
};
use tauri::State;
#[tauri::command]
pub async fn search_ingredients(
    state: State<'_, StorageState>,
    query: String,
) -> AppResult<Vec<Ingredient>> {
    state.database()?.search_ingredients(query).await
}
#[tauri::command]
pub async fn get_measurement_preferences(
    state: State<'_, StorageState>,
) -> AppResult<MeasurementPreferences> {
    state.database()?.measurement_preferences().await
}
#[tauri::command]
pub async fn save_measurement_preferences(
    state: State<'_, StorageState>,
    input: MeasurementPreferences,
) -> AppResult<MeasurementPreferences> {
    state.database()?.save_measurement_preferences(input).await
}
#[tauri::command]
pub async fn convert_measurement(
    state: State<'_, StorageState>,
    input: ConversionRequest,
) -> AppResult<ConversionResult> {
    state.database()?.convert_measurement(input).await
}
#[tauri::command]
pub async fn list_conversion_rules(
    state: State<'_, StorageState>,
    ingredient_id: String,
) -> AppResult<Vec<ConversionRule>> {
    state.database()?.conversion_rules(&ingredient_id).await
}
#[tauri::command]
pub async fn save_conversion_rule(
    state: State<'_, StorageState>,
    id: Option<String>,
    input: RuleInput,
) -> AppResult<ConversionRule> {
    state.database()?.save_conversion_rule(id, input).await
}
#[tauri::command]
pub async fn retire_conversion_rule(
    state: State<'_, StorageState>,
    id: String,
    confirmed: bool,
) -> AppResult<()> {
    state
        .database()?
        .retire_conversion_rule(&id, confirmed)
        .await
}
#[tauri::command]
pub async fn ingredient_candidates(
    state: State<'_, StorageState>,
    names: Vec<String>,
    exclude_id: Option<String>,
) -> AppResult<Vec<Ingredient>> {
    state
        .database()?
        .ingredient_candidates(names, exclude_id)
        .await
}
