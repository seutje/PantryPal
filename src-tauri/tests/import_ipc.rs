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
    .map(|body| body.deserialize().unwrap())
}
#[test]
fn import_review_atomic_save_retry_duplicate_and_restart_use_real_acl_commands() {
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
            commands::inspect_recipe_url,
            commands::preview_recipe_import,
            commands::confirm_recipe_import
        ])
        .build(context)
        .unwrap();
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap()
    });
    let local = "http://tauri.localhost";
    let source = json!({"sourceUrl":"https://example.com/owned-recipe","format":"json_ld","payload":include_str!("../../tests/fixtures/owned-recipe.json"),"permitted":true});
    for cmd in [
        "inspect_recipe_url",
        "preview_recipe_import",
        "confirm_recipe_import",
    ] {
        assert!(call(
            &window,
            cmd,
            json!({"input":source,"sourceUrl":source["sourceUrl"]}),
            "https://example.com"
        )
        .is_err());
    }
    let mut preview = call(
        &window,
        "preview_recipe_import",
        json!({"input":source}),
        local,
    )
    .unwrap();
    assert_eq!(rt.block_on(db.status()).unwrap().meal_count, 0);
    assert!(preview["meal"]["ingredients"][2]["ingredientId"].is_string());
    assert!(preview["meal"]["ingredients"][6]["ingredientId"].is_null());
    preview["meal"]["servings"] = json!("4");
    let mut request = json!({"actionId":"import-once","source":source,"reviewed":preview["meal"],"confirmed":true,"allowDuplicate":false});
    let mut invalid = request.clone();
    invalid["reviewed"]["ingredients"][0]["ingredientId"] = json!("missing-canonical-id");
    assert!(call(
        &window,
        "confirm_recipe_import",
        json!({"input":invalid}),
        local
    )
    .is_err());
    assert_eq!(rt.block_on(db.status()).unwrap().meal_count, 0);
    let saved = call(
        &window,
        "confirm_recipe_import",
        json!({"input":request}),
        local,
    )
    .unwrap();
    let id = saved["id"].as_str().unwrap();
    assert_eq!(
        call(
            &window,
            "confirm_recipe_import",
            json!({"input":request}),
            local
        )
        .unwrap()["id"],
        id
    );
    assert_eq!(rt.block_on(db.status()).unwrap().meal_count, 1);
    let second_preview = call(
        &window,
        "preview_recipe_import",
        json!({"input":source}),
        local,
    )
    .unwrap();
    assert_eq!(second_preview["duplicateTitles"][0], "Owned fixture soup");
    request["actionId"] = json!("import-another");
    for (n, line) in request["reviewed"]["ingredients"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        line["id"] = json!(format!("another-line-{n}"));
    }
    assert!(call(
        &window,
        "confirm_recipe_import",
        json!({"input":request}),
        local
    )
    .is_err());
    request["allowDuplicate"] = json!(true);
    call(
        &window,
        "confirm_recipe_import",
        json!({"input":request}),
        local,
    )
    .unwrap();
    assert_eq!(rt.block_on(db.status()).unwrap().meal_count, 2);
    let sqlite = rt
        .block_on(sqlx::SqlitePool::connect(&format!(
            "sqlite://{}",
            dir.path().join("pantrypal.sqlite3").display()
        )))
        .unwrap();
    let (payload, timestamp): (String, String) = rt
        .block_on(
            sqlx::query_as(
                "SELECT provenance_json,imported_at FROM import_record WHERE id='import-once'",
            )
            .fetch_one(&sqlite),
        )
        .unwrap();
    assert!(!timestamp.is_empty());
    let provenance: Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(provenance["originalPayload"], source["payload"]);
    let mut edit = rt.block_on(db.meal(id)).unwrap().data;
    edit.title = "Edited import".into();
    rt.block_on(db.update_meal(id, 1, edit)).unwrap();
    drop(app);
    drop(db);
    let reopened = rt.block_on(Database::open(dir.path())).unwrap();
    assert_eq!(
        rt.block_on(reopened.meal(id)).unwrap().data.title,
        "Edited import"
    );
    assert_eq!(rt.block_on(reopened.status()).unwrap().meal_count, 2);
    let unchanged: String = rt
        .block_on(
            sqlx::query_scalar("SELECT provenance_json FROM import_record WHERE id='import-once'")
                .fetch_one(&sqlite),
        )
        .unwrap();
    assert_eq!(payload, unchanged);
}
