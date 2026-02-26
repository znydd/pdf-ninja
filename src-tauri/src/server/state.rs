use std::path::PathBuf;
use tauri::AppHandle;

/// Shared state injected into each Axum route handler.
#[derive(Clone)]
pub struct ServerState {
    pub app_handle: AppHandle,
    pub upload_dir: PathBuf,
    pub server_url: String,
}
