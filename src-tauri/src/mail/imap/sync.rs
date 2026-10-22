use imap::Flag;
use crate::mail::imap::client::TlsSession;
use crate::mail::model::{MailMessage, MailAddress, Attachment};

const PREVIEW_MAX_LEN: usize = 200;

pub fn set_flags_seen(session: &mut TlsSession, folder: &str, uid: u32, seen: bool) -> Result<(), String> {
    session
        .select(folder)
        .map_err(|e| format!("选择文件夹失败: {}", e))?;

    if seen {
        session
            .uid_store(format!("{}", uid), "+FLAGS (\\Seen)", imap::UidEnable::Yes)
            .map_err(|e| format!("设置已读标记失败: {}", e))?;
    } else {
        session
            .uid_store(format!("{}", uid), "-FLAGS (\\Seen)", imap::UidEnable::Yes)
            .map_err(|e| format!("取消已读标记失败: {}", e))?;
    }
    Ok(())
}

pub fn set_flags_flagged(session: &mut TlsSession, folder: &str, uid: u32, flagged: bool) -> Result<(), String> {
    session
        .select(folder)
        .map_err(|e| format!("选择文件夹失败: {}", e))?;

    if flagged {
        session
            .uid_store(format!("{}", uid), "+FLAGS (\\Flagged)", imap::UidEnable::Yes)
            .map_err(|e| format!("设置星标失败: {}", e))?;
    } else {
        session
            .uid_store(format!("{}", uid), "-FLAGS (\\Flagged)", imap::UidEnable::Yes)
            .map_err(|e| format!("取消星标失败: {}", e))?;
    }
    Ok(())
}

pub fn delete_message_from_server(session: &mut TlsSession, folder: &str, uid: u32) -> Result<(), String> {
    session
        .select(folder)
        .map_err(|e| format!("选择文件夹失败: {}", e))?;

    session
        .uid_store(format!("{}", uid), "+FLAGS (\\Deleted)", imap::UidEnable::Yes)
        .map_err(|e| format!("标记删除失败: {}", e))?;

    session
        .expunge()
        .map_err(|e| format!("永久删除失败: {}", e))?;

    Ok(())
}

pub fn move_message(session: &mut TlsSession, source_folder: &str, uid: u32, dest_folder: &str) -> Result<(), String> {
    session
        .select(source_folder)
        .map_err(|e| format!("选择源文件夹失败: {}", e))?;

    let uid_str = format!("{}", uid);
    if session.uid_copy(&uid_str, dest_folder).is_ok() {
        session
            .uid_store(&uid_str, "+FLAGS (\\Deleted)", imap::UidEnable::Yes)
            .map_err(|e| format!("标记源邮件删除失败: {}", e))?;
        session
            .expunge()
            .map_err(|e| format!("永久删除失败: {}", e))?;
        return Ok(());
    }

    Err("移动邮件失败: COPY 命令不被支持".to_string())
}

