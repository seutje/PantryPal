//! Verify the complete planner command contract and its local-only capability.
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
    .map(|body| body.deserialize().unwrap())
}
#[test]
fn week_planning_and_history_use_real_commands_with_restart_and_remote_denial() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let db = runtime.block_on(Database::open(dir.path())).unwrap();
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
            commands::get_week,
            commands::apply_plan,
            commands::get_meal_history,
            commands::change_meal_history
        ])
        .build(context)
        .unwrap();
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap()
    });
    let origin = "http://tauri.localhost";
    assert!(call(
        &window,
        "get_week",
        json!({"date":"2021-01-01"}),
        "https://example.com/recipe"
    )
    .is_err());
    let mut week = call(&window, "get_week", json!({"date":"2021-01-01"}), origin).unwrap();
    assert_eq!(week["weekStart"], "2020-12-28");
    assert_eq!(week["weekEnd"], "2021-01-03");
    let add = json!({"actionId":"add-once","weekStart":"2020-12-28","expectedRevision":0,"operation":{"kind":"add","entryId":"ipc-entry","mealId":meal.id,"localDate":null,"servings":"1 1/2"}});
    assert!(call(
        &window,
        "apply_plan",
        json!({"input":add}),
        "https://example.com/recipe"
    )
    .is_err());
    week = call(&window, "apply_plan", json!({"input":add}), origin).unwrap();
    assert_eq!(
        week["entries"][0]["servings"],
        json!({"numerator":3,"denominator":2})
    );
    assert_eq!(
        call(&window, "apply_plan", json!({"input":add}), origin).unwrap()["revision"],
        1
    );
    let cook = json!({"actionId":"cook-once","weekStart":"2020-12-28","expectedRevision":1,"operation":{"kind":"set_status","entryId":"ipc-entry","status":"cooked","cookedDate":"2020-12-31","confirmed":false}});
    week = call(&window, "apply_plan", json!({"input":cook}), origin).unwrap();
    call(&window, "apply_plan", json!({"input":cook}), origin).unwrap();
    assert_eq!(week["entries"][0]["status"], "cooked");
    let history = call(
        &window,
        "get_meal_history",
        json!({"mealId":meal.id}),
        origin,
    )
    .unwrap();
    assert_eq!(history["events"].as_array().unwrap().len(), 1);
    assert_eq!(history["lastCooked"], "2020-12-31");
    let undo = json!({"actionId":"undo-once","mealId":meal.id,"operation":{"kind":"undo","eventId":history["events"][0]["id"],"expectedRevision":1,"confirmed":true}});
    assert!(call(
        &window,
        "change_meal_history",
        json!({"input":undo}),
        "https://example.com/recipe"
    )
    .is_err());
    call(
        &window,
        "change_meal_history",
        json!({"input":undo}),
        origin,
    )
    .unwrap();
    call(&window, "apply_plan", json!({"input":cook}), origin).unwrap();
    assert_eq!(
        call(
            &window,
            "get_meal_history",
            json!({"mealId":meal.id}),
            origin
        )
        .unwrap()["lastCooked"],
        Value::Null
    );
    runtime.block_on(db.close());
    drop(window);
    drop(app);
    let db = runtime.block_on(Database::open(dir.path())).unwrap();
    let week = runtime.block_on(db.get_week("2021-01-01")).unwrap();
    assert_eq!(week.entries.len(), 1);
    assert_eq!(week.entries[0].status.as_str(), "planned");
    assert!(week.entries[0].cooked_event.as_ref().unwrap().voided);
    runtime
        .block_on(db.apply_plan(serde_json::from_value(cook).unwrap()))
        .unwrap();
    assert!(runtime
        .block_on(db.meal_history(&meal.id))
        .unwrap()
        .last_cooked
        .is_none());
    runtime.block_on(db.close());
}
