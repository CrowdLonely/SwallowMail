pub mod localize;

use crate::mail::model::{MailFolder, SpecialUse};
use crate::mail::folder::localize::FolderLocalizer;

pub fn build_display_folder(raw_name: &str, delimiter: &str, attributes: &[String], server_host: &str) -> MailFolder {
    let special_use = detect_special_use(attributes, raw_name);
    let localizer = FolderLocalizer::new("zh-CN");
    let display_name = localizer.localize(raw_name, server_host, &special_use);

    MailFolder {
        name: raw_name.to_string(),
        display_name,
        delimiter: delimiter.to_string(),
        attributes: attributes.to_vec(),
        unread_count: 0,
        total_count: 0,
        special_use,
    }
}

fn detect_special_use(attributes: &[String], name: &str) -> Option<SpecialUse> {
    for attr in attributes {
        match attr.as_str() {
            "\\Inbox" | "\\All" => return Some(SpecialUse::Inbox),
            "\\Sent" => return Some(SpecialUse::Sent),
            "\\Drafts" => return Some(SpecialUse::Drafts),
            "\\Trash" => return Some(SpecialUse::Trash),
            "\\Junk" | "\\Spam" => return Some(SpecialUse::Junk),
            "\\Archive" => return Some(SpecialUse::Archive),
            "\\Flagged" | "\\Starred" => return Some(SpecialUse::Flagged),
            "\\Important" => return Some(SpecialUse::Important),
            _ => {}
        }
    }
    if name.eq_ignore_ascii_case("INBOX") {
        return Some(SpecialUse::Inbox);
    }
    None
}
