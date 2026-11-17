mod commands;
pub mod error;
pub mod excel;
pub mod model;
pub mod output;
pub mod store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::open_workbook,
            commands::extract_workbook,
            commands::diagnose_workbook,
            commands::extract_vba,
            commands::get_recent,
            commands::remove_recent,
            commands::clear_recent,
            commands::check_recent,
            commands::clear_cache,
            commands::export_outputs,
            commands::get_settings,
            commands::save_settings,
            commands::builtin_templates,
            commands::preview_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
