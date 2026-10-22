use tauri::State;
use std::sync::Mutex;
use std::collections::HashMap;
use crate::mail::model::{MailAccount, MailFolder, MailMessage, MailAddress, MailRule, FolderMapping};
use crate::mail::account;
use crate::mail::imap;
use crate::mail::imap::client;
use crate::mail::imap::sync;
use crate::mail::smtp;
use crate::mail::store::MailStore;

pub struct AppState {
    pub store: Mutex<Option<MailStore>>,
    pub imap_sessions: Mutex<HashMap<String, client::TlsSession>>,
}

fn get_store(state: &AppState) -> Result<std::sync::MutexGuard<Option<MailStore>>, String> {
    let store = state.store.lock().map_err(|e| format!("获取存储锁失败: {}", e))?;
    Ok(store)
}

fn ensure_store(state: &AppState) -> Result<std::sync::MutexGuard<Option<MailStore>>, String> {
    let store = get_store(state)?;
    if store.is_none() {
        return Err("数据库未初始化，请先调用 init_store".to_string());
    }
    Ok(store)
}

fn get_imap_session(state: &AppState, account: &MailAccount) -> Result<client::TlsSession, String> {
    let mut sessions = state.imap_sessions.lock().map_err(|e| format!("获取会话锁失败: {}", e))?;
    if let Some(mut session) = sessions.remove(&account.id) {
        if session.noop().is_ok() {
            return Ok(session);
        }
    }
    client::connect(account).map_err(|e| format!("连接IMAP服务器失败: {}", e))
}

const IMAP_POOL_MAX: usize = 10;

fn return_imap_session(state: &AppState, account_id: &str, session: client::TlsSession) {
    if let Ok(mut sessions) = state.imap_sessions.lock() {
        if sessions.len() >= IMAP_POOL_MAX && !sessions.contains_key(account_id) {
            if let Some(oldest_key) = sessions.keys().next().cloned() {
                sessions.remove(&oldest_key);
            }
        }
        sessions.insert(account_id.to_string(), session);
    }
}

fn apply_rules_for_new_messages(state: &AppState, account: &MailAccount, folder_name: &str) -> Result<(), String> {
    use crate::mail::model::{RuleField, RuleOperator, RuleAction};

    let (rules, recent_msgs) = {
        let store = ensure_store(state)?;
        let s = store.as_ref().ok_or("数据库未初始化")?;
        let rules = s.get_applicable_rules(&account.id)?;
        let msgs = s.get_recent_messages(&account.id, folder_name, 50)?;
        (rules, msgs)
    };

    if rules.is_empty() { return Ok(()); }

    for msg in &recent_msgs {
        for rule in &rules {
            let text = match rule.field {
                RuleField::Subject => &msg.subject,
                RuleField::From => msg.from.first().map_or("", |a| &a.address),
                RuleField::To => msg.to.first().map_or("", |a| &a.address),
            };
            let matched = match rule.operator {
                RuleOperator::Contains => text.to_lowercase().contains(&rule.value.to_lowercase()),
                RuleOperator::Equals => text.eq_ignore_ascii_case(&rule.value),
                RuleOperator::StartsWith => text.to_lowercase().starts_with(&rule.value.to_lowercase()),
                RuleOperator::EndsWith => text.to_lowercase().ends_with(&rule.value.to_lowercase()),
            };
            if !matched { continue; }

            let store = ensure_store(state)?;
            let s = store.as_ref().ok_or("数据库未初始化")?;

            match rule.action {
                RuleAction::MarkRead => {
                    let _ = s.mark_message_read(&msg.id, true);
                }
                RuleAction::MarkFlagged => {
                    let _ = s.mark_message_flagged(&msg.id, true);
                }
                RuleAction::Delete => {
                    let _ = s.delete_message(&msg.id);
                }
                RuleAction::MoveTo => {
                    if !rule.action_value.is_empty() {
                        let _ = s.move_message_to_folder(&msg.id, &rule.action_value);
                    }
                }
            }
            break;
        }
    }

    let store = ensure_store(state)?;
    let s = store.as_ref().ok_or("数据库未初始化")?;
    let _ = s.update_folder_unread_count(&account.id, folder_name);

    Ok(())
}

