pub mod client;
pub mod sync;

use crate::mail::model::{MailFolder, MailMessage};
use crate::mail::folder;

#[derive(Debug, thiserror::Error)]
pub enum ImapError {
    #[error("连接失败: {0}")]
    ConnectionFailed(String),
    #[error("认证失败: {0}")]
    AuthFailed(String),
    #[error("协议错误: {0}")]
    ProtocolError(String),
    #[error("TLS错误: {0}")]
    TlsError(#[from] native_tls::Error),
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
}

pub type ImapResult<T> = Result<T, ImapError>;

pub struct ImapSession {
    account_id: String,
    server_host: String,
}

impl ImapSession {
    pub fn new(account_id: &str, server_host: &str) -> Self {
        ImapSession {
            account_id: account_id.to_string(),
            server_host: server_host.to_string(),
        }
    }

    pub fn list_folders(&self, raw_folders: &[(String, String, Vec<String>)]) -> Vec<MailFolder> {
        raw_folders
            .iter()
            .map(|(name, delimiter, attrs)| {
                folder::build_display_folder(name, delimiter, attrs, &self.server_host)
            })
            .collect()
    }
}
