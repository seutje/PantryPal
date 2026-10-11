//! Real command deserialization/ACL/repositories, with an isolated database.
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
fn call(window: &WebviewWindow<MockRuntime>, command: &str, body: Value) -> Result<Value, Value> {
    call_origin(window, command, body, "http://tauri.localhost")
}
fn call_origin(
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
fn search() -> Value {
    json!({"text":"","ingredientId":null,"tag":"","field":null,"cooked":"any","cookedFrom":null,"cookedTo":null})
}
#[test]
fn complete_meal_fields_search_edit_retirement_and_restart() {
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
            commands::create_field,
            commands::update_field,
            commands::retire_field,
            commands::save_meal_draft,
            commands::search_meals,
            commands::retire_meal,
            commands::retire_ingredient
        ])
        .build(context)
        .unwrap();
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap()
    });
    assert!(call_origin(
        &window,
        "search_meals",
        json!({"input":search()}),
        "https://example.com/recipe"
    )
    .is_err());
    let mut values = vec![];
    let mut definitions = vec![];
    for (kind, value) in [
        ("boolean", json!(false)),
        ("select", json!("choice-winter")),
        ("number", json!(2.5)),
        ("date", json!("2026-10-11")),
        ("text", json!("<script>not executable</script>")),
        ("rating", json!(4)),
    ] {
        let input = json!({"label":format!("Field {kind}"),"kind":kind,"description":"Keep values","options":if kind=="select" {json!([{"id":"choice-winter","label":"Winter favourite"}])}else{json!([])},"minimum":if kind=="rating"{json!(1)}else{Value::Null},"maximum":if kind=="rating"{json!(5)}else{Value::Null},"unitLabel":null,"sortOrder":definitions.len()});
        let definition = call(&window, "create_field", json!({"input":input})).unwrap();
        values.push(json!({"fieldId":definition["id"],"value":{"kind":kind,"value":value}}));
        definitions.push((definition, input));
    }
    let mut draft = json!({"title":"Onion supper","description":"Household recipe","servings":"1 1/2","prepMinutes":10,"cookMinutes":20,"instructions":"Chop\nSimmer","notes":"Keep raw units","sourceUrl":"https://example.com/allowed-recipe","sourceSite":"Family","tags":["winter"],"ingredients":[{"id":"line-one","ingredientId":"ingredient-onion","amount":"⅓","unitId":"cup-us","rawText":"⅓ US cup onions, chopped","sourceConvention":"us","note":"finely chopped","optional":false},{"id":"line-two","ingredientId":null,"amount":"to taste","unitId":"pinch","rawText":"a pinch of family seasoning","sourceConvention":null,"note":"unknown ingredient","optional":true}],"customFields":values});
    let saved = call(
        &window,
        "save_meal_draft",
        json!({"id":null,"expectedRevision":null,"input":draft}),
    )
    .unwrap();
    let id = saved["id"].as_str().unwrap();
    assert_eq!(
        saved["baseServings"],
        json!({"numerator":3,"denominator":2})
    );
    assert_eq!(saved["ingredients"][0]["amountText"], "⅓");
    assert_eq!(saved["ingredients"][1]["quantity"], Value::Null);
    let mut filter = search();
    filter["text"] = json!("Winter favourite");
    let library = call(&window, "search_meals", json!({"input":filter})).unwrap();
    assert_eq!(library["meals"].as_array().unwrap().len(), 1);
    assert_eq!(library["meals"][0]["lastCooked"], Value::Null);
    assert_eq!(library["meals"][0]["quantityDisplays"][0], "⅓");
    filter = search();
    filter["ingredientId"] = json!("ingredient-onion");
    filter["field"] = json!({"fieldId":definitions[0].0["id"],"operator":"equals","value":{"kind":"boolean","value":false}});
    assert_eq!(
        call(&window, "search_meals", json!({"input":filter})).unwrap()["meals"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    filter["field"]["operator"] = json!("unknown");
    filter["field"]["value"] = Value::Null;
    assert!(
        call(&window, "search_meals", json!({"input":filter})).unwrap()["meals"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    // Invalid children, units and quantities must leave the entire old record intact.
    for bad in [json!("-1"), json!("1/0"), json!("nonsense")] {
        let mut invalid = draft.clone();
        invalid["ingredients"][0]["amount"] = bad;
        assert!(call(
            &window,
            "save_meal_draft",
            json!({"id":id,"expectedRevision":1,"input":invalid})
        )
        .is_err());
    }
    let mut invalid = draft.clone();
    invalid["ingredients"][0]["unitId"] = json!("celsius");
    assert!(call(
        &window,
        "save_meal_draft",
        json!({"id":id,"expectedRevision":1,"input":invalid})
    )
    .is_err());
    let mut invalid = draft.clone();
    invalid["customFields"][5]["value"]["value"] = json!(6);
    assert!(call(
        &window,
        "save_meal_draft",
        json!({"id":id,"expectedRevision":1,"input":invalid})
    )
    .is_err());
    let mut renamed = definitions[1].1.clone();
    renamed["label"] = json!("Season choice");
    renamed["options"][0]["label"] = json!("Winter comfort");
    call(
        &window,
        "update_field",
        json!({"id":definitions[1].0["id"],"input":renamed}),
    )
    .unwrap();
    renamed["kind"] = json!("text");
    renamed["options"] = json!([]);
    assert!(call(
        &window,
        "update_field",
        json!({"id":definitions[1].0["id"],"input":renamed})
    )
    .is_err());
    assert!(call(
        &window,
        "retire_field",
        json!({"id":definitions[0].0["id"],"confirmed":false})
    )
    .is_err());
    call(
        &window,
        "retire_field",
        json!({"id":definitions[0].0["id"],"confirmed":true}),
    )
    .unwrap();
    call(
        &window,
        "retire_ingredient",
        json!({"id":"ingredient-onion","confirmed":true}),
    )
    .unwrap();
    draft["title"] = json!("Edited supper");
    draft["ingredients"].as_array_mut().unwrap().reverse();
    // Omitted retired field is retained by the repository.
    draft["customFields"].as_array_mut().unwrap().remove(0);
    let edited = call(
        &window,
        "save_meal_draft",
        json!({"id":id,"expectedRevision":1,"input":draft}),
    )
    .unwrap();
    assert_eq!(edited["revision"], 2);
    assert_eq!(edited["ingredients"][1]["amountText"], "⅓");
    assert_eq!(edited["customFields"].as_array().unwrap().len(), 6);
    assert_eq!(
        call(
            &window,
            "save_meal_draft",
            json!({"id":id,"expectedRevision":1,"input":draft})
        )
        .unwrap_err()["code"],
        "conflict"
    );
    assert!(call(
        &window,
        "save_meal_draft",
        json!({"id":null,"expectedRevision":null,"input":draft})
    )
    .is_err());
    rt.block_on(db.close());
    drop(window);
    drop(app);
    let reopened = rt.block_on(Database::open(dir.path())).unwrap();
    assert_eq!(
        serde_json::to_value(rt.block_on(reopened.meal(id)).unwrap()).unwrap(),
        edited
    );
    filter = search();
    filter["text"] = json!("onions");
    filter["field"] = json!({"fieldId":definitions[0].0["id"],"operator":"equals","value":{"kind":"boolean","value":false}});
    let library = rt
        .block_on(reopened.search_meals(serde_json::from_value(filter).unwrap()))
        .unwrap();
    assert_eq!(library.meals.len(), 1);
    assert!(library
        .catalog
        .ingredients
        .iter()
        .any(|i| i.id == "ingredient-onion" && i.retired));
    assert!(rt.block_on(reopened.retire_meal(id, false)).is_err());
    rt.block_on(reopened.retire_meal(id, true)).unwrap();
    assert!(rt.block_on(reopened.list_meals()).unwrap().is_empty());
    assert_eq!(
        rt.block_on(reopened.meal(id))
            .unwrap()
            .data
            .ingredients
            .len(),
        2
    );
    rt.block_on(reopened.close());
}
