use crate::{domain::planning::*, error::AppResult, persistence::StorageState};
use tauri::State;
#[tauri::command]
pub async fn get_week(state: State<'_, StorageState>, date: String) -> AppResult<WeekView> {
    state.database()?.get_week(&date).await
}
#[tauri::command]
pub async fn apply_plan(
    state: State<'_, StorageState>,
    input: PlanMutation,
) -> AppResult<WeekView> {
    state.database()?.apply_plan(input).await
}
#[tauri::command]
pub async fn get_meal_history(
    state: State<'_, StorageState>,
    meal_id: String,
) -> AppResult<MealHistory> {
    state.database()?.meal_history(&meal_id).await
}
#[tauri::command]
pub async fn change_meal_history(
    state: State<'_, StorageState>,
    input: HistoryMutation,
) -> AppResult<MealHistory> {
    state.database()?.change_history(input).await
}
