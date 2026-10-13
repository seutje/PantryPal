//! Native acceptance using bundled assets and isolated data; no recipe websites.
use pantrypal_lib::{
    commands,
    persistence::{Database, StorageState},
    services::browser::BrowserState,
};
use serde_json::{json, Value};
use std::{sync::mpsc, time::Duration};
use tauri::WebviewWindow;
fn eval(window: &WebviewWindow, script: &str) -> Result<Value, String> {
    let (s, r) = mpsc::channel();
    window
        .eval_with_callback(script.to_owned(), move |v| {
            let _ = s.send(v);
        })
        .map_err(|_| "Native evaluation failed")?;
    serde_json::from_str(
        &r.recv_timeout(Duration::from_secs(12))
            .map_err(|_| "Native response timed out")?,
    )
    .map_err(|_| "Invalid result".into())
}
fn until(window: &WebviewWindow, script: &str) -> Result<(), String> {
    let start = std::time::Instant::now();
    for _ in 0..120 {
        if start.elapsed() > Duration::from_secs(30) {
            break;
        }
        if eval(window, script).unwrap_or_default() == true {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!(
        "Native acceptance timed out: {script}; fixture UI: {}",
        eval(window, "document.body?.innerText?.slice(0,1500)").unwrap_or_default()
    ))
}
fn call(window: &WebviewWindow, command: &str, args: Value) -> Result<Value, String> {
    eval(window,&format!("window.__phase8=null;window.__TAURI_INTERNALS__.invoke({},{}).then(value=>window.__phase8={{ok:true,value}},error=>window.__phase8={{ok:false,error}});true",json!(command),args))?;
    until(window, "window.__phase8!==null")?;
    let result = eval(window, "window.__phase8")?;
    if result["ok"] == true {
        Ok(result["value"].clone())
    } else {
        Err("Native command failed".into())
    }
}
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".verification");
    let data = root.join(format!("phase8-{}", uuid::Uuid::new_v4()));
    let db = tauri::async_runtime::block_on(Database::open(&data)).unwrap();
    let mut context = tauri::generate_context!();
    context.config_mut().build.dev_url = None;
    context.config_mut().app.windows.clear();
    tauri::Builder::default().manage(StorageState(Ok(db.clone()))).manage(pantrypal_lib::services::updates::UpdateState::default()).manage(BrowserState::new(Ok(data.join("profile"))))
      .invoke_handler(tauri::generate_handler![commands::check_app_update,commands::install_app_update,commands::convert_measurement,commands::create_ingredient,commands::create_field,commands::create_meal,commands::copy_shopping,commands::get_app_info,commands::get_storage_status,commands::get_catalog,commands::get_week,commands::apply_plan,commands::get_shopping,commands::change_shopping,commands::search_meals,commands::get_measurement_preferences,commands::get_usability_preferences,commands::save_usability_preferences,commands::get_backup_info,commands::export_backup,commands::preview_restore,commands::restore_backup,commands::read_recipe_clipboard_url])
      .setup(move|app|{
        let window=tauri::WebviewWindowBuilder::new(app,"main",tauri::WebviewUrl::App("index.html".into())).title("PantryPal Phase 8 isolated acceptance").data_directory(data.join("shell-profile")).visible(false).build()?;let handle=app.handle().clone();
        std::thread::spawn(move||{
          let result=(||->Result<(),String>{
            println!("Checking bundled Week startup");
            until(&window,"document.querySelector('h1')?.textContent==='Week'&&document.querySelectorAll('section button').length>=7")?;
            if eval(&window,"performance.getEntriesByType('resource').every(e=>{const u=new URL(e.name);return !['http:','https:'].includes(u.protocol)||['tauri.localhost','ipc.localhost'].includes(u.hostname);})")?!=true{return Err("Unexpected remote core resource".into());}
            for (unit,exact) in [("cup-metric","250"),("cup-us","473176473/2000000"),("cup-imperial","454609/1600")]{let converted=call(&window,"convert_measurement",json!({"input":{"amount":"1","fromUnit":unit,"toUnit":"ml","ingredientId":null,"purpose":"quantity"}}))?;if converted["exact"]!=exact{return Err("Regional cup conversion mismatch".into());}}
            for (unit,to) in [("cup","ml"),("g","ml"),("piece","g")]{let result=call(&window,"convert_measurement",json!({"input":{"amount":"1","fromUnit":unit,"toUnit":to,"ingredientId":null,"purpose":"quantity"}}))?;if result["status"]!="cannot_convert"{return Err("Ambiguous/incompatible unit silently converted".into());}}
            let ingredient=call(&window,"create_ingredient",json!({"input":{"name":"Acceptance broccoli","categoryId":null,"sectionId":null,"preferredUnitId":"g","aliases":[],"seasonMonths":[],"notes":"Isolated fixture"}}))?;
            let field=call(&window,"create_field",json!({"input":{"label":"Acceptance favourite","kind":"boolean","description":"Fixture","options":[],"minimum":null,"maximum":null,"unitLabel":null,"sortOrder":0}}))?;
            let mut recipe:Value=serde_json::from_str(include_str!("../../tests/fixtures/demo-meal.json")).unwrap();
            recipe["ingredients"][1]["ingredientId"]=ingredient["id"].clone();
            recipe["customFields"]=json!([{"fieldId":field["id"],"value":{"kind":"boolean","value":false}}]);
            let meal=call(&window,"create_meal",json!({"input":recipe}))?;
            let meal_id=meal["id"].as_str().ok_or("Meal ID missing")?.to_owned();
            call(&window,"apply_plan",json!({"input":{"actionId":"offline-add","weekStart":"2020-12-28","expectedRevision":0,"operation":{"kind":"add","entryId":"offline-entry","mealId":meal_id,"localDate":"2020-12-29","servings":"2"}}}))?;
            let shopping=call(&window,"change_shopping",json!({"input":{"actionId":"offline-shopping","weekStart":"2020-12-28","expectedRevision":0,"operation":{"kind":"refresh"}}}))?;
            if shopping["items"].as_array().is_none_or(|r|r.is_empty()){return Err("Offline shopping missing".into());}
            if !shopping["items"].as_array().ok_or("Items missing")?.iter().any(|i|i["ingredientId"]==ingredient["id"]&&i["numerator"]=="250"){return Err("Planned servings did not scale broccoli to 250 g".into());}
            call(&window,"copy_shopping",json!({"weekStart":"2020-12-28","expectedRevision":shopping["revision"]}))?;
            let mut adjusted=shopping;
            let item=adjusted["items"].as_array().ok_or("Items missing")?.iter().find(|i|i["ingredientId"]==ingredient["id"]).ok_or("Ingredient missing")?["id"].clone();
            for (index,operation) in [json!({"kind":"adjust","itemId":item,"amount":"2","unitId":"kg"}),json!({"kind":"manual","itemId":"acceptance-soap","label":"Acceptance soap","amount":"2","unitId":"piece","sectionId":null}),json!({"kind":"refresh"})].into_iter().enumerate(){adjusted=call(&window,"change_shopping",json!({"input":{"actionId":format!("acceptance-edit-{index}"),"weekStart":"2020-12-28","expectedRevision":adjusted["revision"],"operation":operation}}))?;}
            if !adjusted["items"].as_array().ok_or("Items missing")?.iter().any(|i|i["id"]==item&&i["adjustment"]["amount"]=="2"&&i["adjustment"]["unitId"]=="kg"){return Err("Override lost on refresh".into());}
            if !adjusted["text"].as_str().ok_or("Shopping text missing")?.contains("Acceptance soap"){return Err("Manual item lost on refresh".into());}
            eval(&window,"Array.from(document.querySelectorAll('nav button')).find(b=>b.textContent==='Settings').click();true")?;
            until(&window,"!!Array.from(document.querySelectorAll('button')).find(b=>b.textContent==='Export JSON backup')")?;
            eval(&window,"document.querySelector('nav button').focus();document.querySelector('nav button').dispatchEvent(new KeyboardEvent('keydown',{key:'End',bubbles:true}));true")?;
            if eval(&window,"document.activeElement.textContent==='Settings'")?!=true{return Err("Navigation focus failed".into());}
            if eval(&window,"Array.from(document.querySelectorAll('input,select,textarea')).every(e=>e.labels?.length>0)")?!=true{return Err("Unlabelled Settings control".into());}
            eval(&window,"Array.from(document.querySelectorAll('button')).find(b=>b.textContent==='Export JSON backup').click();true")?;
            until(&window,"document.body.textContent.includes('Backup saved:')")?;
            eval(&window,"Array.from(document.querySelectorAll('button')).find(b=>b.textContent==='Check for updates').click();true")?;
            until(&window,"document.body.textContent.includes('Update checks are available in installed release builds.')")?;
            let exported=call(&window,"export_backup",json!({}))?;
            let payload=std::fs::read_to_string(exported["path"].as_str().ok_or("Backup path missing")?).map_err(|_|"Backup read failed")?;
            let changed=tauri::async_runtime::block_on(db.meal(&meal_id)).map_err(|_|"Fixture read failed")?;
            let mut input=changed.data;input.notes="Changed after export".into();
            tauri::async_runtime::block_on(db.update_meal(&meal_id,changed.revision,input)).map_err(|_|"Fixture edit failed")?;
            eval(&window,&format!("const file=new File([{}],'owned-backup.json',{{type:'application/json'}});const transfer=new DataTransfer();transfer.items.add(file);const input=document.querySelector('input[type=file]');input.files=transfer.files;input.dispatchEvent(new Event('change',{{bubbles:true}}));const mode=Array.from(document.querySelectorAll('select')).find(s=>Array.from(s.options).some(o=>o.value==='replace'));mode.value='replace';mode.dispatchEvent(new Event('change',{{bubbles:true}}));true",json!(payload)))?;
            until(&window,"Array.from(document.querySelectorAll('button')).some(b=>b.textContent==='Review backup'&&!b.disabled)")?;
            eval(&window,"Array.from(document.querySelectorAll('button')).find(b=>b.textContent==='Review backup').click();true")?;
            until(&window,"!!document.querySelector('[aria-label=\"Confirm backup import\"]')")?;
            if eval(&window,"Array.from(document.querySelectorAll('button')).find(b=>b.textContent==='Confirm replacement').disabled")?!=true{return Err("Restore did not require confirmation".into());}
            eval(&window,"document.querySelector('[aria-label=\"Confirm backup import\"] input[type=checkbox]').click();true")?;
            until(&window,"Array.from(document.querySelectorAll('button')).some(b=>b.textContent==='Confirm replacement'&&!b.disabled)")?;
            eval(&window,"Array.from(document.querySelectorAll('button')).find(b=>b.textContent==='Confirm replacement').click();true")?;
            until(&window,"document.body.textContent.includes('Import completed. Recovery backup:')")?;
            let restored=tauri::async_runtime::block_on(db.meal(&meal_id)).map_err(|_|"Restore read failed")?;
            if restored.data.notes!="Fixture only."{return Err("Native restore did not preserve recipe".into());}
            eval(&window,"window.__blocked=null;fetch('https://example.com').then(()=>window.__blocked=false,()=>window.__blocked=true);true")?;
            until(&window,"window.__blocked!==null")?;
            if eval(&window,"window.__blocked")?!=true{return Err("Remote shell network not blocked".into());}
            tauri::async_runtime::block_on(db.close());
            let reopened=tauri::async_runtime::block_on(Database::open(&data)).map_err(|_|"Reopen failed")?;
            if tauri::async_runtime::block_on(reopened.get_week("2020-12-28")).map_err(|_|"Plan reopen failed")?.entries.len()!=1{return Err("Plan did not survive restore/reopen".into());}
            tauri::async_runtime::block_on(reopened.close());
            Ok(())
          })();
          let(message,code)=match result{Ok(())=>("PASS: native ingredient, custom boolean field, meal, planned servings, shopping, Windows clipboard, override/manual refresh; bundled offline planning/shopping, Settings keyboard labels/focus, native export, reviewed confirmed file restore, recovery backup, restart and blocked remote shell fetch".to_owned(),0),Err(error)=>(format!("FAIL: {error}"),1)};
          println!("{message}");std::fs::write(root.join("phase8-acceptance.txt"),message).unwrap();handle.exit(code);
        });Ok(())
      }).run(context).expect("Native Phase 8 acceptance failed");
}
