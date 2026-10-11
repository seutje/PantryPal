use crate::error::{AppError, AppResult};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
pub struct BrowserState {
    profile: AppResult<PathBuf>,
    busy: Arc<AtomicBool>,
}
impl BrowserState {
    /// Backend-owned profile, never supplied through IPC. Examples use isolated
    /// test directories; production resolves the per-user app-data directory.
    pub fn new(profile: AppResult<PathBuf>) -> Self {
        Self {
            profile,
            busy: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn profile(&self) -> AppResult<PathBuf> {
        self.profile.clone()
    }
    pub fn begin(&self) -> AppResult<BrowserTask> {
        self.busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| {
                AppError::validation("A browser operation is in progress. Wait and retry.")
            })?;
        Ok(BrowserTask(self.busy.clone()))
    }
}
pub struct BrowserTask(Arc<AtomicBool>);
impl Drop for BrowserTask {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
pub async fn clear_profile(window: &tauri::WebviewWindow) -> AppResult<()> {
    #[cfg(windows)]
    {
        use webview2_com::{
            ClearBrowsingDataCompletedHandler,
            Microsoft::Web::WebView2::Win32::{ICoreWebView2Profile2, ICoreWebView2_13},
        };
        use windows_core::Interface;
        let (sender, receiver) = std::sync::mpsc::channel();
        window
            .with_webview(move |native| {
                let failed = sender.clone();
                // SAFETY: Tauri dispatches this closure to WebView2's owning UI
                // thread. COM interfaces stay in this closure; only a boolean
                // completion result crosses threads. No site data is read.
                let start = unsafe {
                    (|| -> windows_core::Result<()> {
                        let profile = native
                            .controller()
                            .CoreWebView2()?
                            .cast::<ICoreWebView2_13>()?
                            .Profile()?
                            .cast::<ICoreWebView2Profile2>()?;
                        profile.ClearBrowsingDataAll(&ClearBrowsingDataCompletedHandler::create(
                            Box::new(move |result| {
                                let _ = sender.send(result.is_ok());
                                Ok(())
                            }),
                        ))
                    })()
                };
                if start.is_err() {
                    let _ = failed.send(false);
                }
            })
            .map_err(|_| reset_error())?;
        let success = tauri::async_runtime::spawn_blocking(move || {
            receiver
                .recv_timeout(std::time::Duration::from_secs(30))
                .unwrap_or(false)
        })
        .await
        .map_err(|_| reset_error())?;
        if !success {
            return Err(reset_error());
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        Err(reset_error())
    }
}
fn reset_error() -> AppError {
    AppError::validation("Website sessions could not be confirmed cleared. Keep PantryPal open and retry; recipes are unchanged.")
}
