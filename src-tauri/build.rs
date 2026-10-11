fn main() {
    // Integration executables using Tauri's mock window also link its native
    // dialog code. TaskDialogIndirect requires the Common Controls v6 manifest;
    // Tauri's ordinary application resource is linked only into the app binary.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest = std::path::Path::new("tests/windows.manifest")
            .canonicalize()
            .expect("test manifest missing");
        println!("cargo:rerun-if-changed=tests/windows.manifest");
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
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
            "get_measurement_preferences",
            "save_measurement_preferences",
            "convert_measurement",
            "list_conversion_rules",
            "save_conversion_rule",
            "retire_conversion_rule",
            "ingredient_candidates",
            "search_meals",
            "save_meal_draft",
            "get_week",
            "apply_plan",
            "get_meal_history",
            "change_meal_history",
            "get_shopping",
            "change_shopping",
            "copy_shopping",
        ]),
    ))
    .expect("failed to build PantryPal Tauri configuration");
}
