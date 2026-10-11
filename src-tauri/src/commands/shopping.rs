use crate::{
    domain::shopping::{ShoppingMutation, ShoppingView},
    error::{AppError, AppResult},
    persistence::StorageState,
};
use tauri::{Runtime, State, WebviewWindow};
#[tauri::command]
pub async fn get_shopping(
    state: State<'_, StorageState>,
    week_start: String,
) -> AppResult<ShoppingView> {
    state.database()?.get_shopping(&week_start).await
}
#[tauri::command]
pub async fn change_shopping(
    state: State<'_, StorageState>,
    input: ShoppingMutation,
) -> AppResult<ShoppingView> {
    state.database()?.change_shopping(input).await
}
#[tauri::command]
pub async fn copy_shopping<R: Runtime>(
    window: WebviewWindow<R>,
    state: State<'_, StorageState>,
    week_start: String,
    expected_revision: i64,
) -> AppResult<()> {
    let view = state.database()?.get_shopping(&week_start).await?;
    if view.revision != expected_revision {
        return Err(AppError::conflict());
    }
    if view.stale {
        return Err(AppError::validation(
            "Refresh this shopping list before copying.",
        ));
    }
    if !view
        .items
        .iter()
        .any(|i| !i.item.excluded && !i.item.orphaned)
    {
        return Err(AppError::validation(
            "Add an item or generate this week's list before copying.",
        ));
    }
    #[cfg(windows)]
    {
        let owner = window.hwnd().map_err(|_| AppError::storage())?;
        crate::services::clipboard::write(owner.0 as isize, &view.text)
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        Err(AppError::validation(
            "Clipboard integration currently supports Windows.",
        ))
    }
}
