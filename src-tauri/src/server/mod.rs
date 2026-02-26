pub mod mobile_page;
pub mod routes;
pub mod state;

use axum::{
    extract::{DefaultBodyLimit, Extension, Path},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::{net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::errors::AppError;
use state::ServerState;

/// The port the upload server listens on.
pub const SERVER_PORT: u16 = 7878;

/// Serves a single uploaded image by filename.
async fn serve_image(
    Extension(state): Extension<ServerState>,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    let file_path = state.upload_dir.join(&filename);

    match tokio::fs::read(&file_path).await {
        Ok(data) => {
            // Determine content type from extension
            let content_type = match file_path.extension().and_then(|e| e.to_str()) {
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("png") => "image/png",
                Some("gif") => "image/gif",
                Some("webp") => "image/webp",
                Some("svg") => "image/svg+xml",
                Some("bmp") => "image/bmp",
                _ => "application/octet-stream",
            };
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                data,
            )
                .into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Starts the HTTP server in the background.
///
/// Returns a shutdown sender and the socket address.
/// Drop or send on the sender to stop the server gracefully.
pub async fn start_server(
    app_handle: tauri::AppHandle,
    upload_dir: PathBuf,
    server_url: String,
) -> Result<(tokio::sync::oneshot::Sender<()>, SocketAddr), AppError> {
    // Ensure the upload directory exists
    tokio::fs::create_dir_all(&upload_dir)
        .await
        .map_err(AppError::Io)?;

    let shared_state = ServerState {
        app_handle,
        upload_dir,
        server_url,
    };

    // CORS: allow any origin so the mobile browser can POST to the server
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(routes::serve_mobile_page))
        .route("/upload", axum::routing::post(routes::handle_upload))
        .route("/images/{filename}", get(serve_image))
        .layer(cors)
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB
        .layer(Extension(shared_state));

    let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));
    let listener = TcpListener::bind(addr).await.map_err(|e| {
        AppError::Server(format!("Failed to bind to port {}: {}", SERVER_PORT, e))
    })?;

    let local_addr = listener.local_addr().map_err(|e| {
        AppError::Server(format!("Failed to get local address: {}", e))
    })?;

    info!("Upload server started on {}", local_addr);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
                info!("Server received shutdown signal");
            })
            .await
        {
            tracing::error!("Server error: {}", e);
        }
    });

    Ok((shutdown_tx, local_addr))
}
