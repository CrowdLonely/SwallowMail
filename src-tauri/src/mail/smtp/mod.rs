use lettre::{SmtpTransport, Transport, message::header::ContentType};
use lettre::message::Message;
use crate::mail::model::MailAccount;
use crate::mail::model::MailAddress;

pub struct FileAttachment {
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum SmtpError {
    #[error("构建邮件失败: {0}")]
    BuildFailed(String),
    #[error("发送失败: {0}")]
    SendFailed(String),
}

pub fn create_transport(account: &MailAccount) -> Result<SmtpTransport, SmtpError> {
    let transport = if account.use_ssl {
        SmtpTransport::relay(&account.smtp_host)
            .map_err(|e| SmtpError::BuildFailed(e.to_string()))?
            .port(account.smtp_port)
            .credentials(lettre::transport::smtp::authentication::Credentials::new(
                account.username.clone(),
                account.password.clone(),
            ))
            .build()
    } else {
        SmtpTransport::builder_dangerous(&account.smtp_host)
            .port(account.smtp_port)
            .credentials(lettre::transport::smtp::authentication::Credentials::new(
                account.username.clone(),
                account.password.clone(),
            ))
            .build()
    };
    Ok(transport)
}

pub struct ComposeMail {
    pub from: MailAddress,
    pub to: Vec<MailAddress>,
    pub cc: Vec<MailAddress>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub attachments: Vec<FileAttachment>,
}

pub fn send_mail(account: &MailAccount, compose: &ComposeMail) -> Result<(), SmtpError> {
    let from_addr = format_address(&compose.from);
    let to_addrs: Vec<String> = compose.to.iter().map(format_address).collect();
    let cc_addrs: Vec<String> = compose.cc.iter().map(format_address).collect();

    let mut builder = Message::builder()
        .from(from_addr.parse().map_err(|e| SmtpError::BuildFailed(e.to_string()))?)
        .subject(&compose.subject);

    for addr in &to_addrs {
        builder = builder.to(addr.parse().map_err(|e| SmtpError::BuildFailed(e.to_string()))?);
    }
    for addr in &cc_addrs {
        builder = builder.cc(addr.parse().map_err(|e| SmtpError::BuildFailed(e.to_string()))?);
    }

    let has_attachments = !compose.attachments.is_empty();

    let message = if has_attachments {
        let mut multipart = lettre::message::MultiPart::mixed();

        if let Some(html) = &compose.body_html {
            multipart = multipart.multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(compose.body_text.clone()),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html.clone()),
                    )
            );
        } else {
            multipart = multipart.singlepart(
                lettre::message::SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(compose.body_text.clone()),
            );
        }

        for att in &compose.attachments {
            let content_type: ContentType = att.content_type.parse()
                .unwrap_or_else(|_| ContentType::parse("application/octet-stream").unwrap());
            multipart = multipart.singlepart(
                lettre::message::SinglePart::builder()
                    .header(content_type)
                    .header(lettre::message::header::ContentDisposition::attachment(att.filename.clone()))
                    .body(att.data.clone()),
            );
        }

        builder.multipart(multipart)
    } else if let Some(html) = &compose.body_html {
        builder.multipart(
            lettre::message::MultiPart::alternative()
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(compose.body_text.clone()),
                )
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html.clone()),
                ),
        )
    } else {
        builder.body(compose.body_text.clone())
    }
    .map_err(|e| SmtpError::BuildFailed(e.to_string()))?;

    let transport = create_transport(account)?;
    transport
        .send(&message)
        .map_err(|e| SmtpError::SendFailed(e.to_string()))?;

    Ok(())
}

fn format_address(addr: &MailAddress) -> String {
    match &addr.name {
        Some(name) => format!("{} <{}>", name, addr.address),
        None => addr.address.clone(),
    }
}