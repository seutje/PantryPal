//! Narrow UTF-16 Windows writer and bounded opt-in recipe URL reader.
//! Ownership follows Microsoft's Open/Empty/Set/CloseClipboard contract.
#[cfg(windows)]
mod windows {
    use crate::error::{AppError, AppResult};
    use std::ffi::c_void;
    #[link(name = "user32")]
    extern "system" {
        fn OpenClipboard(owner: isize) -> i32;
        fn EmptyClipboard() -> i32;
        fn SetClipboardData(format: u32, memory: *mut c_void) -> *mut c_void;
        fn CloseClipboard() -> i32;
        fn GetClipboardData(format: u32) -> *mut c_void;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalAlloc(flags: u32, size: usize) -> *mut c_void;
        fn GlobalLock(memory: *mut c_void) -> *mut c_void;
        fn GlobalUnlock(memory: *mut c_void) -> i32;
        fn GlobalFree(memory: *mut c_void) -> *mut c_void;
        fn GlobalSize(memory: *mut c_void) -> usize;
    }
    struct Clipboard;
    impl Drop for Clipboard {
        fn drop(&mut self) {
            unsafe {
                CloseClipboard();
            }
        }
    }
    /// Opt-in caller receives only a validated public recipe URL, never arbitrary
    /// clipboard text. The read is bounded while the clipboard remains open.
    pub fn recipe_url(owner: isize) -> AppResult<Option<String>> {
        if owner == 0 {
            return Err(AppError::storage());
        }
        unsafe {
            if OpenClipboard(owner) == 0 {
                return Ok(None);
            }
            let _clipboard = Clipboard;
            let handle = GetClipboardData(13);
            if handle.is_null() {
                return Ok(None);
            }
            let bytes = GlobalSize(handle);
            if bytes == 0 || bytes > 8192 || !bytes.is_multiple_of(2) {
                return Ok(None);
            }
            let pointer = GlobalLock(handle).cast::<u16>();
            if pointer.is_null() {
                return Ok(None);
            }
            let data = std::slice::from_raw_parts(pointer, bytes / 2);
            let text = data
                .iter()
                .position(|c| *c == 0)
                .and_then(|end| String::from_utf16(&data[..end]).ok());
            GlobalUnlock(handle);
            Ok(text.and_then(|s| {
                crate::importers::url::validate(s.trim())
                    .ok()
                    .map(|u| u.to_string())
            }))
        }
    }
    struct Memory(*mut c_void);
    impl Drop for Memory {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    GlobalFree(self.0);
                }
            }
        }
    }
    pub fn write(owner: isize, text: &str) -> AppResult<()> {
        if owner == 0 || text.contains('\0') || text.len() > 4_000_000 {
            return Err(AppError::validation(
                "Shopping text is too large or invalid for copying.",
            ));
        }
        let error = || AppError {
            code: "clipboard_unavailable",
            message: "Could not copy. Another app may be using the clipboard; try again.".into(),
        };
        let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        // Allocate before emptying: a failed allocation cannot erase clipboard.
        // All pointers below come from GlobalAlloc, are checked, and remain
        // locked until the bounded slice is copied. Windows owns successful sets.
        unsafe {
            let mut memory = Memory(GlobalAlloc(0x0002, utf16.len() * 2));
            if memory.0.is_null() {
                return Err(error());
            }
            let pointer = GlobalLock(memory.0).cast::<u16>();
            if pointer.is_null() {
                return Err(error());
            }
            std::ptr::copy_nonoverlapping(utf16.as_ptr(), pointer, utf16.len());
            GlobalUnlock(memory.0);
            if OpenClipboard(owner) == 0 {
                return Err(error());
            }
            let _clipboard = Clipboard;
            if EmptyClipboard() == 0 || SetClipboardData(13, memory.0).is_null() {
                return Err(error());
            }
            memory.0 = std::ptr::null_mut();
        }
        Ok(())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[link(name = "user32")]
        extern "system" {
            fn GetDesktopWindow() -> isize;
            fn GetClipboardData(format: u32) -> *mut c_void;
        }
        fn fixture_write(owner: isize, text: &str) {
            // Windows clipboard listeners may briefly hold the clipboard after
            // each change. Only the explicit fixture test retries; product UI
            // still reports contention and offers a deliberate retry.
            for _ in 0..40 {
                if write(owner, text).is_ok() {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            panic!("Fixture could not obtain the Windows clipboard");
        }
        #[test]
        #[ignore = "Explicit Windows acceptance check writes fixture text to the clipboard"]
        fn windows_clipboard_roundtrip() {
            // Exercise generation -> adjustment -> backend text -> native paste
            // data, using temporary storage rather than the household database.
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let dir = tempfile::tempdir().unwrap();
            let fixture=runtime.block_on(async {
                let db=crate::persistence::Database::open(dir.path()).await.unwrap();
                let meal=db.create_meal(serde_json::from_str(include_str!("../../../tests/fixtures/demo-meal.json")).unwrap()).await.unwrap();
                for index in 0..2 {db.apply_plan(serde_json::from_value(serde_json::json!({"actionId":format!("plan-{index}"),"weekStart":"2020-12-28","expectedRevision":index,"operation":{"kind":"add","entryId":format!("entry-{index}"),"mealId":meal.id,"localDate":null,"servings":"4"}})).unwrap()).await.unwrap();}
                let mut view=db.change_shopping(serde_json::from_value(serde_json::json!({"actionId":"refresh","weekStart":"2020-12-28","expectedRevision":0,"operation":{"kind":"refresh"}})).unwrap()).await.unwrap();
                let id=view.items.iter().find(|i|i.item.ingredient_id.as_deref()==Some("ingredient-broccoli")).unwrap().item.id.clone();
                for operation in [serde_json::json!({"kind":"adjust","itemId":id,"amount":"1 1/2","unitId":"kg"}),serde_json::json!({"kind":"check","itemId":id,"checked":true})] {view=db.change_shopping(serde_json::from_value(serde_json::json!({"actionId":uuid::Uuid::new_v4().to_string(),"weekStart":"2020-12-28","expectedRevision":view.revision,"operation":operation})).unwrap()).await.unwrap();}
                assert!(view.text.contains("[x] Broccoli — 1½ kg"));db.close().await;view.text
            });
            let verification = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join(".verification");
            std::fs::create_dir_all(&verification).unwrap();
            std::fs::write(verification.join("phase5-clipboard-expected.txt"), &fixture).unwrap();
            unsafe {
                fixture_write(GetDesktopWindow(), &fixture);
                assert_ne!(OpenClipboard(GetDesktopWindow()), 0);
                let _clipboard = Clipboard;
                let handle = GetClipboardData(13);
                assert!(!handle.is_null());
                let pointer = GlobalLock(handle).cast::<u16>();
                assert!(!pointer.is_null());
                let expected: Vec<u16> = fixture.encode_utf16().chain(std::iter::once(0)).collect();
                assert_eq!(
                    std::slice::from_raw_parts(pointer, expected.len()),
                    expected.as_slice()
                );
                GlobalUnlock(handle);
            }
            // Paste into another application's native text control while the
            // fixture writer is still alive. Do not assume clipboard contents
            // remain unchanged between independently launched verification jobs.
            let paste=std::process::Command::new("powershell.exe")
                .args(["-NoProfile","-NonInteractive","-STA","-Command",r#"Add-Type -AssemblyName System.Windows.Forms; $control=[Windows.Forms.TextBox]::new(); $control.Multiline=$true; $control.CreateControl(); $control.Paste(); $expected=[IO.File]::ReadAllText($env:PANTRYPAL_PASTE_FIXTURE); if ($control.Text -cne $expected) { throw 'Generated grocery paste mismatch' }; $control.Dispose()"#])
                .env("PANTRYPAL_PASTE_FIXTURE",verification.join("phase5-clipboard-expected.txt"))
                .output().unwrap();
            assert!(
                paste.status.success(),
                "Separate Windows paste failed: {}",
                String::from_utf8_lossy(&paste.stderr)
            );
            unsafe {
                let owner = GetDesktopWindow();
                for (text, expected) in [
                    ("https://www.budgetbytes.com/chicken-noodle-soup/", true),
                    ("https://user:secret@example.com/recipe", false),
                    ("http://127.0.0.1/recipe", false),
                    ("ordinary clipboard text", false),
                ] {
                    fixture_write(owner, text);
                    assert_eq!(recipe_url(owner).unwrap().is_some(), expected);
                }
                fixture_write(owner, &"x".repeat(5000));
                assert!(recipe_url(owner).unwrap().is_none());
            }
        }
    }
}
#[cfg(windows)]
pub use windows::recipe_url;
#[cfg(windows)]
pub use windows::write;
