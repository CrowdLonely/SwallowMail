use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailAccount {
    pub id: String,
    pub display_name: String,
    pub email_address: String,
    pub account_type: AccountType,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub use_ssl: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    Imap,
    Gmail,
    Outlook,
    QQ,
    NetEase163,
    NetEase126,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailFolder {
    pub name: String,
    pub display_name: String,
    pub delimiter: String,
    pub attributes: Vec<String>,
    pub unread_count: u32,
    pub total_count: u32,
    pub special_use: Option<SpecialUse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecialUse {
    Inbox,
    Sent,
    Drafts,
    Trash,
    Junk,
    Archive,
    Flagged,
    Important,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailMessage {
    pub id: String,
    pub folder: String,
    pub uid: u32,
    pub subject: String,
    pub from: Vec<MailAddress>,
    pub to: Vec<MailAddress>,
    pub cc: Vec<MailAddress>,
    pub date: Option<DateTime<Utc>>,
    pub flags: Vec<String>,
    pub is_read: bool,
    pub is_flagged: bool,
    pub has_attachments: bool,
    pub preview: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailAddress {
    pub name: Option<String>,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub size: u64,
}