pub fn fetch_attachment_data(
    session: &mut TlsSession,
    folder: &str,
    uid: u32,
    filename: &str,
) -> Result<Vec<u8>, String> {
    session
        .select(folder)
        .map_err(|e| format!("选择文件夹失败: {}", e))?;

    let fetched = session
        .uid_fetch(format!("{}", uid), "BODY.PEEK[]")
        .map_err(|e| format!("获取邮件原始数据失败: {}", e))?;

    let msg = fetched
        .iter()
        .next()
        .ok_or_else(|| "未找到指定UID的邮件".to_string())?;

    let raw_body = msg
        .body()
        .ok_or_else(|| "邮件无正文数据".to_string())?;

    let parsed = mail_parser::MessageParser::default()
        .parse(raw_body)
        .ok_or_else(|| "解析邮件失败".to_string())?;

    for attachment in parsed.attachments() {
        let att_name = attachment.attachment_name().unwrap_or("unnamed").to_string();
        if att_name == filename {
            return Ok(attachment.contents().to_vec());
        }
    }

    Err(format!("未找到附件: {}", filename))
}

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
            .uid_fetch(format!("{}", uid), "(ENVELOPE FLAGS)")
            .map_err(|e| format!("获取邮件失败: {}", e));

        if let Ok(fetched) = fetch {
            for msg in fetched.iter() {
                let envelope = msg.envelope();

                let subject = envelope
                    .and_then(|e| e.subject.as_ref())
                    .map(|s| String::from_utf8_lossy(s).into_owned())
                    .unwrap_or_default();

                let from = envelope
                    .and_then(|e| e.from.as_ref())
                    .map(|addrs| addrs.iter().map(parse_address).collect())
                    .unwrap_or_default();

                let to = envelope
                    .and_then(|e| e.to.as_ref())
                    .map(|addrs| addrs.iter().map(parse_address).collect())
                    .unwrap_or_default();

                let cc = envelope
                    .and_then(|e| e.cc.as_ref())
                    .map(|addrs| addrs.iter().map(parse_address).collect())
                    .unwrap_or_default();

                let date = envelope
                    .and_then(|e| e.date.as_ref())
                    .and_then(|d| parse_date(d));

                let is_read = !msg.flags().iter().any(|f| matches!(f, Flag::Seen));
                let is_flagged = msg.flags().iter().any(|f| matches!(f, Flag::Flagged));

                let message = MailMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    folder: folder.to_string(),
                    uid: *uid,
                    subject,
                    from,
                    to,
                    cc,
                    date,
                    flags: Vec::new(),
                    is_read,
                    is_flagged,
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

fn parse_address(addr: &imap::types::Address) -> MailAddress {
    let name = addr.name.as_ref()
        .map(|n| String::from_utf8_lossy(n).into_owned());
    let address_parts: Vec<String> = vec![
        addr.mailbox.as_ref().map(|m| String::from_utf8_lossy(m).into_owned()),
        addr.host.as_ref().map(|h| String::from_utf8_lossy(h).into_owned()),
    ];
    let address = match (&address_parts[0], &address_parts[1]) {
        (Some(user), Some(domain)) => format!("{}@{}", user, domain),
        (Some(user), None) => user.clone(),
        _ => String::new(),
    };
    MailAddress { name, address }
}

fn parse_date(date_bytes: &[u8]) -> Option<chrono::DateTime<chrono::Utc>> {
    let date_str = String::from_utf8_lossy(date_bytes);
    let formats = [
        "%d %b %Y %H:%M:%S %z",
        "%d %b %Y %H:%M:%S",
        "%a, %d %b %Y %H:%M:%S %z",
        "%a, %d %b %Y %H:%M:%S",
        "%a, %d %b %Y %H:%M %z",
        "%d-%b-%Y %H:%M:%S %z",
    ];
    for fmt in &formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&date_str, fmt) {
            return Some(dt.and_utc());
        }
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(&date_str) {
        return Some(dt.with_timezone(&chrono::Utc));
    }
    None
}

pub struct MessageBody {
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub preview: String,
    pub has_attachments: bool,
    pub attachments: Vec<Attachment>,
}

pub fn fetch_message_body(
    session: &mut TlsSession,
    folder: &str,
    uid: u32,
) -> Result<MessageBody, String> {
    session
        .select(folder)
        .map_err(|e| format!("选择文件夹失败: {}", e))?;

    let fetched = session
        .uid_fetch(format!("{}", uid), "BODY.PEEK[]")
        .map_err(|e| format!("获取邮件正文失败: {}", e))?;

    let msg = fetched
        .iter()
        .next()
        .ok_or_else(|| "未找到指定UID的邮件".to_string())?;

    let raw_body = msg
        .body()
        .ok_or_else(|| "邮件无正文数据".to_string())?;

    let parsed = mail_parser::MessageParser::default()
        .parse(raw_body)
        .ok_or_else(|| "解析邮件正文失败".to_string())?;

    let mut body_text = None;
    let mut body_html = None;
    let mut preview = String::new();
    let mut has_attachments = false;
    let mut attachments = Vec::new();

    if let Some(text) = parsed.text_bodies().next() {
        let content = text.contents.clone().into_owned();
        body_text = Some(content.clone());
        preview = truncate_preview(&content);
    }

    if let Some(html) = parsed.html_bodies().next() {
        body_html = Some(sanitize_html(&html.contents));
    }

    for attachment in parsed.attachments() {
        has_attachments = true;
        let filename = attachment.attachment_name().unwrap_or("unnamed").to_string();
        let content_type = attachment.content_type().map(|ct| ct.ctype()).unwrap_or("application/octet-stream").to_string();
        let size = attachment.contents().len() as u64;
        attachments.push(Attachment {
            filename,
            content_type,
            size,
        });
    }

    if preview.is_empty() {
        if let Some(html) = &body_html {
            preview = truncate_preview(&strip_html_tags(html));
        }
    }

    Ok(MessageBody {
        body_text,
        body_html,
        preview,
        has_attachments,
        attachments,
    })
}

fn truncate_preview(text: &str) -> String {
    let cleaned: String = text.chars().take(PREVIEW_MAX_LEN).collect();
    if text.len() > PREVIEW_MAX_LEN {
        format!("{}…", cleaned)
    } else {
        cleaned
    }
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

fn sanitize_html(html: &str) -> String {
    let re_script = regex::Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
    let re_on_attr = regex::Regex::new(r#"\s+on\w+\s*=\s*"[^"]*""#).unwrap();
    let re_on_attr2 = regex::Regex::new(r#"\s+on\w+\s*=\s*'[^']*'"#).unwrap();
    let step1 = re_script.replace_all(html, "");
    let step2 = re_on_attr.replace_all(&step1, "");
    let step3 = re_on_attr2.replace_all(&step2, "");
    step3.into_owned()
}