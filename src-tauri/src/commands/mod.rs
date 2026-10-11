use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "PantryPal",
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_identity_matches_package() {
        let info = get_app_info();
        assert_eq!(info.name, "PantryPal");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        let payload = serde_json::to_value(info).unwrap();
        assert_eq!(payload["name"], "PantryPal");
        assert!(payload["version"].is_string());
    }
}
