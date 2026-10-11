//! Actual deserialization/ACL and SQLite, with the OS WebView runtime mocked.
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
    command: &str,
    body: Value,
    origin: &str,
) -> Result<Value, Value> {
    get_ipc_response(
        window,
        InvokeRequest {
            cmd: command.into(),
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
fn shopping_commands_generate_edit_retry_and_deny_remote_access_and_stale_copy() {
    let directory = tempfile::tempdir().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let db = runtime.block_on(Database::open(directory.path())).unwrap();
    let meal = runtime
        .block_on(db.create_meal(
            serde_json::from_str(include_str!("../../tests/fixtures/demo-meal.json")).unwrap(),
        ))
        .unwrap();
    let mut context = tauri::generate_context!();
    context.set_default_window_icon(Some(tauri::image::Image::new_owned(
        vec![0, 100, 60, 255],
        1,
        1,
    )));
    let app = mock_builder()
        .manage(StorageState(Ok(db.clone())))
        .invoke_handler(tauri::generate_handler![
            commands::get_shopping,
            commands::change_shopping,
            commands::copy_shopping
        ])
        .build(context)
        .unwrap();
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap()
    });
    let origin = "http://tauri.localhost";
    for (command, body) in [
        ("get_shopping", json!({"weekStart":"2020-12-28"})),
        (
            "change_shopping",
            json!({"input":{"actionId":"remote-refresh","weekStart":"2020-12-28","expectedRevision":0,"operation":{"kind":"refresh"}}}),
        ),
        (
            "copy_shopping",
            json!({"weekStart":"2020-12-28","expectedRevision":0}),
        ),
    ] {
        assert!(call(&window, command, body, "https://example.com/recipe").is_err());
    }
    assert_eq!(
        call(
            &window,
            "copy_shopping",
            json!({"weekStart":"2020-12-28","expectedRevision":0}),
            origin
        )
        .unwrap_err()["code"],
        "validation"
    );
    for index in 0..2 {
        runtime.block_on(db.apply_plan(serde_json::from_value(json!({"actionId":format!("add-{index}"),"weekStart":"2020-12-28","expectedRevision":index,"operation":{"kind":"add","entryId":format!("entry-{index}"),"mealId":meal.id,"localDate":null,"servings":"4"}})).unwrap())).unwrap();
    }
    let refresh = json!({"actionId":"refresh","weekStart":"2020-12-28","expectedRevision":0,"operation":{"kind":"refresh"}});
    let mut view = call(&window, "change_shopping", json!({"input":refresh}), origin).unwrap();
    assert_eq!(view["revision"], 1);
    assert_eq!(
        call(&window, "change_shopping", json!({"input":refresh}), origin).unwrap()["revision"],
        1
    );
    let broccoli = view["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["ingredientId"] == "ingredient-broccoli")
        .unwrap();
    assert_eq!(broccoli["numerator"], "1000");
    assert_eq!(broccoli["sources"].as_array().unwrap().len(), 2);
    let id = broccoli["id"].clone();
    for (index,operation) in [json!({"kind":"adjust","itemId":id,"amount":"1 1/2","unitId":"kg"}),json!({"kind":"check","itemId":id,"checked":true}),json!({"kind":"manual","itemId":"soap","label":"Soap","amount":"2","unitId":"piece","sectionId":null}),json!({"kind":"refresh"})].into_iter().enumerate() {view=call(&window,"change_shopping",json!({"input":{"actionId":format!("edit-{index}"),"weekStart":"2020-12-28","expectedRevision":view["revision"],"operation":operation}}),origin).unwrap();}
    assert!(view["text"]
        .as_str()
        .unwrap()
        .contains("[x] Broccoli — 1½ kg"));
    assert!(view["text"].as_str().unwrap().contains("Soap — 2 pcs"));
    assert_eq!(
        call(
            &window,
            "copy_shopping",
            json!({"weekStart":"2020-12-28","expectedRevision":0}),
            origin
        )
        .unwrap_err()["code"],
        "conflict"
    );
    let mut data = meal.data;
    data.notes = "Source changed".into();
    runtime.block_on(db.update_meal(&meal.id, 1, data)).unwrap();
    assert_eq!(
        call(
            &window,
            "get_shopping",
            json!({"weekStart":"2020-12-28"}),
            origin
        )
        .unwrap()["stale"],
        true
    );
    assert_eq!(
        call(
            &window,
            "copy_shopping",
            json!({"weekStart":"2020-12-28","expectedRevision":view["revision"]}),
            origin
        )
        .unwrap_err()["code"],
        "validation"
    );
    runtime.block_on(db.close());
}
