//! Explicit compatibility visits to the owner's five requested URLs. Reads
//! only the displayed page title/load state; never recipe data or credentials.
//! Dedicated disposable profiles, no household database and no native commands.
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};
use tauri::{
    webview::{NewWindowResponse, PermissionResponse},
    Manager, WebviewUrl,
};
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".verification");
    let profile = root.join(format!("requested-sites-profile-{}", std::process::id()));
    let sites: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../tests/fixtures/import-sites.json")).unwrap();
    tauri::Builder::default().setup(move|app|{
        if let Some(main)=app.get_webview_window("main"){main.hide()?;}
        let handle=app.handle().clone();
        std::thread::spawn(move||{
            let mut results=vec![];
            for (index,site) in sites.iter().enumerate(){
                let requested=site["url"].as_str().unwrap();
                let url=pantrypal_lib::importers::url::validate(requested).unwrap();let origin=url.origin();
                let window=tauri::WebviewWindowBuilder::new(&handle,format!("site-test-{index}"),WebviewUrl::External(url))
                    .title("PantryPal requested recipe compatibility test").data_directory(profile.join(index.to_string()))
                    .on_navigation(move|url|pantrypal_lib::importers::url::validate(url.as_str()).is_ok()&&url.origin()==origin)
                    .on_new_window(|_,_|NewWindowResponse::Deny).on_download(|_,_|false)
                    .on_permission_request(|_,_|PermissionResponse::Deny).build().unwrap();
                let deadline=Instant::now()+Duration::from_secs(25);let mut displayed=serde_json::Value::Null;
                while Instant::now()<deadline {
                    let (sender,receiver)=mpsc::channel();
                    window.eval_with_callback("({ready:document.readyState==='complete',title:document.title.slice(0,200)})",move|s|{let _=sender.send(s);}).unwrap();
                    if let Ok(value)=receiver.recv_timeout(Duration::from_secs(3)){
                        if let Ok(value)=serde_json::from_str::<serde_json::Value>(&value){if value["ready"]==true&&!value["title"].as_str().unwrap_or_default().is_empty(){displayed=value;break;}}
                    }
                    std::thread::sleep(Duration::from_millis(250));
                }
                results.push(serde_json::json!({"requestedUrl":requested,"displayedPage":displayed,"automaticRecipeExtraction":false}));
                println!("Visit {}: {}",index+1,displayed["title"].as_str().unwrap_or("no completed page within 25 seconds"));
                window.close().unwrap();
            }
            std::fs::create_dir_all(&root).unwrap();std::fs::write(root.join("requested-site-visits.json"),serde_json::to_string_pretty(&results).unwrap()).unwrap();handle.exit(0);
        });Ok(())
    }).run(tauri::generate_context!()).expect("Compatibility runtime failed");
}
