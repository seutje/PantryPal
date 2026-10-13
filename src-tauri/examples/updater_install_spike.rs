//! Explicit Windows SDK installer acceptance; only the fixed acceptance identity.
//! Run through scripts/Test-Installer.ps1, never against the household identity.
use pantrypal_lib::persistence::Database;
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tauri_plugin_updater::UpdaterExt;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".verification")
        .join("phase8-installers");
    let data = PathBuf::from(std::env::var_os("APPDATA").expect("Windows APPDATA"))
        .join("ca.pantrypal.acceptance");
    let mode = std::env::args().nth(1).expect("seed, install or verify");
    tauri::async_runtime::block_on(async {
        let db = Database::open(&data).await.unwrap();
        if mode == "seed" {
            assert!(
                db.list_meals().await.unwrap().is_empty(),
                "Acceptance directory is already in use"
            );
            let meal = db
                .create_meal(
                    serde_json::from_str(include_str!("../../tests/fixtures/demo-meal.json"))
                        .unwrap(),
                )
                .await
                .unwrap();
            db.apply_plan(serde_json::from_value(json!({"actionId":"upgrade-plan","weekStart":"2020-12-28","expectedRevision":0,"operation":{"kind":"add","entryId":"upgrade-entry","mealId":meal.id,"localDate":"2020-12-29","servings":"2"}})).unwrap()).await.unwrap();
            db.change_shopping(serde_json::from_value(json!({"actionId":"upgrade-manual","weekStart":"2020-12-28","expectedRevision":0,"operation":{"kind":"manual","itemId":"upgrade-soap","label":"Upgrade soap","amount":"2","unitId":"piece","sectionId":null}})).unwrap()).await.unwrap();
            println!(
                "PASS: isolated installed database seeded with recipe, plan and manual shopping"
            );
        } else if mode == "verify" {
            let backup = db.export_backup().await.unwrap();
            let before: Value =
                serde_json::from_slice(&std::fs::read(root.join("before-upgrade.json")).unwrap())
                    .unwrap();
            let after: Value =
                serde_json::from_slice(&std::fs::read(backup.path).unwrap()).unwrap();
            assert_eq!(before["tables"], after["tables"]);
            assert_eq!(db.list_meals().await.unwrap().len(), 1);
            assert_eq!(db.get_week("2020-12-28").await.unwrap().entries.len(), 1);
            assert!(db
                .get_shopping("2020-12-28")
                .await
                .unwrap()
                .text
                .contains("Upgrade soap"));
            println!("PASS: installed upgrade/restart retained all exported tables, recipe, plan and manual item");
        } else {
            assert_eq!(mode, "install");
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let base = format!("http://{}", listener.local_addr().unwrap());
            let installer =
                std::fs::read(root.join("PantryPalAcceptance_0.1.1_x64-setup.exe")).unwrap();
            let signature =
                std::fs::read_to_string(root.join("PantryPalAcceptance_0.1.1_x64-setup.exe.sig"))
                    .unwrap();
            let manifest=json!({"version":"0.1.1","url":format!("{base}/installer"),"signature":signature.trim()}).to_string();
            let stopped = Arc::new(AtomicBool::new(false));
            let stop = stopped.clone();
            let server = std::thread::spawn(move || {
                while !stop.load(Ordering::Acquire) {
                    if let Ok((mut stream, _)) = listener.accept() {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(5)))
                            .unwrap();
                        let mut request = [0; 4096];
                        let n = stream.read(&mut request).unwrap();
                        let body = if String::from_utf8_lossy(&request[..n])
                            .starts_with("GET /installer ")
                        {
                            installer.as_slice()
                        } else {
                            manifest.as_bytes()
                        };
                        write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).unwrap();
                        stream.write_all(body).unwrap();
                    } else {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                }
            });
            let mut context = tauri::generate_context!();
            // This fixture upgrades an installed 0.1.0 regardless of the source version.
            context.package_info_mut().version = "0.1.0".parse().unwrap();
            let public = context.config().plugins.0["updater"]["pubkey"].clone();
            context.config_mut().plugins.0.insert("updater".into(),json!({"pubkey":public,"requireSignedVersion":true,"dangerousInsecureTransportProtocol":true,"windows":{"installMode":"quiet"}}));
            let app = tauri::test::mock_builder()
                .plugin(tauri_plugin_updater::Builder::new().build())
                .build(context)
                .unwrap();
            let update = app
                .updater_builder()
                .endpoints(vec![format!("{base}/manifest").parse().unwrap()])
                .unwrap()
                .timeout(Duration::from_secs(15))
                .no_proxy()
                .installer_arg(format!("/D={}", root.join("installed").display()))
                .build()
                .unwrap()
                .check()
                .await
                .unwrap()
                .expect("Newer fixture version")
                .restart_after_install(false);
            let bytes = update.download(|_, _| {}, || {}).await.unwrap();
            stopped.store(true, Ordering::Release);
            server.join().unwrap();
            let backup = db.export_backup().await.unwrap();
            std::fs::copy(backup.path, root.join("before-upgrade.json")).unwrap();
            db.close().await;
            println!("PASS: newer NSIS download and signed version verified; recovery backup saved; handing off to SDK installer");
            update.install(bytes).unwrap();
            unreachable!("Successful Windows handoff exits the helper");
        }
        db.close().await;
    });
}
