//! Explicit Windows feasibility/acceptance harness. Only this owned fixture
//! permits loopback HTTP; production URLs always require public HTTPS.
//! Run `write`, `read`, `cleared` in separate processes, in that order.
use pantrypal_lib::{
    commands,
    persistence::{Database, StorageState},
};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    time::Duration,
};
use tauri::{
    webview::{NewWindowResponse, PermissionResponse},
    Manager, WebviewUrl, WebviewWindow,
};
const ORIGIN: &str = "http://127.0.0.1:18426";

fn evaluate(window: &WebviewWindow, js: &str) -> Result<Value, String> {
    let (sender, receiver) = mpsc::channel();
    window
        .eval_with_callback(js.to_owned(), move |value| {
            let _ = sender.send(value);
        })
        .map_err(|_| "Evaluation failed")?;
    let value = receiver
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| "Evaluation timeout")?;
    serde_json::from_str(&value).map_err(|_| "Invalid result".into())
}
fn until(window: &WebviewWindow, js: &str) -> Result<(), String> {
    for _ in 0..100 {
        if evaluate(window, js).unwrap_or_default() == true {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("Fixture check timed out".into())
}
fn serve_fixture() {
    let listener = TcpListener::bind("127.0.0.1:18426").expect("Test port unavailable");
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let mut buffer = [0; 16384];
            let Ok(size) = stream.read(&mut buffer) else {
                continue;
            };
            let request = String::from_utf8_lossy(&buffer[..size]);
            let login = request.starts_with("POST /login ");
            // This is our own synthetic login. Never print cookies or headers.
            let authenticated = request.lines().any(|line| {
                line.to_lowercase().starts_with("cookie:")
                    && line.contains("pantry_fixture=fixture-only")
            });
            let recipe = include_str!("../../tests/fixtures/owned-recipe.json");
            let body = if login {
                "Logged in to owned fixture".to_owned()
            } else {
                format!(
                    r#"<!doctype html><title>Owned PantryPal fixture</title><h1>Owned fixture</h1><p id="login">{}</p><script type="application/ld+json">{recipe}</script>"#,
                    if authenticated {
                        "Authenticated fixture"
                    } else {
                        "Sign in to the fixture"
                    }
                )
            };
            let cookie = if login {
                "Set-Cookie: pantry_fixture=fixture-only; Max-Age=3600; HttpOnly; SameSite=Lax; Path=/\r\n"
            } else {
                ""
            };
            let response=format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n{cookie}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
            let _ = stream.write_all(response.as_bytes());
        }
    });
}
fn checks(browser: &WebviewWindow, main: &WebviewWindow, mode: &str) -> Result<(), String> {
    until(
        browser,
        "document.readyState==='complete'&&!!document.querySelector('#login')",
    )?;
    let commands = [
        "get_app_info",
        "get_catalog",
        "preview_recipe_import",
        "confirm_recipe_import",
        "reset_recipe_browser",
    ];
    evaluate(
        browser,
        &format!(
            r#"window.__probe={{}};for(const cmd of {}){{window.__TAURI_INTERNALS__.invoke(cmd,{{}}).then(()=>window.__probe[cmd]='ALLOWED',e=>window.__probe[cmd]=String(e));}}true"#,
            json!(commands)
        ),
    )?;
    until(browser, "Object.keys(window.__probe||{}).length===5")?;
    let denied = evaluate(browser, "window.__probe")?;
    for command in commands {
        let error = denied[command]
            .as_str()
            .ok_or("Remote denial unavailable")?;
        if !error.to_lowercase().contains("not allowed") {
            return Err(format!("Remote command {command} not rejected by ACL"));
        }
    }
    let metadata = evaluate(
        browser,
        "JSON.parse(document.querySelector('script[type=\"application/ld+json\"]').textContent)",
    )?;
    if metadata["@graph"][1]["name"] != "Owned fixture soup" {
        return Err("Native evaluation failed".into());
    }
    if mode == "write" {
        evaluate(browser,"localStorage.setItem('pantry_fixture','persisted');fetch('/login',{method:'POST',body:'fixture-only'}).then(()=>window.__login=true);true")?;
        until(browser, "window.__login===true")?;
        // Transport the owned fixture into LOCAL review commands, as deliberate
        // paste does. The HTTPS source is a test provenance placeholder; it is
        // not fetched. No product URL-validation exception is introduced.
        until(main, "!!window.__TAURI_INTERNALS__")?;
        let source = json!({"sourceUrl":"https://example.com/owned-recipe","format":"json_ld","payload":metadata.to_string(),"permitted":true});
        evaluate(
            main,
            &format!(
                r#"window.__importResult=null;window.__TAURI_INTERNALS__.invoke('preview_recipe_import',{{input:{source}}}).then(preview=>{{preview.meal.servings='4';return window.__TAURI_INTERNALS__.invoke('confirm_recipe_import',{{input:{{actionId:'native-import-fixture',source:{source},reviewed:preview.meal,confirmed:true,allowDuplicate:false}}}});}}).then(meal=>window.__importResult=meal,e=>window.__importResult={{error:String(e)}});true"#
            ),
        )?;
        until(
            main,
            "window.__importResult!==null&&window.__importResult!==undefined",
        )?;
        let saved = evaluate(main, "window.__importResult")?;
        if saved["title"] != "Owned fixture soup" {
            return Err(format!(
                "Local reviewed native import failed: {}",
                saved["error"]
            ));
        }
    } else if mode == "read" {
        until(browser,"document.querySelector('#login').textContent==='Authenticated fixture'&&localStorage.getItem('pantry_fixture')==='persisted'")?;
        evaluate(main,"window.__resetResult=null;window.__TAURI_INTERNALS__.invoke('reset_recipe_browser',{confirmed:true}).then(()=>window.__resetResult=true,()=>window.__resetResult=false);true")?;
        until(main, "window.__resetResult===true")?;
    } else {
        until(browser,"document.querySelector('#login').textContent==='Sign in to the fixture'&&localStorage.getItem('pantry_fixture')===null")?;
    }
    until(main, "!!window.__TAURI_INTERNALS__")?;
    evaluate(main,"window.__savedMeals=null;window.__TAURI_INTERNALS__.invoke('list_meals',{}).then(meals=>window.__savedMeals=meals,()=>window.__savedMeals=[]);true")?;
    until(main, "window.__savedMeals!==null")?;
    let saved = evaluate(main, "window.__savedMeals")?;
    if saved.as_array().map(Vec::len) != Some(1)
        || saved[0]["title"] != "Owned fixture soup"
        || saved[0]["ingredients"][0]["rawText"] != "½ US cup rice"
    {
        return Err("Reviewed meal did not survive process restart".into());
    }
    Ok(())
}
fn main() {
    let mode = std::env::args().nth(1).expect("Use write, read or cleared");
    assert!(["write", "read", "cleared"].contains(&mode.as_str()));
    serve_fixture();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".verification");
    std::fs::create_dir_all(&root).unwrap();
    let data_name = if mode == "write" {
        let name = format!("browser-spike-data-{}", std::process::id());
        std::fs::write(root.join("browser-spike-last-data.txt"), &name).unwrap();
        name
    } else {
        std::fs::read_to_string(root.join("browser-spike-last-data.txt")).unwrap()
    };
    assert!(
        data_name.starts_with("browser-spike-data-")
            && data_name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    );
    let data = root.join(data_name);
    let db = tauri::async_runtime::block_on(Database::open(&data)).unwrap();
    let mut context = tauri::generate_context!();
    context.config_mut().build.dev_url = None;
    let browser_state =
        pantrypal_lib::services::browser::BrowserState::new(Ok(root.join("browser-spike-profile")));
    tauri::Builder::default().manage(StorageState(Ok(db))).manage(browser_state)
        .invoke_handler(tauri::generate_handler![commands::get_app_info,commands::get_catalog,commands::preview_recipe_import,commands::confirm_recipe_import,commands::reset_recipe_browser,commands::list_meals])
        .setup(move|app|{
            let main=app.get_webview_window("main").expect("Local window missing");main.hide()?;
            let origin:tauri::Url=ORIGIN.parse().unwrap();
            let expected=origin.origin();
            let browser=tauri::WebviewWindowBuilder::new(app,"recipe-browser",WebviewUrl::External(origin))
                .title("PantryPal isolated feasibility fixture").data_directory(root.join("browser-spike-profile"))
                .on_navigation(move|url|url.origin()==expected)
                .on_new_window(|_,_|NewWindowResponse::Deny).on_download(|_,_|false)
                .on_permission_request(|_,_|PermissionResponse::Deny).build()?;
            let handle=app.handle().clone();
            std::thread::spawn(move||{
                let result=checks(&browser,&main,&mode);
                let (message,code)=match result{Ok(())=>(format!("PASS {mode}: native ACL denial, owned metadata, reviewed import, session persistence/reset\n"),0),Err(error)=>(error,1)};
                std::fs::write(root.join(format!("browser-spike-{mode}.txt")),message).unwrap();handle.exit(code);
            });Ok(())
        }).run(context).expect("Spike runtime failed");
}
