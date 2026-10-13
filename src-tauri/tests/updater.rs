//! Owned loopback fixtures exercise the actual SDK; no installer is executed.
use pantrypal_lib::{
    commands,
    persistence::{Database, StorageState},
    services::updates::UpdateState,
};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::TcpListener,
    time::Duration,
};
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, INVOKE_KEY},
    webview::InvokeRequest,
    Manager,
};
use tauri_plugin_updater::UpdaterExt;

#[tokio::test]
async fn signed_download_rejects_tampering_wrong_version_and_network_failure() {
    for (version, tamper, fail, expected) in [
        ("0.2.0", false, false, true),
        ("0.2.0", true, false, false),
        ("0.3.0", false, false, false),
        ("0.2.0", false, true, false),
        ("0.1.0", false, false, false),
        ("0.0.9", false, false, false),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let manifest = json!({"version":version,"notes":"Fixture only", "url":format!("{base}/payload"),"signature":include_str!("../../tests/fixtures/updater-payload.txt.sig").trim()}).to_string();
        let stopped = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop = stopped.clone();
        let server = std::thread::spawn(move || {
            while !stop.load(std::sync::atomic::Ordering::Acquire) {
                if let Ok((mut stream, _)) = listener.accept() {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    let mut request = [0; 4096];
                    let n = stream.read(&mut request).unwrap();
                    let payload =
                        String::from_utf8_lossy(&request[..n]).starts_with("GET /payload ");
                    let body = if payload {
                        if tamper {
                            "tampered"
                        } else {
                            include_str!("../../tests/fixtures/updater-payload.txt")
                        }
                    } else {
                        &manifest
                    };
                    let status = if payload && fail {
                        "500 Internal Server Error"
                    } else {
                        "200 OK"
                    };
                    write!(stream,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
                } else {
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        });
        let mut context = tauri::generate_context!();
        let public = context.config().plugins.0["updater"]["pubkey"].clone();
        context.config_mut().plugins.0.insert("updater".into(), json!({"pubkey":public,"requireSignedVersion":true,"dangerousInsecureTransportProtocol":true}));
        let app = mock_builder()
            .plugin(tauri_plugin_updater::Builder::new().build())
            .build(context)
            .unwrap();
        let update = app
            .updater_builder()
            .endpoints(vec![format!("{base}/manifest").parse().unwrap()])
            .unwrap()
            .timeout(Duration::from_secs(3))
            .no_proxy()
            .build()
            .unwrap()
            .check()
            .await
            .unwrap();
        if matches!(version, "0.1.0" | "0.0.9") {
            assert!(update.is_none());
        } else {
            assert_eq!(
                update.unwrap().download(|_, _| {}, || {}).await.is_ok(),
                expected
            );
        }
        stopped.store(true, std::sync::atomic::Ordering::Release);
        server.join().unwrap();
    }
}

#[test]
fn updater_commands_are_local_only_and_sdk_ipc_is_never_granted() {
    let directory = tempfile::tempdir().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let db = runtime.block_on(Database::open(directory.path())).unwrap();
    let app = mock_builder()
        .manage(StorageState(Ok(db.clone())))
        .manage(UpdateState::default())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::check_app_update,
            commands::install_app_update
        ])
        .build(tauri::generate_context!())
        .unwrap();
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap()
    });
    let call = |command: &str, origin: &str, body: Value| {
        get_ipc_response(
            &window,
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
    };
    for command in [
        "check_app_update",
        "install_app_update",
        "plugin:updater|check",
    ] {
        assert!(call(command, "https://example.com", json!({})).is_err());
    }
    assert!(call("plugin:updater|check", "http://tauri.localhost", json!({})).is_err());
    let result: Value = call("check_app_update", "http://tauri.localhost", json!({}))
        .unwrap()
        .deserialize()
        .unwrap();
    assert_eq!(result, json!({"enabled":false,"update":null}));
    let denied = call(
        "install_app_update",
        "http://tauri.localhost",
        json!({"version":"0.2.0","confirmed":false,"progress":"__CHANNEL__:0"}),
    )
    .unwrap_err();
    assert_eq!(denied["code"], "validation");
    let missing = call(
        "install_app_update",
        "http://tauri.localhost",
        json!({"version":"0.2.0","confirmed":true,"progress":"__CHANNEL__:0"}),
    )
    .unwrap_err();
    assert_eq!(missing["code"], "update_unavailable");
    assert!(
        !directory.path().join("backups").exists()
            || std::fs::read_dir(directory.path().join("backups"))
                .unwrap()
                .next()
                .is_none()
    );
    runtime.block_on(db.close());
}
