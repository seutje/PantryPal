fn main() {
    println!("cargo:rerun-if-changed=migrations");
    // Custom commands must participate in ACL; registration alone defaults to
    // allowing application commands in every window.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_app_info",
            "get_storage_status",
            "get_catalog",
            "create_ingredient",
            "update_ingredient",
            "retire_ingredient",
            "list_meals",
            "get_meal",
            "create_meal",
            "update_meal",
            "retire_meal",
            "list_fields",
            "create_field",
            "update_field",
            "retire_field",
            "create_vocabulary",
            "update_vocabulary",
            "retire_vocabulary",
        ]),
    ))
    .expect("failed to build PantryPal Tauri configuration");
}
