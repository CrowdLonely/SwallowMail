use rusqlite::{Connection, params};
use crate::mail::model::{MailAccount, MailFolder, MailMessage, MailAddress, AccountType, SpecialUse, Attachment, MailRule, RuleField, RuleOperator, RuleAction, FolderMapping};
use chrono::Utc;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use sha2::{Sha256, Digest};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

const APP_KEY_SEED: &[u8] = b"SwallowMail-AES256-Key-Seed-2024";

fn derive_key() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(APP_KEY_SEED);
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

fn encrypt_password(plain: &str) -> String {
    let key = aes_gcm::Key::<Aes256Gcm>::from(derive_key());
    let cipher = Aes256Gcm::new(&key);
    let nonce_bytes: [u8; 12] = {
        let mut hasher = Sha256::new();
        hasher.update(plain.as_bytes());
        hasher.update(APP_KEY_SEED);
        let h = hasher.finalize();
        let mut n = [0u8; 12];
        n.copy_from_slice(&h[..12]);
        n
    };
    let nonce = Nonce::from(nonce_bytes);
    match cipher.encrypt(&nonce, plain.as_bytes()) {
        Ok(ciphertext) => {
            let mut combined = nonce_bytes.to_vec();
            combined.extend_from_slice(&ciphertext);
            format!("aes:{}", BASE64.encode(&combined))
        }
        Err(_) => plain.to_string(),
    }
}

fn decrypt_password(encrypted: &str) -> String {
    if !encrypted.starts_with("aes:") {
        if encrypted.starts_with("enc:") {
            return deencrypt_password_legacy(encrypted);
        }
        return encrypted.to_string();
    }
    let b64 = &encrypted[4..];
    let combined = match BASE64.decode(b64) {
        Ok(v) => v,
        Err(_) => return encrypted.to_string(),
    };
    if combined.len() < 12 { return encrypted.to_string(); }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let key = aes_gcm::Key::<Aes256Gcm>::from(derive_key());
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);
    match cipher.decrypt(&nonce, ciphertext) {
        Ok(plaintext) => String::from_utf8_lossy(&plaintext).to_string(),
        Err(_) => encrypted.to_string(),
    }
}

fn deencrypt_password_legacy(obfuscated: &str) -> String {
    if !obfuscated.starts_with("enc:") {
        return obfuscated.to_string();
    }
    let hex = &obfuscated[4..];
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&hex[i..i+2], 16).ok())
        .collect();
    let decoded: Vec<u8> = bytes.iter()
        .enumerate()
        .map(|(i, &b)| b ^ APP_KEY_SEED[i % APP_KEY_SEED.len()])
        .collect();
    String::from_utf8_lossy(&decoded).to_string()
}

pub struct MailStore {
    conn: Connection,
}

impl MailStore {
    pub fn new(db_path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        let store = MailStore { conn };
        store.init_tables()?;
        Ok(store)
    }

