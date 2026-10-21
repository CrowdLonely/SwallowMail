use crate::mail::imap::client::{TlsSession, fetch_folder_list};
use crate::mail::model::MailMessage;

pub struct SyncState {
    pub folder_name: String,
    pub last_uid: u32,
    pub total_synced: u32,
}

pub fn sync_folder(session: &mut TlsSession, folder: &str, since_uid: u32) -> Result<Vec<MailMessage>, String> {
    session
        .select(folder)
        .map_err(|e| format!("选择文件夹失败: {}", e))?;

    let search_criteria = if since_uid > 0 {
        format!("UID {}:*", since_uid + 1)
    } else {
        "ALL".to_string()
    };

    let uids = session
        .uid_search(search_criteria)
        .map_err(|e| format!("搜索邮件失败: {}", e))?;

    let mut messages = Vec::new();
    for uid in uids.iter().take(50) {
        let fetch = session
            .uid_fetch(format!("{}", uid), "(ENVELOPE BODY.PEEK[HEADER.FIELDS (SUBJECT FROM TO CC DATE)] FLAGS)")
            .map_err(|e| format!("获取邮件失败: {}", e));

        if let Ok(fetched) = fetch {
            for msg in fetched.iter() {
                let message = MailMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    folder: folder.to_string(),
                    uid: *uid,
                    subject: msg.envelope().and_then(|e| e.subject)
                        .and_then(|s| String::from_utf8(s).ok())
                        .unwrap_or_default(),
                    from: Vec::new(),
                    to: Vec::new(),
                    cc: Vec::new(),
                    date: None,
                    flags: Vec::new(),
                    is_read: !msg.flags().iter().any(|f| format!("{:?}", f).contains("Seen")),
                    is_flagged: msg.flags().iter().any(|f| format!("{:?}", f).contains("Flagged")),
                    has_attachments: false,
                    preview: String::new(),
                    body_text: None,
                    body_html: None,
                    attachments: Vec::new(),
                };
                messages.push(message);
            }
        }
    }

    Ok(messages)
}