#[tauri::command]
pub fn init_store(state: State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let db_dir = app.path().app_data_dir().map_err(|e| format!("获取数据目录失败: {}", e))?;
    std::fs::create_dir_all(&db_dir).map_err(|e| format!("创建数据目录失败: {}", e))?;
    let db_path = db_dir.join("swallowmail.db");
    let store = MailStore::new(db_path.to_str().ok_or("数据库路径无效")?)
        .map_err(|e| format!("初始化数据库失败: {}", e))?;
    let mut guard = state.store.lock().map_err(|e| format!("获取存储锁失败: {}", e))?;
    *guard = Some(store);
    Ok(())
}

#[tauri::command]
pub fn add_qq_account(email: String, password: String, display_name: String) -> Result<MailAccount, String> {
    let account = account::create_qq_account(&email, &password, &display_name);
    Ok(account)
}

#[tauri::command]
pub fn add_163_account(email: String, password: String, display_name: String) -> Result<MailAccount, String> {
    let account = account::create_163_account(&email, &password, &display_name);
    Ok(account)
}

#[tauri::command]
pub fn add_126_account(email: String, password: String, display_name: String) -> Result<MailAccount, String> {
    let account = account::create_126_account(&email, &password, &display_name);
    Ok(account)
}

#[tauri::command]
pub fn add_gmail_account(email: String, password: String, display_name: String) -> Result<MailAccount, String> {
    let account = account::create_gmail_account(&email, &password, &display_name);
    Ok(account)
}

#[tauri::command]
pub fn add_outlook_account(email: String, password: String, display_name: String) -> Result<MailAccount, String> {
    let account = account::create_outlook_account(&email, &password, &display_name);
    Ok(account)
}

#[tauri::command]
pub fn add_custom_account(
    email: String,
    password: String,
    display_name: String,
    imap_host: String,
    imap_port: u16,
    smtp_host: String,
    smtp_port: u16,
    use_ssl: bool,
) -> Result<MailAccount, String> {
    let account = account::create_custom_account(
        &email, &password, &display_name,
        &imap_host, imap_port, &smtp_host, smtp_port, use_ssl,
    );
    Ok(account)
}

#[tauri::command]
pub fn save_account_to_db(state: State<AppState>, account: MailAccount) -> Result<(), String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.save_account(&account)
}

#[tauri::command]
pub fn list_accounts_from_db(state: State<AppState>) -> Result<Vec<MailAccount>, String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.list_accounts()
}

#[tauri::command]
pub fn delete_account_from_db(state: State<AppState>, account_id: String) -> Result<(), String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.delete_account(&account_id)
}

#[tauri::command]
pub fn update_account_in_db(state: State<AppState>, account: MailAccount) -> Result<(), String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.update_account(&account)
}

#[tauri::command]
pub fn connect_and_fetch_folders(state: State<AppState>, account: MailAccount) -> Result<Vec<MailFolder>, String> {
    let mut session = get_imap_session(&state, &account)?;

    let raw_folders = client::fetch_folder_list(&mut session)
        .map_err(|e| format!("获取文件夹列表失败: {}", e))?;

    let imap_session = imap::ImapSession::new(&account.id, &account.imap_host);
    let folders = imap_session.list_folders(&raw_folders);

    return_imap_session(&state, &account.id, session);

    Ok(folders)
}

#[tauri::command]
pub fn save_folders_to_db(state: State<AppState>, account_id: String, folders: Vec<MailFolder>) -> Result<(), String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.save_folders(&account_id, &folders)
}

#[tauri::command]
pub fn list_folders_from_db(state: State<AppState>, account_id: String) -> Result<Vec<MailFolder>, String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.list_folders(&account_id)
}