    fn init_tables(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                email_address TEXT NOT NULL,
                account_type TEXT NOT NULL,
                imap_host TEXT NOT NULL,
                imap_port INTEGER NOT NULL,
                smtp_host TEXT NOT NULL,
                smtp_port INTEGER NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                use_ssl INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS folders (
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                delimiter TEXT NOT NULL,
                special_use TEXT,
                unread_count INTEGER NOT NULL DEFAULT 0,
                total_count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (account_id, name),
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            );
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                folder_name TEXT NOT NULL,
                uid INTEGER NOT NULL,
                subject TEXT NOT NULL,
                sender TEXT NOT NULL,
                recipients TEXT NOT NULL,
                date TEXT,
                is_read INTEGER NOT NULL DEFAULT 0,
                is_flagged INTEGER NOT NULL DEFAULT 0,
                has_attachments INTEGER NOT NULL DEFAULT 0,
                preview TEXT NOT NULL DEFAULT '',
                body_text TEXT,
                body_html TEXT,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            );
            CREATE INDEX IF NOT EXISTS idx_messages_folder ON messages(account_id, folder_name);
            CREATE INDEX IF NOT EXISTS idx_messages_uid ON messages(account_id, folder_name, uid);
            "
        )?;

        self.conn.execute_batch(
            "ALTER TABLE messages ADD COLUMN attachments TEXT NOT NULL DEFAULT '[]';"
        ).ok();

        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS rules (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                field TEXT NOT NULL,
                operator TEXT NOT NULL,
                value TEXT NOT NULL,
                action TEXT NOT NULL,
                action_value TEXT NOT NULL DEFAULT '',
                enabled INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            );
            ALTER TABLE accounts ADD COLUMN folder_mapping TEXT NOT NULL DEFAULT '{}';"
        ).ok();

        Ok(())
    }

    pub fn save_account(&self, account: &MailAccount) -> Result<(), String> {
        self.conn.execute(
            "INSERT OR REPLACE INTO accounts (id, display_name, email_address, account_type, imap_host, imap_port, smtp_host, smtp_port, username, password, use_ssl, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                account.id,
                account.display_name,
                account.email_address,
                format!("{:?}", account.account_type),
                account.imap_host,
                account.imap_port,
                account.smtp_host,
                account.smtp_port,
                account.username,
                encrypt_password(&account.password),
                account.use_ssl as i32,
                account.created_at.to_rfc3339(),
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_accounts(&self) -> Result<Vec<MailAccount>, String> {
        let mut stmt = self.conn
            .prepare("SELECT id, display_name, email_address, account_type, imap_host, imap_port, smtp_host, smtp_port, username, password, use_ssl, created_at FROM accounts")
            .map_err(|e| e.to_string())?;
        let accounts: Vec<MailAccount> = stmt.query_map([], |row| {
            let account_type_str: String = row.get(3)?;
            let account_type = match account_type_str.as_str() {
                "QQ" => AccountType::QQ,
                "NetEase163" => AccountType::NetEase163,
                "NetEase126" => AccountType::NetEase126,
                "Gmail" => AccountType::Gmail,
                "Outlook" => AccountType::Outlook,
                _ => AccountType::Imap,
            };
            let use_ssl: i32 = row.get(10)?;
            let created_at_str: String = row.get(11)?;
            Ok(MailAccount {
                id: row.get(0)?,
                display_name: row.get(1)?,
                email_address: row.get(2)?,
                account_type,
                imap_host: row.get(4)?,
                imap_port: row.get(5)?,
                smtp_host: row.get(6)?,
                smtp_port: row.get(7)?,
                username: row.get(8)?,
                password: {
                    let raw: String = row.get(9)?;
                    deencrypt_password(&raw)
                },
                use_ssl: use_ssl != 0,
                created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(accounts)
    }

    pub fn save_folders(&self, account_id: &str, folders: &[MailFolder]) -> Result<(), String> {
        for folder in folders {
            self.conn.execute(
                "INSERT OR REPLACE INTO folders (account_id, name, display_name, delimiter, special_use, unread_count, total_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    account_id,
                    folder.name,
                    folder.display_name,
                    folder.delimiter,
                    folder.special_use.as_ref().map(|su| format!("{:?}", su)),
                    folder.unread_count,
                    folder.total_count,
                ],
            ).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn list_folders(&self, account_id: &str) -> Result<Vec<MailFolder>, String> {
        let mut stmt = self.conn
            .prepare("SELECT name, display_name, delimiter, special_use, unread_count, total_count FROM folders WHERE account_id = ?1")
            .map_err(|e| e.to_string())?;
        let folders: Vec<MailFolder> = stmt.query_map(params![account_id], |row| {
            let special_use_str: Option<String> = row.get(3)?;
            let special_use = special_use_str.and_then(|s| match s.as_str() {
                "Inbox" => Some(SpecialUse::Inbox),
                "Sent" => Some(SpecialUse::Sent),
                "Drafts" => Some(SpecialUse::Drafts),
                "Trash" => Some(SpecialUse::Trash),
                "Junk" => Some(SpecialUse::Junk),
                "Archive" => Some(SpecialUse::Archive),
                "Flagged" => Some(SpecialUse::Flagged),
                "Important" => Some(SpecialUse::Important),
                _ => None,
            });
            Ok(MailFolder {
                name: row.get(0)?,
                display_name: row.get(1)?,
                delimiter: row.get(2)?,
                attributes: Vec::new(),
                unread_count: row.get::<_, i32>(4)? as u32,
                total_count: row.get::<_, i32>(5)? as u32,
                special_use,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(folders)
    }

    pub fn get_account(&self, account_id: &str) -> Result<Option<MailAccount>, String> {
        let mut stmt = self.conn
            .prepare("SELECT id, display_name, email_address, account_type, imap_host, imap_port, smtp_host, smtp_port, username, password, use_ssl, created_at FROM accounts WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut accounts: Vec<MailAccount> = stmt.query_map(params![account_id], |row| {
            let account_type_str: String = row.get(3)?;
            let account_type = match account_type_str.as_str() {
                "QQ" => AccountType::QQ,
                "NetEase163" => AccountType::NetEase163,
                "NetEase126" => AccountType::NetEase126,
                "Gmail" => AccountType::Gmail,
                "Outlook" => AccountType::Outlook,
                _ => AccountType::Imap,
            };
            let use_ssl: i32 = row.get(10)?;
            let created_at_str: String = row.get(11)?;
            Ok(MailAccount {
                id: row.get(0)?,
                display_name: row.get(1)?,
                email_address: row.get(2)?,
                account_type,
                imap_host: row.get(4)?,
                imap_port: row.get(5)?,
                smtp_host: row.get(6)?,
                smtp_port: row.get(7)?,
                username: row.get(8)?,
                password: {
                    let raw: String = row.get(9)?;
                    deencrypt_password(&raw)
                },
                use_ssl: use_ssl != 0,
                created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(accounts.pop())
    }

    pub fn save_messages(&self, account_id: &str, folder_name: &str, messages: &[MailMessage]) -> Result<usize, String> {
        let mut saved = 0;
        for msg in messages {
            let sender = serde_json::to_string(&msg.from).unwrap_or_default();
            let recipients = serde_json::to_string(&msg.to).unwrap_or_default();
            let date_str = msg.date.map(|d| d.to_rfc3339());
            let changed = self.conn.execute(
                "INSERT OR IGNORE INTO messages (id, account_id, folder_name, uid, subject, sender, recipients, date, is_read, is_flagged, has_attachments, preview, body_text, body_html)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    msg.id,
                    account_id,
                    folder_name,
                    msg.uid,
                    msg.subject,
                    sender,
                    recipients,
                    date_str,
                    msg.is_read as i32,
                    msg.is_flagged as i32,
                    msg.has_attachments as i32,
                    msg.preview,
                    msg.body_text,
                    msg.body_html,
                ],
            ).map_err(|e| e.to_string())?;
            if changed > 0 {
                saved += 1;
            }
        }
        Ok(saved)
    }

    pub fn list_messages(&self, account_id: &str, folder_name: &str, limit: u32, offset: u32) -> Result<Vec<MailMessage>, String> {
        let mut stmt = self.conn
            .prepare("SELECT id, uid, subject, sender, recipients, date, is_read, is_flagged, has_attachments, preview, body_text, body_html, attachments FROM messages WHERE account_id = ?1 AND folder_name = ?2 ORDER BY date DESC LIMIT ?3 OFFSET ?4")
            .map_err(|e| e.to_string())?;
        let messages: Vec<MailMessage> = stmt.query_map(params![account_id, folder_name, limit, offset], |row| {
            let sender_str: String = row.get(3)?;
            let recipients_str: String = row.get(4)?;
            let date_str: Option<String> = row.get(5)?;
            let is_read: i32 = row.get(6)?;
            let is_flagged: i32 = row.get(7)?;
            let has_attachments: i32 = row.get(8)?;
            let attachments_str: String = row.get(12)?;

            let from: Vec<MailAddress> = serde_json::from_str(&sender_str).unwrap_or_default();
            let to: Vec<MailAddress> = serde_json::from_str(&recipients_str).unwrap_or_default();
            let attachments: Vec<Attachment> = serde_json::from_str(&attachments_str).unwrap_or_default();

            Ok(MailMessage {
                id: row.get(0)?,
                folder: folder_name.to_string(),
                uid: row.get(1)?,
                subject: row.get(2)?,
                from,
                to,
                cc: Vec::new(),
                date: date_str.and_then(|s| s.parse().ok()),
                flags: Vec::new(),
                is_read: is_read != 0,
                is_flagged: is_flagged != 0,
                has_attachments: has_attachments != 0,
                preview: row.get(9)?,
                body_text: row.get(10)?,
                body_html: row.get(11)?,
                attachments,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(messages)
    }

    pub fn get_last_uid(&self, account_id: &str, folder_name: &str) -> Result<u32, String> {
        let result: Option<u32> = self.conn.query_row(
            "SELECT MAX(uid) FROM messages WHERE account_id = ?1 AND folder_name = ?2",
            params![account_id, folder_name],
            |row| row.get(0),
        ).ok();
        Ok(result.unwrap_or(0))
    }

    pub fn delete_account(&self, account_id: &str) -> Result<(), String> {
        self.conn.execute("DELETE FROM messages WHERE account_id = ?1", params![account_id]).map_err(|e| e.to_string())?;
        self.conn.execute("DELETE FROM folders WHERE account_id = ?1", params![account_id]).map_err(|e| e.to_string())?;
        self.conn.execute("DELETE FROM accounts WHERE id = ?1", params![account_id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_account(&self, account: &MailAccount) -> Result<(), String> {
        self.conn.execute(
            "UPDATE accounts SET display_name = ?1, email_address = ?2, imap_host = ?3, imap_port = ?4, smtp_host = ?5, smtp_port = ?6, username = ?7, password = ?8, use_ssl = ?9 WHERE id = ?10",
            params![
                account.display_name,
                account.email_address,
                account.imap_host,
                account.imap_port,
                account.smtp_host,
                account.smtp_port,
                account.username,
                encrypt_password(&account.password),
                account.use_ssl as i32,
                account.id,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_message_body(
        &self,
        message_id: &str,
        body_text: Option<&str>,
        body_html: Option<&str>,
        preview: &str,
        has_attachments: bool,
        attachments: &[Attachment],
    ) -> Result<(), String> {
        let attachments_json = serde_json::to_string(attachments).unwrap_or_else(|_| "[]".to_string());
        self.conn.execute(
            "UPDATE messages SET body_text = ?1, body_html = ?2, preview = ?3, has_attachments = ?4, attachments = ?5 WHERE id = ?6",
            params![body_text, body_html, preview, has_attachments as i32, attachments_json, message_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_message(&self, message_id: &str) -> Result<Option<MailMessage>, String> {
        let result = self.conn.query_row(
            "SELECT id, uid, subject, sender, recipients, date, is_read, is_flagged, has_attachments, preview, body_text, body_html, folder_name, attachments FROM messages WHERE id = ?1",
            params![message_id],
            |row| {
                let sender_str: String = row.get(3)?;
                let recipients_str: String = row.get(4)?;
                let date_str: Option<String> = row.get(5)?;
                let is_read: i32 = row.get(6)?;
                let is_flagged: i32 = row.get(7)?;
                let has_attachments: i32 = row.get(8)?;
                let folder_name: String = row.get(12)?;
                let attachments_str: String = row.get(13)?;

                let from: Vec<MailAddress> = serde_json::from_str(&sender_str).unwrap_or_default();
                let to: Vec<MailAddress> = serde_json::from_str(&recipients_str).unwrap_or_default();
                let attachments: Vec<Attachment> = serde_json::from_str(&attachments_str).unwrap_or_default();

                Ok(MailMessage {
                    id: row.get(0)?,
                    folder: folder_name,
                    uid: row.get(1)?,
                    subject: row.get(2)?,
                    from,
                    to,
                    cc: Vec::new(),
                    date: date_str.and_then(|s| s.parse().ok()),
                    flags: Vec::new(),
                    is_read: is_read != 0,
                    is_flagged: is_flagged != 0,
                    has_attachments: has_attachments != 0,
                    preview: row.get(9)?,
                    body_text: row.get(10)?,
                    body_html: row.get(11)?,
                    attachments,
                })
            },
        );

        match result {
            Ok(msg) => Ok(Some(msg)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn toggle_message_read(&self, message_id: &str, is_read: bool) -> Result<(), String> {
        self.conn.execute(
            "UPDATE messages SET is_read = ?1 WHERE id = ?2",
            params![is_read as i32, message_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn toggle_message_flagged(&self, message_id: &str, is_flagged: bool) -> Result<(), String> {
        self.conn.execute(
            "UPDATE messages SET is_flagged = ?1 WHERE id = ?2",
            params![is_flagged as i32, message_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_message(&self, message_id: &str) -> Result<(), String> {
        self.conn.execute(
            "DELETE FROM messages WHERE id = ?1",
            params![message_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn mark_message_read(&self, message_id: &str, is_read: bool) -> Result<(), String> {
        self.conn.execute(
            "UPDATE messages SET is_read = ?1 WHERE id = ?2",
            params![is_read as i32, message_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn mark_message_flagged(&self, message_id: &str, is_flagged: bool) -> Result<(), String> {
        self.conn.execute(
            "UPDATE messages SET is_flagged = ?1 WHERE id = ?2",
            params![is_flagged as i32, message_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn search_messages(
        &self,
        account_id: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<MailMessage>, String> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn
            .prepare(
                "SELECT id, uid, subject, sender, recipients, date, is_read, is_flagged, has_attachments, preview, body_text, body_html, folder_name, attachments \
                 FROM messages \
                 WHERE account_id = ?1 \
                 AND (subject LIKE ?2 OR sender LIKE ?2 OR recipients LIKE ?2) \
                 ORDER BY date DESC LIMIT ?3"
            )
            .map_err(|e| e.to_string())?;
        let messages: Vec<MailMessage> = stmt.query_map(params![account_id, pattern, limit], |row| {
            let sender_str: String = row.get(3)?;
            let recipients_str: String = row.get(4)?;
            let date_str: Option<String> = row.get(5)?;
            let is_read: i32 = row.get(6)?;
            let is_flagged: i32 = row.get(7)?;
            let has_attachments: i32 = row.get(8)?;
            let folder_name: String = row.get(12)?;
            let attachments_str: String = row.get(13)?;

            let from: Vec<MailAddress> = serde_json::from_str(&sender_str).unwrap_or_default();
            let to: Vec<MailAddress> = serde_json::from_str(&recipients_str).unwrap_or_default();
            let attachments: Vec<Attachment> = serde_json::from_str(&attachments_str).unwrap_or_default();

            Ok(MailMessage {
                id: row.get(0)?,
                folder: folder_name,
                uid: row.get(1)?,
                subject: row.get(2)?,
                from,
                to,
                cc: Vec::new(),
                date: date_str.and_then(|s| s.parse().ok()),
                flags: Vec::new(),
                is_read: is_read != 0,
                is_flagged: is_flagged != 0,
                has_attachments: has_attachments != 0,
                preview: row.get(9)?,
                body_text: row.get(10)?,
                body_html: row.get(11)?,
                attachments,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(messages)
    }

    pub fn get_thread_messages(
        &self,
        account_id: &str,
        subject: &str,
    ) -> Result<Vec<MailMessage>, String> {
        let clean_subject = subject
            .trim_start_matches("Re:")
            .trim_start_matches("Fwd:")
            .trim_start_matches("回复:")
            .trim_start_matches("转发:")
            .trim();
        let pattern = format!("%{}%", clean_subject);

        let mut stmt = self.conn
            .prepare(
                "SELECT id, uid, subject, sender, recipients, date, is_read, is_flagged, has_attachments, preview, body_text, body_html, folder_name, attachments \
                 FROM messages \
                 WHERE account_id = ?1 \
                 AND (subject = ?2 OR subject LIKE ?3) \
                 ORDER BY date ASC"
            )
            .map_err(|e| e.to_string())?;

        let messages: Vec<MailMessage> = stmt.query_map(params![account_id, clean_subject, pattern], |row| {
            let sender_str: String = row.get(3)?;
            let recipients_str: String = row.get(4)?;
            let date_str: Option<String> = row.get(5)?;
            let is_read: i32 = row.get(6)?;
            let is_flagged: i32 = row.get(7)?;
            let has_attachments: i32 = row.get(8)?;
            let folder_name: String = row.get(12)?;
            let attachments_str: String = row.get(13)?;

            let from: Vec<MailAddress> = serde_json::from_str(&sender_str).unwrap_or_default();
            let to: Vec<MailAddress> = serde_json::from_str(&recipients_str).unwrap_or_default();
            let attachments: Vec<Attachment> = serde_json::from_str(&attachments_str).unwrap_or_default();

            Ok(MailMessage {
                id: row.get(0)?,
                folder: folder_name,
                uid: row.get(1)?,
                subject: row.get(2)?,
                from,
                to,
                cc: Vec::new(),
                date: date_str.and_then(|s| s.parse().ok()),
                flags: Vec::new(),
                is_read: is_read != 0,
                is_flagged: is_flagged != 0,
                has_attachments: has_attachments != 0,
                preview: row.get(9)?,
                body_text: row.get(10)?,
                body_html: row.get(11)?,
                attachments,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(messages)
    }

    pub fn get_message_uid_and_folder(&self, message_id: &str) -> Result<Option<(u32, String)>, String> {
        let result = self.conn.query_row(
            "SELECT uid, folder_name FROM messages WHERE id = ?1",
            params![message_id],
            |row| {
                let uid: u32 = row.get(0)?;
                let folder_name: String = row.get(1)?;
                Ok((uid, folder_name))
            },
        );

        match result {
            Ok(pair) => Ok(Some(pair)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn save_draft(
        &self,
        account_id: &str,
        subject: &str,
        sender: &str,
        recipients: &str,
        body_text: Option<&str>,
        body_html: Option<&str>,
    ) -> Result<String, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let preview = body_text
            .map(|t| t.chars().take(200).collect::<String>())
            .unwrap_or_default();

        self.conn.execute(
            "INSERT INTO messages (id, account_id, folder_name, uid, subject, sender, recipients, date, is_read, is_flagged, has_attachments, preview, body_text, body_html, attachments)
             VALUES (?1, ?2, 'Drafts', 0, ?3, ?4, ?5, ?6, 1, 0, 0, ?7, ?8, ?9, '[]')",
            params![id, account_id, subject, sender, recipients, now, preview, body_text, body_html],
        ).map_err(|e| e.to_string())?;
        Ok(id)
    }

    pub fn get_unread_count(&self, account_id: &str, folder_name: &str) -> Result<u32, String> {
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE account_id = ?1 AND folder_name = ?2 AND is_read = 0",
            params![account_id, folder_name],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        Ok(count)
    }

    pub fn update_folder_unread_count(&self, account_id: &str, folder_name: &str) -> Result<u32, String> {
        let count = self.get_unread_count(account_id, folder_name)?;
        self.conn.execute(
            "UPDATE folders SET unread_count = ?1 WHERE account_id = ?2 AND name = ?3",
            params![count, account_id, folder_name],
        ).map_err(|e| e.to_string())?;
        Ok(count)
    }

    pub fn move_message_to_folder(&self, message_id: &str, dest_folder: &str) -> Result<(), String> {
        self.conn.execute(
            "UPDATE messages SET folder_name = ?1 WHERE id = ?2",
            params![dest_folder, message_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_all_inbox_messages(&self, account_id: &str, limit: u32) -> Result<Vec<MailMessage>, String> {
        let mut stmt = self.conn
            .prepare(
                "SELECT m.id, m.uid, m.subject, m.sender, m.recipients, m.date, m.is_read, m.is_flagged, m.has_attachments, m.preview, m.body_text, m.body_html, m.folder_name, m.attachments \
                 FROM messages m \
                 INNER JOIN folders f ON m.account_id = f.account_id AND m.folder_name = f.name \
                 WHERE m.account_id = ?1 AND f.special_use = 'Inbox' \
                 ORDER BY m.date DESC LIMIT ?2"
            )
            .map_err(|e| e.to_string())?;
        let messages: Vec<MailMessage> = stmt.query_map(params![account_id, limit], |row| {
            let sender_str: String = row.get(3)?;
            let recipients_str: String = row.get(4)?;
            let date_str: Option<String> = row.get(5)?;
            let is_read: i32 = row.get(6)?;
            let is_flagged: i32 = row.get(7)?;
            let has_attachments: i32 = row.get(8)?;
            let folder_name: String = row.get(12)?;
            let attachments_str: String = row.get(13)?;

            let from: Vec<MailAddress> = serde_json::from_str(&sender_str).unwrap_or_default();
            let to: Vec<MailAddress> = serde_json::from_str(&recipients_str).unwrap_or_default();
            let attachments: Vec<Attachment> = serde_json::from_str(&attachments_str).unwrap_or_default();

            Ok(MailMessage {
                id: row.get(0)?,
                folder: folder_name,
                uid: row.get(1)?,
                subject: row.get(2)?,
                from,
                to,
                cc: Vec::new(),
                date: date_str.and_then(|s| s.parse().ok()),
                flags: Vec::new(),
                is_read: is_read != 0,
                is_flagged: is_flagged != 0,
                has_attachments: has_attachments != 0,
                preview: row.get(9)?,
                body_text: row.get(10)?,
                body_html: row.get(11)?,
                attachments,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(messages)
    }

    pub fn save_rule(&self, rule: &MailRule) -> Result<(), String> {
        self.conn.execute(
            "INSERT OR REPLACE INTO rules (id, account_id, field, operator, value, action, action_value, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                rule.id,
                rule.account_id,
                format!("{:?}", rule.field).to_lowercase(),
                format!("{:?}", rule.operator).to_lowercase(),
                rule.value,
                format!("{:?}", rule.action).to_lowercase(),
                rule.action_value,
                rule.enabled as i32,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_rules(&self, account_id: &str) -> Result<Vec<MailRule>, String> {
        let mut stmt = self.conn
            .prepare("SELECT id, account_id, field, operator, value, action, action_value, enabled FROM rules WHERE account_id = ?1")
            .map_err(|e| e.to_string())?;
        let rules: Vec<MailRule> = stmt.query_map(params![account_id], |row| {
            let field_str: String = row.get(2)?;
            let field = match field_str.as_str() {
                "from" => RuleField::From,
                "to" => RuleField::To,
                _ => RuleField::Subject,
            };
            let operator_str: String = row.get(3)?;
            let operator = match operator_str.as_str() {
                "equals" => RuleOperator::Equals,
                "startswith" => RuleOperator::StartsWith,
                "endswith" => RuleOperator::EndsWith,
                _ => RuleOperator::Contains,
            };
            let action_str: String = row.get(5)?;
            let action = match action_str.as_str() {
                "mark-read" => RuleAction::MarkRead,
                "mark-flagged" => RuleAction::MarkFlagged,
                "delete" => RuleAction::Delete,
                _ => RuleAction::MoveTo,
            };
            let enabled: i32 = row.get(7)?;
            Ok(MailRule {
                id: row.get(0)?,
                account_id: row.get(1)?,
                field,
                operator,
                value: row.get(4)?,
                action,
                action_value: row.get(6)?,
                enabled: enabled != 0,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r: Result<MailRule, _>| r.ok()).collect();
        Ok(rules)
    }

    pub fn delete_rule(&self, rule_id: &str) -> Result<(), String> {
        self.conn.execute("DELETE FROM rules WHERE id = ?1", params![rule_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_applicable_rules(&self, account_id: &str) -> Result<Vec<MailRule>, String> {
        let mut stmt = self.conn
            .prepare("SELECT id, account_id, field, operator, value, action, action_value, enabled FROM rules WHERE account_id = ?1 AND enabled = 1")
            .map_err(|e| e.to_string())?;
        let rules: Vec<MailRule> = stmt.query_map(params![account_id], |row| {
            let field_str: String = row.get(2)?;
            let field = match field_str.as_str() {
                "from" => RuleField::From,
                "to" => RuleField::To,
                _ => RuleField::Subject,
            };
            let operator_str: String = row.get(3)?;
            let operator = match operator_str.as_str() {
                "equals" => RuleOperator::Equals,
                "startswith" => RuleOperator::StartsWith,
                "endswith" => RuleOperator::EndsWith,
                _ => RuleOperator::Contains,
            };
            let action_str: String = row.get(5)?;
            let action = match action_str.as_str() {
                "mark-read" => RuleAction::MarkRead,
                "mark-flagged" => RuleAction::MarkFlagged,
                "delete" => RuleAction::Delete,
                _ => RuleAction::MoveTo,
            };
            let enabled: i32 = row.get(7)?;
            Ok(MailRule {
                id: row.get(0)?,
                account_id: row.get(1)?,
                field,
                operator,
                value: row.get(4)?,
                action,
                action_value: row.get(6)?,
                enabled: enabled != 0,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r: Result<MailRule, _>| r.ok()).collect();
        Ok(rules)
    }

    pub fn save_folder_mapping(&self, account_id: &str, mapping: &FolderMapping) -> Result<(), String> {
        let json = serde_json::to_string(mapping).map_err(|e| e.to_string())?;
        self.conn.execute(
            "UPDATE accounts SET folder_mapping = ?1 WHERE id = ?2",
            params![json, account_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_folder_mapping(&self, account_id: &str) -> Result<FolderMapping, String> {
        let mut stmt = self.conn
            .prepare("SELECT folder_mapping FROM accounts WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let json: String = stmt.query_row(params![account_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }

    pub fn get_recent_messages(&self, account_id: &str, folder_name: &str, limit: u32) -> Result<Vec<MailMessage>, String> {
        let mut stmt = self.conn
            .prepare(
                "SELECT id, account_id, folder_name, uid, subject, sender, recipients, cc, date, flags, is_read, is_flagged, has_attachments, preview, body_text, body_html, attachments
                 FROM messages WHERE account_id = ?1 AND folder_name = ?2
                 ORDER BY date DESC LIMIT ?3"
            )
            .map_err(|e| e.to_string())?;
        let messages: Vec<MailMessage> = stmt.query_map(params![account_id, folder_name, limit], |row| {
            let sender_json: String = row.get(5)?;
            let recipients_json: String = row.get(6)?;
            let cc_json: String = row.get(7)?;
            let flags_json: String = row.get(9)?;
            let attachments_json: String = row.get(15)?;
            let from: Vec<MailAddress> = serde_json::from_str(&sender_json).unwrap_or_default();
            let to: Vec<MailAddress> = serde_json::from_str(&recipients_json).unwrap_or_default();
            let cc: Vec<MailAddress> = serde_json::from_str(&cc_json).unwrap_or_default();
            let flags: Vec<String> = serde_json::from_str(&flags_json).unwrap_or_default();
            let attachments: Vec<Attachment> = serde_json::from_str(&attachments_json).unwrap_or_default();
            let is_read: i32 = row.get(10)?;
            let is_flagged: i32 = row.get(11)?;
            let has_attachments: i32 = row.get(12)?;
            let date_str: Option<String> = row.get(8)?;
            Ok(MailMessage {
                id: row.get(0)?,
                from,
                to,
                cc,
                date: date_str.and_then(|s| s.parse().ok()),
                flags,
                is_read: is_read != 0,
                is_flagged: is_flagged != 0,
                has_attachments: has_attachments != 0,
                subject: row.get(4)?,
                preview: row.get(13)?,
                body_text: row.get(14)?,
                body_html: row.get(15)?,
                attachments,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r: Result<MailMessage, _>| r.ok()).collect();
        Ok(messages)
    }
}