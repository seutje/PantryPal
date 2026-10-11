pub mod commands;
pub mod domain;
pub mod error;
pub mod importers;
pub mod persistence;
pub mod services;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::get_app_info])
        .run(tauri::generate_context!())
        .expect("failed to run PantryPal");
}
