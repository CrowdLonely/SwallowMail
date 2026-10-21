use rusqlite::{Connection, params};
use crate::mail::model::{MailAccount, MailFolder, MailMessage};

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
                account.password,
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
        let accounts = stmt.query_map([], |row| {
            Ok(())
        }).map_err(|e| e.to_string())?;
        Ok(Vec::new())
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
}
