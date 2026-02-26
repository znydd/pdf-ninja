mod commands;
mod errors;
mod server;

use commands::server::ManagedServerHandle;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(Mutex::new(None::<commands::server::ServerHandle>)) as ManagedServerHandle)
        .invoke_handler(tauri::generate_handler![
            commands::greet::greet,
            commands::server::start_upload_server,
            commands::server::stop_upload_server,
            commands::server::get_server_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
