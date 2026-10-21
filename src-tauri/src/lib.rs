mod mail;
mod commands;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            store: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_qq_account,
            commands::add_163_account,
            commands::add_126_account,
            commands::add_gmail_account,
            commands::add_outlook_account,
            commands::add_custom_account,
            commands::localize_folder_name,
            commands::get_provider_presets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
