use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use tokio::sync::Mutex;
use tracing::info;

use crate::errors::AppError;
use crate::server;

/// Holds the shutdown channel for the running server.
pub struct ServerHandle {
    pub shutdown_tx: tokio::sync::oneshot::Sender<()>,
    pub url: String,
}

/// Managed state that Tauri tracks for the server lifecycle.
pub type ManagedServerHandle = Arc<Mutex<Option<ServerHandle>>>;

/// Info returned to the frontend about the server status.
#[derive(Serialize, Clone)]
pub struct ServerInfo {
    pub running: bool,
    pub url: Option<String>,
}

/// Starts the upload server. Returns the full URL (e.g., "http://192.168.1.15:7878").
#[tauri::command]
pub async fn start_upload_server(
    app: tauri::AppHandle,
    handle: State<'_, ManagedServerHandle>,
) -> Result<String, AppError> {
    let mut guard = handle.lock().await;

    // Don't start if already running
    if guard.is_some() {
        return Err(AppError::AlreadyRunning);
    }

    // Detect the local IP address
    let local_ip = local_ip_address::local_ip()
        .map_err(|e| AppError::NetworkDetection(e.to_string()))?;

    // Resolve the uploads directory
    let upload_dir = dirs_next_or_fallback();

    // Build the URL before starting the server
    let url = format!("http://{}:{}", local_ip, server::SERVER_PORT);

    let (shutdown_tx, _addr) = server::start_server(app, upload_dir, url.clone()).await?;

    info!("Upload server accessible at {}", url);

    *guard = Some(ServerHandle {
        shutdown_tx,
        url: url.clone(),
    });

    Ok(url)
}

/// Stops the running upload server.
#[tauri::command]
pub async fn stop_upload_server(
    handle: State<'_, ManagedServerHandle>,
) -> Result<(), AppError> {
    let mut guard = handle.lock().await;

    match guard.take() {
        Some(server) => {
            // Send the shutdown signal — Axum will stop accepting connections
            let _ = server.shutdown_tx.send(());
            info!("Upload server stopped");
            Ok(())
        }
        None => Err(AppError::NotRunning),
    }
}

/// Returns the current server status and URL.
#[tauri::command]
pub async fn get_server_status(
    handle: State<'_, ManagedServerHandle>,
) -> Result<ServerInfo, AppError> {
    let guard = handle.lock().await;

    match guard.as_ref() {
        Some(server) => Ok(ServerInfo {
            running: true,
            url: Some(server.url.clone()),
        }),
        None => Ok(ServerInfo {
            running: false,
            url: None,
        }),
    }
}

/// Returns the uploads directory path, falling back to a temp directory.
fn dirs_next_or_fallback() -> std::path::PathBuf {
    if let Some(pics) = dirs::picture_dir() {
        pics.join("pdf-ninja-uploads")
    } else {
        std::env::temp_dir().join("pdf-ninja-uploads")
    }
}
