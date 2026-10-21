use tauri::State;
use std::sync::Mutex;
use crate::mail::model::{MailAccount, MailFolder, MailMessage};
use crate::mail::account;
use crate::mail::imap;
use crate::mail::smtp;
use crate::mail::store::MailStore;

pub struct AppState {
    pub store: Mutex<Option<MailStore>>,
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
pub fn localize_folder_name(raw_name: String, server_host: String, attributes: Vec<String>) -> Result<MailFolder, String> {
    let session = imap::ImapSession::new("", &server_host);
    let folder = session.list_folders(&[(raw_name, "/".to_string(), attributes)]);
    Ok(folder.into_iter().next().unwrap())
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
