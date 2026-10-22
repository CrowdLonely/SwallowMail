mod mail;
mod commands;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            store: std::sync::Mutex::new(None),
            imap_sessions: std::sync::Mutex::new(std::collections::HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::init_store,
            commands::add_qq_account,
            commands::add_163_account,
            commands::add_126_account,
            commands::add_gmail_account,
            commands::add_outlook_account,
            commands::add_custom_account,
            commands::save_account_to_db,
            commands::list_accounts_from_db,
            commands::delete_account_from_db,
            commands::update_account_in_db,
            commands::connect_and_fetch_folders,
            commands::save_folders_to_db,
            commands::list_folders_from_db,
            commands::sync_folder_messages,
            commands::list_messages_from_db,
            commands::fetch_message_body,
            commands::send_mail,
            commands::toggle_message_read,
            commands::toggle_message_flagged,
            commands::delete_message,
            commands::search_messages,
            commands::get_thread_messages,
            commands::save_attachment,
            commands::save_draft,
            commands::disconnect_imap,
            commands::refresh_unread_counts,
            commands::move_message,
            commands::localize_folder_name,
            commands::get_provider_presets,
            commands::list_all_inbox_messages,
            commands::save_rule,
            commands::list_rules,
            commands::delete_rule,
            commands::save_folder_mapping,
            commands::get_folder_mapping,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}