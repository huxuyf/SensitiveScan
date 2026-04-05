// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod scanner;
mod db;
mod patterns;
mod models;
mod commands;

pub use commands::*;

use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = db::Database::new().expect("failed to initialize database");
    let db_arc = Arc::new(db);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(db_arc)
        .invoke_handler(tauri::generate_handler![
            commands::select_folder,
            commands::start_scan,
            commands::pause_scan,
            commands::resume_scan,
            commands::stop_scan,
            commands::get_aggregated_results,
            commands::open_file,
            commands::delete_file,
            commands::export_results,
            commands::get_scan_stats,
            commands::clear_results,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
