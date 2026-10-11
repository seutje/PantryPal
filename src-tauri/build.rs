fn main() {
    // Custom commands must participate in ACL; registration alone defaults to
    // allowing application commands in every window.
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["get_app_info"])),
    )
    .expect("failed to build PantryPal Tauri configuration");
}
