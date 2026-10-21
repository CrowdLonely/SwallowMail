use crate::mail::model::MailAccount;
use crate::mail::model::AccountType;
use chrono::Utc;

pub fn create_qq_account(email: &str, password: &str, display_name: &str) -> MailAccount {
    MailAccount {
        id: uuid::Uuid::new_v4().to_string(),
        display_name: display_name.to_string(),
        email_address: email.to_string(),
        account_type: AccountType::QQ,
        imap_host: "imap.qq.com".to_string(),
        imap_port: 993,
        smtp_host: "smtp.qq.com".to_string(),
        smtp_port: 465,
        username: email.to_string(),
        password: password.to_string(),
        use_ssl: true,
        created_at: Utc::now(),
    }
}

pub fn create_163_account(email: &str, password: &str, display_name: &str) -> MailAccount {
    MailAccount {
        id: uuid::Uuid::new_v4().to_string(),
        display_name: display_name.to_string(),
        email_address: email.to_string(),
        account_type: AccountType::NetEase163,
        imap_host: "imap.163.com".to_string(),
        imap_port: 993,
        smtp_host: "smtp.163.com".to_string(),
        smtp_port: 465,
        username: email.to_string(),
        password: password.to_string(),
        use_ssl: true,
        created_at: Utc::now(),
    }
}

pub fn create_126_account(email: &str, password: &str, display_name: &str) -> MailAccount {
    MailAccount {
        id: uuid::Uuid::new_v4().to_string(),
        display_name: display_name.to_string(),
        email_address: email.to_string(),
        account_type: AccountType::NetEase126,
        imap_host: "imap.126.com".to_string(),
        imap_port: 993,
        smtp_host: "smtp.126.com".to_string(),
        smtp_port: 465,
        username: email.to_string(),
        password: password.to_string(),
        use_ssl: true,
        created_at: Utc::now(),
    }
}

pub fn create_gmail_account(email: &str, password: &str, display_name: &str) -> MailAccount {
    MailAccount {
        id: uuid::Uuid::new_v4().to_string(),
        display_name: display_name.to_string(),
        email_address: email.to_string(),
        account_type: AccountType::Gmail,
        imap_host: "imap.gmail.com".to_string(),
        imap_port: 993,
        smtp_host: "smtp.gmail.com".to_string(),
        smtp_port: 465,
        username: email.to_string(),
        password: password.to_string(),
        use_ssl: true,
        created_at: Utc::now(),
    }
}

pub fn create_outlook_account(email: &str, password: &str, display_name: &str) -> MailAccount {
    MailAccount {
        id: uuid::Uuid::new_v4().to_string(),
        display_name: display_name.to_string(),
        email_address: email.to_string(),
        account_type: AccountType::Outlook,
        imap_host: "outlook.office365.com".to_string(),
        imap_port: 993,
        smtp_host: "smtp.office365.com".to_string(),
        smtp_port: 587,
        username: email.to_string(),
        password: password.to_string(),
        use_ssl: true,
        created_at: Utc::now(),
    }
}

pub fn create_custom_account(
    email: &str,
    password: &str,
    display_name: &str,
    imap_host: &str,
    imap_port: u16,
    smtp_host: &str,
    smtp_port: u16,
    use_ssl: bool,
) -> MailAccount {
    MailAccount {
        id: uuid::Uuid::new_v4().to_string(),
        display_name: display_name.to_string(),
        email_address: email.to_string(),
        account_type: AccountType::Imap,
        imap_host: imap_host.to_string(),
        imap_port,
        smtp_host: smtp_host.to_string(),
        smtp_port,
        username: email.to_string(),
        password: password.to_string(),
        use_ssl,
        created_at: Utc::now(),
    }
}
