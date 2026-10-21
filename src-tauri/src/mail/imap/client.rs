use imap::Session;
use native_tls::TlsStream;
use std::net::TcpStream;
use crate::mail::model::MailAccount;
use crate::mail::imap::ImapError;

pub type TlsSession = Session<TlsStream<TcpStream>>;

pub fn connect(account: &MailAccount) -> Result<TlsSession, ImapError> {
    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|e| ImapError::TlsError(e))?;

    let client = imap::connect(
        (account.imap_host.as_str(), account.imap_port),
        account.imap_host.as_str(),
        &tls,
    )
    .map_err(|e| ImapError::ConnectionFailed(e.to_string()))?;

    let session = client
        .login(account.username.as_str(), account.password.as_str())
        .map_err(|e| ImapError::AuthFailed(e.0.to_string()))?;

    Ok(session)
}

pub fn fetch_folder_list(session: &mut TlsSession) -> Result<Vec<(String, String, Vec<String>)>, ImapError> {
    let mailboxes = session
        .list(Some(""), Some("*"))
        .map_err(|e| ImapError::ProtocolError(e.to_string()))?;

    let mut folders = Vec::new();
    for mb in mailboxes.iter() {
        let name = mb.name().to_string();
        let delimiter = mb.delimiter().to_string();
        let attributes: Vec<String> = mb.attributes()
            .iter()
            .map(|a| format!("{:?}", a))
            .collect();
        folders.push((name, delimiter, attributes));
    }
    Ok(folders)
}
