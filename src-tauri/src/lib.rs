pub mod commands;
pub mod domain;
pub mod error;
pub mod importers;
pub mod persistence;
pub mod services;

pub fn run() {
    use tauri::Manager;
    tauri::Builder::default()
        .setup(|app| {
            app.manage(services::browser::BrowserState::new(
                app.path()
                    .app_data_dir()
                    .map(|directory| directory.join("recipe-browser-profile"))
                    .map_err(|_| error::AppError::storage()),
            ));
            let database = app
                .path()
                .app_data_dir()
                .map_err(|_| error::AppError::storage())
                .and_then(|directory| {
                    tauri::async_runtime::block_on(persistence::Database::open(&directory))
                });
            app.manage(persistence::StorageState(database));
            Ok(())
        })
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
            commands::capture_recipe_browser
        ])
        .run(tauri::generate_context!())
        .expect("failed to run PantryPal");
}
