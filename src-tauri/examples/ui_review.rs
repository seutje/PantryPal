//! Visual review fixture: bundled UI with disposable local data, never household data.
use pantrypal_lib::{
    commands,
    persistence::{Database, StorageState},
    services::{browser::BrowserState, updates::UpdateState},
};
use serde_json::json;

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".verification");
    let data = root.join(format!("ui-review-{}", uuid::Uuid::new_v4()));
    let db = tauri::async_runtime::block_on(async {
        let db = Database::open(&data).await.unwrap();
        let field = db.create_field(serde_json::from_value(json!({"label":"Freezes well","kind":"boolean","description":"Household preference","options":[],"minimum":null,"maximum":null,"unitLabel":null,"sortOrder":0})).unwrap()).await.unwrap();
        let today = std::env::args()
            .nth(1)
            .expect("Pass the local review date as YYYY-MM-DD");
        let week = db.get_week(&today).await.unwrap();
        for (index, title) in [
            "Broccoli and rice",
            "Weeknight vegetable soup",
            "Sunday rice bowl",
        ]
        .iter()
        .enumerate()
        {
            let mut recipe: serde_json::Value =
                serde_json::from_str(include_str!("../../tests/fixtures/demo-meal.json")).unwrap();
            recipe["title"] = json!(title);
            recipe["description"] = json!("Visual review fixture. All records are disposable.");
            for line in recipe["ingredients"].as_array_mut().unwrap() {
                line["id"] = json!(format!("{}-{index}", line["id"].as_str().unwrap()));
            }
            recipe["customFields"] =
                json!([{"fieldId":field.id,"value":{"kind":"boolean","value":false}}]);
            let meal = db
                .create_meal(serde_json::from_value(recipe).unwrap())
                .await
                .unwrap();
            let current = db.get_week(&today).await.unwrap();
            db.apply_plan(serde_json::from_value(json!({"actionId":format!("review-add-{index}"),"weekStart":week.week_start,"expectedRevision":current.revision,"operation":{"kind":"add","entryId":format!("review-entry-{index}"),"mealId":meal.id,"localDate":week.days[index],"servings":"2"}})).unwrap()).await.unwrap();
        }
        db.change_shopping(serde_json::from_value(json!({"actionId":"review-refresh","weekStart":week.week_start,"expectedRevision":0,"operation":{"kind":"refresh"}})).unwrap()).await.unwrap();
        db
    });
    println!("Disposable visual review data: {}", data.display());
    let mut context = tauri::generate_context!();
    context.config_mut().build.dev_url = None;
    context.config_mut().app.windows.clear();
    tauri::Builder::default()
        .manage(StorageState(Ok(db)))
        .manage(UpdateState::default())
        .manage(BrowserState::new(Ok(data.join("recipe-profile"))))
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::get_storage_status,
            commands::get_catalog,
            commands::create_ingredient,
            commands::update_ingredient,
            commands::retire_ingredient,
            commands::list_meals,
            commands::get_meal,
            commands::create_meal,
            commands::update_meal,
            commands::retire_meal,
            commands::list_fields,
            commands::create_field,
            commands::update_field,
            commands::retire_field,
            commands::create_vocabulary,
            commands::update_vocabulary,
            commands::retire_vocabulary,
            commands::get_measurement_preferences,
            commands::save_measurement_preferences,
            commands::convert_measurement,
            commands::list_conversion_rules,
            commands::save_conversion_rule,
            commands::retire_conversion_rule,
            commands::ingredient_candidates,
            commands::search_ingredients,
            commands::search_meals,
            commands::save_meal_draft,
            commands::get_week,
            commands::apply_plan,
            commands::get_meal_history,
            commands::change_meal_history,
            commands::get_shopping,
            commands::change_shopping,
            commands::copy_shopping,
            commands::inspect_recipe_url,
            commands::preview_recipe_import,
            commands::confirm_recipe_import,
            commands::open_recipe_browser,
            commands::reset_recipe_browser,
            commands::capture_recipe_browser,
            commands::get_backup_info,
            commands::export_backup,
            commands::preview_restore,
            commands::restore_backup,
            commands::get_usability_preferences,
            commands::save_usability_preferences,
            commands::read_recipe_clipboard_url,
            commands::check_app_update,
            commands::install_app_update
        ])
        .setup(move |app| {
            tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("PantryPal - disposable UI review")
            .inner_size(1100.0, 760.0)
            .min_inner_size(640.0, 480.0)
            .data_directory(data.join("shell-profile"))
            .visible(true)
            .build()?;
            Ok(())
        })
        .run(context)
        .expect("Visual review fixture could not start");
}
