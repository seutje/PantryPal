use crate::{
    error::AppResult,
    persistence::{
        backups::{BackupInfo, BackupResult, RestoreMode, RestorePreview},
        usability::UsabilityPreferences,
        StorageState,
    },
};
use tauri::State;
#[tauri::command]
pub async fn get_backup_info(state: State<'_, StorageState>) -> AppResult<BackupInfo> {
    Ok(state.database()?.backup_info())
}
#[tauri::command]
pub async fn export_backup(state: State<'_, StorageState>) -> AppResult<BackupResult> {
    state.database()?.export_backup().await
}
#[tauri::command]
pub async fn preview_restore(
    state: State<'_, StorageState>,
    text: String,
    mode: RestoreMode,
) -> AppResult<RestorePreview> {
    state.database()?.preview_restore(&text, mode).await
}
#[tauri::command]
pub async fn restore_backup(
    state: State<'_, StorageState>,
    text: String,
    mode: RestoreMode,
    expected_state: String,
    confirmed: bool,
) -> AppResult<BackupResult> {
    state
        .database()?
        .restore_backup(&text, mode, &expected_state, confirmed)
        .await
}
#[tauri::command]
pub async fn get_usability_preferences(
    state: State<'_, StorageState>,
) -> AppResult<UsabilityPreferences> {
    state.database()?.usability_preferences().await
}
#[tauri::command]
pub async fn save_usability_preferences(
    state: State<'_, StorageState>,
    input: UsabilityPreferences,
) -> AppResult<()> {
    state.database()?.save_usability_preferences(input).await
}
#[tauri::command]
pub async fn read_recipe_clipboard_url<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    state: State<'_, StorageState>,
) -> AppResult<Option<String>> {
    if !state
        .database()?
        .usability_preferences()
        .await?
        .clipboard_url_prefill
    {
        return Ok(None);
    }
    #[cfg(windows)]
    {
        let owner = window
            .hwnd()
            .map_err(|_| crate::error::AppError::storage())?;
        crate::services::clipboard::recipe_url(owner.0 as isize)
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        Ok(None)
    }
}
