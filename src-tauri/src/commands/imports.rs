use crate::{
    domain::meals::Meal,
    error::{AppError, AppResult},
    importers::{self, ImportInput, ImportedRecipeDraft},
    persistence::{imports::ConfirmImport, StorageState},
};
use tauri::{
    webview::{NewWindowResponse, PermissionResponse},
    Manager, State, WebviewUrl,
};

#[tauri::command]
pub fn inspect_recipe_url(source_url: String) -> AppResult<String> {
    let url = importers::url::validate(&source_url)?;
    Ok(importers::url::site_message(url.host_str().unwrap()).into())
}
#[tauri::command]
pub async fn preview_recipe_import(
    state: State<'_, StorageState>,
    input: ImportInput,
) -> AppResult<ImportedRecipeDraft> {
    state.database()?.preview_import(&input).await
}
#[tauri::command]
pub async fn confirm_recipe_import(
    state: State<'_, StorageState>,
    input: ConfirmImport,
) -> AppResult<Meal> {
    state.database()?.confirm_import(input).await
}
#[tauri::command]
pub async fn open_recipe_browser(app: tauri::AppHandle, source_url: String) -> AppResult<()> {
    let url = importers::url::validate(&source_url)?;
    let state = app.state::<crate::services::browser::BrowserState>();
    let _task = state.begin()?;
    if app.get_webview_window("recipe-browser-reset").is_some() {
        return Err(AppError::validation(
            "Website sessions are being cleared. Wait, then open the recipe again.",
        ));
    }
    // A repeated open keeps the existing login/page intact. Changing sites
    // requires closing the previous window so its fixed origin policy stays clear.
    if let Some(window) = app.get_webview_window("recipe-browser") {
        if window.url().is_ok_and(|current| current == url) {
            window.set_focus().map_err(|_| browser_error())?;
            return Ok(());
        }
        return Err(AppError::validation(
            "Close the previous recipe website window, then open this recipe.",
        ));
    }
    let origin = url.origin();
    let profile = state.profile()?;
    tauri::WebviewWindowBuilder::new(&app, "recipe-browser", WebviewUrl::External(url))
        .title("Recipe website · PantryPal")
        .inner_size(1050.0, 800.0)
        .data_directory(profile)
        .on_navigation(move |url| {
            importers::url::validate(url.as_str()).is_ok() && url.origin() == origin
        })
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_download(|_, _| false)
        .on_permission_request(|_, _| PermissionResponse::Deny)
        .build()
        .map_err(|_| browser_error())?;
    Ok(())
}
#[tauri::command]
pub async fn reset_recipe_browser(app: tauri::AppHandle, confirmed: bool) -> AppResult<()> {
    if !confirmed {
        return Err(AppError::validation(
            "Confirm clearing recipe website sessions.",
        ));
    }
    let state = app.state::<crate::services::browser::BrowserState>();
    let _task = state.begin()?;
    // Close active website content so it cannot repopulate storage while the
    // profile clears. A distinct hidden label avoids a close/reopen label race.
    if app.get_webview_window("recipe-browser-reset").is_some() {
        return Err(AppError::validation(
            "A session reset is already in progress.",
        ));
    }
    if let Some(window) = app.get_webview_window("recipe-browser") {
        window.close().map_err(|_| browser_error())?;
    }
    let window = {
        let profile = state.profile()?;
        tauri::WebviewWindowBuilder::new(
            &app,
            "recipe-browser-reset",
            WebviewUrl::External("about:blank".parse().unwrap()),
        )
        .visible(false)
        .data_directory(profile)
        .on_navigation(|url| url.as_str() == "about:blank")
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_download(|_, _| false)
        .on_permission_request(|_, _| PermissionResponse::Deny)
        .build()
        .map_err(|_| browser_error())?
    };
    let cleared = crate::services::browser::clear_profile(&window).await;
    let closed = window.close().map_err(|_| browser_error());
    cleared?;
    closed?;
    Ok(())
}
fn browser_error() -> AppError {
    AppError::validation("Recipe browsing is unavailable. Use your regular browser and manual recipe entry; your meals are unchanged.")
}
