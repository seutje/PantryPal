use pantrypal_lib::{
    commands,
    persistence::{Database, StorageState},
};
use serde_json::{json, Value};
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, MockRuntime, INVOKE_KEY},
    webview::InvokeRequest,
    Manager, WebviewWindow,
};
fn call(
    window: &WebviewWindow<MockRuntime>,
    cmd: &str,
    body: Value,
    origin: &str,
) -> Result<Value, Value> {
    get_ipc_response(
        window,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: origin.parse().unwrap(),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.into(),
        },
    )
    .map(|b| b.deserialize().unwrap())
}
#[test]
fn backup_restore_settings_and_clipboard_opt_in_use_local_acl_and_real_storage() {
    let dir = tempfile::tempdir().unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = rt.block_on(Database::open(dir.path())).unwrap();
    let mut context = tauri::generate_context!();
    context.set_default_window_icon(Some(tauri::image::Image::new_owned(
        vec![0, 100, 60, 255],
        1,
        1,
    )));
    let app = mock_builder()
        .manage(StorageState(Ok(db.clone())))
        .invoke_handler(tauri::generate_handler![
            commands::get_backup_info,
            commands::export_backup,
            commands::preview_restore,
            commands::restore_backup,
            commands::get_usability_preferences,
            commands::save_usability_preferences,
            commands::read_recipe_clipboard_url
        ])
        .build(context)
        .unwrap();
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap()
    });
    for cmd in [
        "get_backup_info",
        "export_backup",
        "preview_restore",
        "restore_backup",
        "get_usability_preferences",
        "save_usability_preferences",
        "read_recipe_clipboard_url",
    ] {
        assert!(call(&window, cmd, json!({}), "https://example.com").is_err());
    }
    let local = "http://tauri.localhost";
    assert!(
        call(&window, "get_backup_info", json!({}), local).unwrap()["directory"]
            .as_str()
            .unwrap()
            .ends_with("backups")
    );
    assert!(call(&window, "read_recipe_clipboard_url", json!({}), local)
        .unwrap()
        .is_null()); // Opt-out never reads the OS clipboard, even in mock runtime.
    let mut settings = call(&window, "get_usability_preferences", json!({}), local).unwrap();
    settings["sectionOrder"].as_array_mut().unwrap().reverse();
    call(
        &window,
        "save_usability_preferences",
        json!({"input":settings}),
        local,
    )
    .unwrap();
    assert_eq!(
        call(&window, "get_usability_preferences", json!({}), local).unwrap(),
        settings
    );
    let exported = call(&window, "export_backup", json!({}), local).unwrap();
    let text = std::fs::read_to_string(exported["path"].as_str().unwrap()).unwrap();
    let preview = call(
        &window,
        "preview_restore",
        json!({"text":text,"mode":"replace"}),
        local,
    )
    .unwrap();
    assert!(call(&window,"restore_backup",json!({"text":text,"mode":"replace","expectedState":preview["expectedState"],"confirmed":false}),local).is_err());
    call(&window,"restore_backup",json!({"text":text,"mode":"replace","expectedState":preview["expectedState"],"confirmed":true}),local).unwrap();
    assert!(call(
        &window,
        "preview_restore",
        json!({"text":"{}","mode":"replace"}),
        local
    )
    .is_err());
    rt.block_on(db.close());
    let db = rt.block_on(Database::open(dir.path())).unwrap();
    assert_eq!(
        serde_json::to_value(rt.block_on(db.usability_preferences()).unwrap()).unwrap(),
        settings
    );
}
