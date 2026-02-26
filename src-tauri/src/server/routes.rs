use axum::{
    extract::{Extension, Multipart},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json},
};
use serde_json::json;
use tauri::Emitter;
use tokio::fs;
use tracing::{error, info};
use uuid::Uuid;

use super::mobile_page::MOBILE_PAGE_HTML;
use super::state::ServerState;

/// GET / — serves the mobile upload page to the phone browser.
pub async fn serve_mobile_page() -> Html<&'static str> {
    Html(MOBILE_PAGE_HTML)
}

/// POST /upload — handles multipart image uploads from the mobile browser.
pub async fn handle_upload(
    Extension(state): Extension<ServerState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut saved_count: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        // Validate content type — only accept images
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        if !content_type.starts_with("image/") {
            errors.push(format!("Rejected non-image file (type: {})", content_type));
            continue;
        }

        // Determine file extension from content type
        let extension = match content_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "image/svg+xml" => "svg",
            "image/bmp" => "bmp",
            "image/heic" => "heic",
            "image/heif" => "heif",
            _ => "bin",
        };

        // Read file bytes
        let data = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to read upload field: {}", e);
                errors.push(format!("Failed to read file: {}", e));
                continue;
            }
        };

        // Enforce size limit (20 MB per file)
        const MAX_FILE_SIZE: usize = 20 * 1024 * 1024;
        if data.len() > MAX_FILE_SIZE {
            errors.push("File exceeds 20MB limit".to_string());
            continue;
        }

        // Generate a unique filename and save
        let filename = format!("{}.{}", Uuid::new_v4(), extension);
        let save_path = state.upload_dir.join(&filename);

        if let Err(e) = fs::write(&save_path, &data).await {
            error!("Failed to save file {}: {}", filename, e);
            errors.push(format!("Failed to save {}: {}", filename, e));
            continue;
        }

        // Encode as base64 data URL for the desktop webview
        // (WebKitGTK blocks HTTP requests to localhost, but data URLs always work)
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
        let data_url = format!("data:{};base64,{}", content_type, b64);
        if let Err(e) = state.app_handle.emit("image-received", &data_url) {
            error!("Failed to emit event: {}", e);
        }

        info!("Saved uploaded image: {}", filename);
        saved_count += 1;
    }

    let status = if errors.is_empty() {
        StatusCode::OK
    } else if saved_count > 0 {
        StatusCode::OK // partial success
    } else {
        StatusCode::BAD_REQUEST
    };

    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "saved": saved_count,
            "errors": errors,
        })),
    )
}
