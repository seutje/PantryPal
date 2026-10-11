//! Explicit live acceptance check for the five authorized publisher URLs.
//! Uses production commands and ACL, isolated profiles/database, no saved meals.
use pantrypal_lib::{
    commands,
    persistence::{Database, StorageState},
    services::browser::BrowserState,
};
use serde_json::{json, Value};
use std::{sync::mpsc, time::Duration};
use tauri::{Manager, WebviewWindow};
fn eval(window: &WebviewWindow, script: &str) -> Result<Value, String> {
    let (s, r) = mpsc::channel();
    window
        .eval_with_callback(script.to_owned(), move |v| {
            let _ = s.send(v);
        })
        .map_err(|_| "Evaluate failed")?;
    let value = r
        .recv_timeout(Duration::from_secs(12))
        .map_err(|_| "Evaluate timed out")?;
    serde_json::from_str(&value).map_err(|_| "Invalid native response".into())
}
fn until(window: &WebviewWindow, script: &str) -> Result<(), String> {
    for _ in 0..120 {
        if eval(window, script).unwrap_or_default() == true {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    Err("Website or command timed out".into())
}
fn call(window: &WebviewWindow, command: &str, args: Value) -> Result<Value, String> {
    let script=format!("window.__adapterResult=null;window.__TAURI_INTERNALS__.invoke({},{}).then(value=>window.__adapterResult={{ok:true,value}},error=>window.__adapterResult={{ok:false,error}});true",json!(command),args);
    eval(window, &script)?;
    until(window, "window.__adapterResult!==null")?;
    let result = eval(window, "window.__adapterResult")?;
    if result["ok"] == true {
        Ok(result["value"].clone())
    } else {
        Err(result["error"].to_string())
    }
}
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".verification");
    let data = root.join(format!("adapter-sites-{}", std::process::id()));
    let db = tauri::async_runtime::block_on(Database::open(&data)).unwrap();
    let mut context = tauri::generate_context!();
    context.config_mut().build.dev_url = None;
    tauri::Builder::default().manage(StorageState(Ok(db.clone()))).manage(BrowserState::new(Ok(data.join("profile"))))
        .invoke_handler(tauri::generate_handler![commands::open_recipe_browser,commands::capture_recipe_browser,commands::preview_recipe_import,commands::confirm_recipe_import])
        .setup(move|app|{
            let main=app.get_webview_window("main").unwrap();main.hide()?;let handle=app.handle().clone();
            std::thread::spawn(move||{
                let result=(||->Result<Vec<Value>,String>{
                    until(&main,"!!window.__TAURI_INTERNALS__")?;
                    let sites:Vec<Value>=serde_json::from_str(include_str!("../../tests/fixtures/import-sites.json")).unwrap();
                    let mut results=vec![];
                    for site in sites {
                        let url=site["url"].as_str().unwrap();
                        call(&main,"open_recipe_browser",json!({"sourceUrl":url}))?;
                        let browser=handle.get_webview_window("recipe-browser").ok_or("Browser missing")?;
                        until(&browser,"document.readyState==='complete'")?;std::thread::sleep(Duration::from_secs(2));
                        // With valid arguments, an external page must fail at ACL.
                        let rejected=call(&browser,"capture_recipe_browser",json!({"sourceUrl":url,"permitted":true})).unwrap_err();
                        if !rejected.to_lowercase().contains("not allowed"){return Err("Remote capture was not denied by ACL".into());}
                        let preview=call(&main,"capture_recipe_browser",json!({"sourceUrl":url,"permitted":true}))?;
                        let count=preview["draft"]["meal"]["ingredients"].as_array().ok_or("Ingredients missing")?.len();
                        let steps=preview["draft"]["meal"]["instructions"].as_str().ok_or("Steps missing")?.len();
                        if count==0||steps==0||preview["source"]["format"]!="website"{return Err("Empty recipe capture".into());}
                        println!("PASS {url}: {count} ingredient lines, visible instructions, remote capture denied");
                        results.push(json!({"url":url,"ingredients":count,"instructionBytes":steps,"remoteCaptureDenied":true,"previewOnly":true}));
                        browser.close().map_err(|_|"Close failed")?;
                        until(&main,"true")?;std::thread::sleep(Duration::from_millis(500));
                    }
                    let status=tauri::async_runtime::block_on(db.status()).map_err(|_|"Database status failed")?;
                    if status.meal_count!=0{return Err("Preview wrote a meal".into());}
                    Ok(results)
                })();
                match result {Ok(results)=>{std::fs::write(root.join("adapter-sites-results.json"),serde_json::to_string_pretty(&results).unwrap()).unwrap();handle.exit(0);},Err(error)=>{eprintln!("FAIL: {error}");handle.exit(1);}}
            });Ok(())
        }).run(context).expect("Native adapter harness failed");
}