#[tauri::command]
pub fn sync_folder_messages(
    state: State<AppState>,
    account: MailAccount,
    folder_name: String,
) -> Result<SyncResult, String> {
    let since_uid = {
        let store = ensure_store(&state)?;
        store.as_ref().ok_or("数据库未初始化")?.get_last_uid(&account.id, &folder_name)?
    };

    let mut session = get_imap_session(&state, &account)?;

    let messages = sync::sync_folder(&mut session, &folder_name, since_uid)?;

    return_imap_session(&state, &account.id, session);

    let synced_count = {
        let store = ensure_store(&state)?;
        let s = store.as_ref().ok_or("数据库未初始化")?;
        let count = s.save_messages(&account.id, &folder_name, &messages)?;
        let _ = s.update_folder_unread_count(&account.id, &folder_name);
        count
    };

    if synced_count > 0 {
        let _ = apply_rules_for_new_messages(&state, &account, &folder_name);
    }

    Ok(SyncResult {
        folder: folder_name,
        total_fetched: messages.len() as u32,
        new_saved: synced_count as u32,
    })
}

#[tauri::command]
pub fn list_messages_from_db(
    state: State<AppState>,
    account_id: String,
    folder_name: String,
    limit: u32,
    offset: u32,
) -> Result<Vec<MailMessage>, String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.list_messages(&account_id, &folder_name, limit, offset)
}

#[tauri::command]
pub fn fetch_message_body(
    state: State<AppState>,
    account: MailAccount,
    message_id: String,
    folder_name: String,
    uid: u32,
) -> Result<MailMessage, String> {
    {
        let store = ensure_store(&state)?;
        let store = store.as_ref().ok_or("数据库未初始化")?;
        if let Some(msg) = store.get_message(&message_id)? {
            if msg.body_text.is_some() || msg.body_html.is_some() {
                return Ok(msg);
            }
        }
    }

    let mut session = get_imap_session(&state, &account)?;

    let body = sync::fetch_message_body(&mut session, &folder_name, uid)?;

    return_imap_session(&state, &account.id, session);

    {
        let store = ensure_store(&state)?;
        let store = store.as_ref().ok_or("数据库未初始化")?;
        store.update_message_body(
            &message_id,
            body.body_text.as_deref(),
            body.body_html.as_deref(),
            &body.preview,
            body.has_attachments,
            &body.attachments,
        )?;
        let msg = store.get_message(&message_id)?;
        msg.ok_or_else(|| "更新后未找到邮件".to_string())
    }
}

#[tauri::command]
pub fn send_mail(
    account: MailAccount,
    from: MailAddress,
    to: Vec<MailAddress>,
    cc: Vec<MailAddress>,
    subject: String,
    body_text: String,
    body_html: Option<String>,
    attachments: Vec<ComposeAttachment>,
) -> Result<(), String> {
    let file_attachments: Vec<smtp::FileAttachment> = attachments.into_iter().map(|a| smtp::FileAttachment {
        filename: a.filename,
        content_type: a.content_type,
        data: a.data,
    }).collect();
    let compose = smtp::ComposeMail {
        from,
        to,
        cc,
        subject,
        body_text,
        body_html,
        attachments: file_attachments,
    };
    smtp::send_mail(&account, &compose)
        .map_err(|e| format!("发送邮件失败: {}", e))
}

