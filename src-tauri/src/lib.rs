// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod scanner;
mod db;
mod patterns;
mod models;
mod commands;

pub use commands::*;

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
            commands::get_scan_results,
            commands::export_results,
            commands::get_history,
            commands::delete_history,
            commands::add_whitelist,
            commands::get_whitelist,
            commands::delete_whitelist,
            commands::get_scan_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
