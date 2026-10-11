use crate::{
    domain::{
        meal_library::{MealDraft, MealLibrary, MealSearch},
        meals::Meal,
    },
    error::AppResult,
    persistence::StorageState,
};
use tauri::State;
#[tauri::command]
pub async fn save_meal_draft(
    state: State<'_, StorageState>,
    id: Option<String>,
    expected_revision: Option<i64>,
    input: MealDraft,
) -> AppResult<Meal> {
    state
        .database()?
        .save_meal_draft(id, expected_revision, input)
        .await
}
#[tauri::command]
pub async fn search_meals(
    state: State<'_, StorageState>,
    input: MealSearch,
) -> AppResult<MealLibrary> {
    state.database()?.search_meals(input).await
}