#[derive(Debug, serde::Deserialize)]
pub struct ComposeAttachment {
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

#[tauri::command]
pub fn toggle_message_read(state: State<AppState>, account: MailAccount, message_id: String, is_read: bool) -> Result<(), String> {
    let folder_name = {
        let store = ensure_store(&state)?;
        let s = store.as_ref().ok_or("数据库未初始化")?;
        s.toggle_message_read(&message_id, is_read)?;
        s.get_message_uid_and_folder(&message_id).ok().flatten().map(|(_, f)| f)
    };

    if let Some((uid, folder)) = {
        let store = ensure_store(&state)?;
        store.as_ref().ok_or("数据库未初始化")?.get_message_uid_and_folder(&message_id).ok().flatten()
    } {
        if let Ok(mut session) = get_imap_session(&state, &account) {
            if sync::set_flags_seen(&mut session, &folder, uid, is_read).is_ok() {
                return_imap_session(&state, &account.id, session);
            }
        }
    }

    if let Some(folder) = folder_name {
        let store = ensure_store(&state)?;
        let _ = store.as_ref().ok_or("数据库未初始化")?.update_folder_unread_count(&account.id, &folder);
    }

    Ok(())
}

#[tauri::command]
pub fn toggle_message_flagged(state: State<AppState>, account: MailAccount, message_id: String, is_flagged: bool) -> Result<(), String> {
    let store = ensure_store(&state)?;
    let s = store.as_ref().ok_or("数据库未初始化")?;
    s.toggle_message_flagged(&message_id, is_flagged)?;

    if let Some((uid, folder)) = s.get_message_uid_and_folder(&message_id).ok().flatten() {
        if let Ok(mut session) = get_imap_session(&state, &account) {
            if sync::set_flags_flagged(&mut session, &folder, uid, is_flagged).is_ok() {
                return_imap_session(&state, &account.id, session);
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn delete_message(state: State<AppState>, account: MailAccount, message_id: String) -> Result<(), String> {
    let folder_name = {
        let store = ensure_store(&state)?;
        let s = store.as_ref().ok_or("数据库未初始化")?;
        let folder = s.get_message_uid_and_folder(&message_id).ok().flatten().map(|(_, f)| f);

        if let Some((uid, ref f)) = s.get_message_uid_and_folder(&message_id).ok().flatten() {
            if let Ok(mut session) = get_imap_session(&state, &account) {
                if sync::delete_message_from_server(&mut session, f, uid).is_ok() {
                    return_imap_session(&state, &account.id, session);
                }
            }
        }

        s.delete_message(&message_id)?;
        folder
    };

    if let Some(folder) = folder_name {
        let store = ensure_store(&state)?;
        let _ = store.as_ref().ok_or("数据库未初始化")?.update_folder_unread_count(&account.id, &folder);
    }

    Ok(())
}

#[tauri::command]
pub fn search_messages(
    state: State<AppState>,
    account_id: String,
    query: String,
) -> Result<Vec<MailMessage>, String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.search_messages(&account_id, &query, 100)
}

#[tauri::command]
pub fn get_thread_messages(
    state: State<AppState>,
    account_id: String,
    subject: String,
) -> Result<Vec<MailMessage>, String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.get_thread_messages(&account_id, &subject)
}

#[tauri::command]
pub fn localize_folder_name(raw_name: String, server_host: String, attributes: Vec<String>) -> Result<MailFolder, String> {
    let session = imap::ImapSession::new("", &server_host);
    let folders = session.list_folders(&[(raw_name, "/".to_string(), attributes)]);
    folders.into_iter().next().ok_or_else(|| "无法本地化文件夹名".to_string())
}

#[tauri::command]
pub fn get_provider_presets() -> Vec<ProviderPreset> {
    vec![
        ProviderPreset {
            name: "QQ邮箱".to_string(),
            account_type: "qq".to_string(),
            imap_host: "imap.qq.com".to_string(),
            imap_port: 993,
            smtp_host: "smtp.qq.com".to_string(),
            smtp_port: 465,
        },
        ProviderPreset {
            name: "网易163".to_string(),
            account_type: "163".to_string(),
            imap_host: "imap.163.com".to_string(),
            imap_port: 993,
            smtp_host: "smtp.163.com".to_string(),
            smtp_port: 465,
        },
        ProviderPreset {
            name: "网易126".to_string(),
            account_type: "126".to_string(),
            imap_host: "imap.126.com".to_string(),
            imap_port: 993,
            smtp_host: "smtp.126.com".to_string(),
            smtp_port: 465,
        },
        ProviderPreset {
            name: "Gmail".to_string(),
            account_type: "gmail".to_string(),
            imap_host: "imap.gmail.com".to_string(),
            imap_port: 993,
            smtp_host: "smtp.gmail.com".to_string(),
            smtp_port: 465,
        },
        ProviderPreset {
            name: "Outlook".to_string(),
            account_type: "outlook".to_string(),
            imap_host: "outlook.office365.com".to_string(),
            imap_port: 993,
            smtp_host: "smtp.office365.com".to_string(),
            smtp_port: 587,
        },
    ]
}

#[derive(Debug, serde::Serialize)]
pub struct ProviderPreset {
    pub name: String,
    pub account_type: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
}

#[derive(Debug, serde::Serialize)]
pub struct SyncResult {
    pub folder: String,
    pub total_fetched: u32,
    pub new_saved: u32,
}

#[tauri::command]
pub async fn save_attachment(
    app: tauri::AppHandle,
    state: State<AppState>,
    account: MailAccount,
    folder_name: String,
    uid: u32,
    filename: String,
) -> Result<(), String> {
    let mut session = get_imap_session(&state, &account)?;

    let data = sync::fetch_attachment_data(&mut session, &folder_name, uid, &filename)?;

    return_imap_session(&state, &account.id, session);

    let file_path = tauri_plugin_dialog::FileDialogBuilder::new(&app)
        .set_file_name(&filename)
        .save_file()
        .await
        .map_err(|e| format!("选择保存路径失败: {}", e))?
        .ok_or_else(|| "用户取消了保存".to_string())?;

    std::fs::write(&file_path, &data)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn save_draft(
    state: State<AppState>,
    account_id: String,
    subject: String,
    sender: String,
    recipients: String,
    body_text: Option<String>,
    body_html: Option<String>,
) -> Result<String, String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.save_draft(
        &account_id,
        &subject,
        &sender,
        &recipients,
        body_text.as_deref(),
        body_html.as_deref(),
    )
}

#[tauri::command]
pub fn disconnect_imap(state: State<AppState>, account_id: String) -> Result<(), String> {
    let mut sessions = state.imap_sessions.lock().map_err(|e| format!("获取会话锁失败: {}", e))?;
    sessions.remove(&account_id);
    Ok(())
}

#[tauri::command]
pub fn refresh_unread_counts(state: State<AppState>, account_id: String) -> Result<Vec<(String, u32)>, String> {
    let store = ensure_store(&state)?;
    let s = store.as_ref().ok_or("数据库未初始化")?;
    let folders = s.list_folders(&account_id)?;
    let mut result = Vec::new();
    for folder in &folders {
        let count = s.update_folder_unread_count(&account_id, &folder.name)?;
        result.push((folder.name.clone(), count));
    }
    Ok(result)
}

#[tauri::command]
pub fn move_message(state: State<AppState>, account: MailAccount, message_id: String, dest_folder: String) -> Result<(), String> {
    let (uid, source_folder) = {
        let store = ensure_store(&state)?;
        let s = store.as_ref().ok_or("数据库未初始化")?;
        let (uid, folder) = s.get_message_uid_and_folder(&message_id)?
            .ok_or("邮件不存在或无 UID")?;
        s.move_message_to_folder(&message_id, &dest_folder)?;
        (uid, folder)
    };

    if let Ok(mut session) = get_imap_session(&state, &account) {
        if sync::move_message(&mut session, &source_folder, uid, &dest_folder).is_ok() {
            return_imap_session(&state, &account.id, session);
        }
    }

    {
        let store = ensure_store(&state)?;
        let s = store.as_ref().ok_or("数据库未初始化")?;
        let _ = s.update_folder_unread_count(&account.id, &source_folder);
        let _ = s.update_folder_unread_count(&account.id, &dest_folder);
    }

    Ok(())
}

#[tauri::command]
pub fn list_all_inbox_messages(state: State<AppState>, account_id: String, limit: u32) -> Result<Vec<MailMessage>, String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.list_all_inbox_messages(&account_id, limit)
}

#[tauri::command]
pub fn save_rule(state: State<AppState>, rule: MailRule) -> Result<(), String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.save_rule(&rule)
}

#[tauri::command]
pub fn list_rules(state: State<AppState>, account_id: String) -> Result<Vec<MailRule>, String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.list_rules(&account_id)
}

#[tauri::command]
pub fn delete_rule(state: State<AppState>, rule_id: String) -> Result<(), String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.delete_rule(&rule_id)
}

#[tauri::command]
pub fn save_folder_mapping(state: State<AppState>, account_id: String, mapping: FolderMapping) -> Result<(), String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.save_folder_mapping(&account_id, &mapping)
}

#[tauri::command]
pub fn get_folder_mapping(state: State<AppState>, account_id: String) -> Result<FolderMapping, String> {
    let store = ensure_store(&state)?;
    store.as_ref().ok_or("数据库未初始化")?.get_folder_mapping(&account_id)
}