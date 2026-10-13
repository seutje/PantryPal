use crate::{
    error::AppResult,
    persistence::StorageState,
    services::updates::{self, Check, Progress, UpdateState},
};
use tauri::{ipc::Channel, AppHandle, Runtime, State};
#[tauri::command]
pub async fn check_app_update<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, UpdateState>,
) -> AppResult<Check> {
    updates::check(&app, &state).await
}
#[tauri::command]
pub async fn install_app_update(
    state: State<'_, UpdateState>,
    storage: State<'_, StorageState>,
    version: String,
    confirmed: bool,
    progress: Channel<Progress>,
) -> AppResult<()> {
    updates::install(&state, storage.database()?, &version, confirmed, progress).await
}
