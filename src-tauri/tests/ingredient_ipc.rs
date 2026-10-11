//! Exercise actual command deserialization, ACL and SQLite; only the OS webview
//! is mocked. React flows are tested separately against these payload contracts.
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
fn ingredient_commands_persist_and_measurements_use_narrow_local_permissions() {
    let directory = tempfile::tempdir().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let db = runtime.block_on(Database::open(directory.path())).unwrap();
    let mut context = tauri::generate_context!();
    context.set_default_window_icon(Some(tauri::image::Image::new_owned(
        vec![0, 100, 60, 255],
        1,
        1,
    )));
    let app = mock_builder()
        .manage(StorageState(Ok(db.clone())))
        .invoke_handler(tauri::generate_handler![
            commands::get_catalog,
            commands::create_ingredient,
            commands::update_ingredient,
            commands::retire_ingredient,
            commands::ingredient_candidates,
            commands::save_conversion_rule,
            commands::convert_measurement,
            commands::get_measurement_preferences,
            commands::save_measurement_preferences
        ])
        .build(context)
        .unwrap();
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap()
    });
    let origin = "http://tauri.localhost";
    let input = json!({"name":"IPC herb","categoryId":"category-vegetable","sectionId":"section-produce","preferredUnitId":"g","aliases":["IPC leaves"],"seasonMonths":[5,6],"notes":"Keep intact"});
    let saved = call(&window, "create_ingredient", json!({"input":input}), origin).unwrap();
    let id = saved["id"].as_str().unwrap();
    let mut edited = input.clone();
    edited["notes"] = json!("Persisted edit");
    assert_eq!(
        call(
            &window,
            "update_ingredient",
            json!({"id":id,"input":edited}),
            origin
        )
        .unwrap()["notes"],
        "Persisted edit"
    );
    assert_eq!(
        call(
            &window,
            "ingredient_candidates",
            json!({"names":[" IPC LEAVES "],"excludeId":null}),
            origin
        )
        .unwrap()[0]["id"],
        id
    );
    let rule = json!({"ingredientId":id,"fromUnitId":"piece","fromAmount":"½","toUnitId":"g","toAmount":"75","approximate":true,"notes":"Medium herb bundle"});
    assert_eq!(
        call(
            &window,
            "save_conversion_rule",
            json!({"id":null,"input":rule}),
            origin
        )
        .unwrap()["originalFromText"],
        "½"
    );
    let converted=call(&window,"convert_measurement",json!({"input":{"amount":"2","fromUnit":"piece","toUnit":"g","ingredientId":id,"purpose":"quantity"}}),origin).unwrap();
    assert_eq!(converted["status"], "converted");
    assert_eq!(converted["exact"], "300");
    assert_eq!(converted["approximate"], true);
    let mut prefs = call(&window, "get_measurement_preferences", json!({}), origin).unwrap();
    prefs["mode"] = json!("metric");
    prefs["cupConvention"] = json!("us");
    call(
        &window,
        "save_measurement_preferences",
        json!({"input":prefs}),
        origin,
    )
    .unwrap();
    assert!(call(
        &window,
        "retire_ingredient",
        json!({"id":id,"confirmed":false}),
        origin
    )
    .is_err());
    // Applies the real capability manifest to the real registered command.
    assert!(call(
        &window,
        "get_catalog",
        json!({}),
        "https://example.com/recipe"
    )
    .is_err());
    assert!(call(&window,"convert_measurement",json!({"input":{"amount":"1","fromUnit":"g","toUnit":"kg","ingredientId":null,"purpose":"quantity"}}),"https://example.com/recipe").is_err());
    call(
        &window,
        "retire_ingredient",
        json!({"id":id,"confirmed":true}),
        origin,
    )
    .unwrap();
    assert!(
        !call(&window, "get_catalog", json!({}), origin).unwrap()["ingredients"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == id)
    );
    runtime.block_on(db.close());
    let reopened = runtime.block_on(Database::open(directory.path())).unwrap();
    let ingredient = runtime.block_on(reopened.ingredient(id)).unwrap();
    assert_eq!(ingredient.data.notes, "Persisted edit");
    assert_eq!(ingredient.data.aliases, vec!["IPC leaves"]);
    assert_eq!(ingredient.data.season_months, vec![5, 6]);
    assert!(ingredient.retired);
    assert_eq!(
        runtime
            .block_on(reopened.measurement_preferences())
            .unwrap()
            .mode,
        pantrypal_lib::domain::measurements::Mode::Metric
    );
    runtime.block_on(reopened.close());
}
