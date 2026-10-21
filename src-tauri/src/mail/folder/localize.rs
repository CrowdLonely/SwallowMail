use std::collections::HashMap;
use crate::mail::model::SpecialUse;

pub struct FolderLocalizer {
    locale: String,
    special_use_map: HashMap<String, String>,
    provider_maps: HashMap<String, HashMap<String, String>>,
}

impl FolderLocalizer {
    pub fn new(locale: &str) -> Self {
        let mut special_use_map = HashMap::new();
        match locale {
            "zh-CN" | "zh-TW" | "zh-HK" => {
                special_use_map.insert("inbox".to_string(), "收件箱".to_string());
                special_use_map.insert("sent".to_string(), "已发送".to_string());
                special_use_map.insert("drafts".to_string(), "草稿箱".to_string());
                special_use_map.insert("trash".to_string(), "已删除".to_string());
                special_use_map.insert("junk".to_string(), "垃圾邮件".to_string());
                special_use_map.insert("archive".to_string(), "归档".to_string());
                special_use_map.insert("flagged".to_string(), "星标邮件".to_string());
                special_use_map.insert("important".to_string(), "重要邮件".to_string());
            }
            _ => {
                special_use_map.insert("inbox".to_string(), "Inbox".to_string());
                special_use_map.insert("sent".to_string(), "Sent".to_string());
                special_use_map.insert("drafts".to_string(), "Drafts".to_string());
                special_use_map.insert("trash".to_string(), "Trash".to_string());
                special_use_map.insert("junk".to_string(), "Junk".to_string());
                special_use_map.insert("archive".to_string(), "Archive".to_string());
                special_use_map.insert("flagged".to_string(), "Flagged".to_string());
                special_use_map.insert("important".to_string(), "Important".to_string());
            }
        }

        let mut provider_maps = HashMap::new();

        let mut qq = HashMap::new();
        qq.insert("INBOX".to_string(), "收件箱".to_string());
        qq.insert("Sent Messages".to_string(), "已发送".to_string());
        qq.insert("Drafts".to_string(), "草稿箱".to_string());
        qq.insert("Deleted Messages".to_string(), "已删除".to_string());
        qq.insert("Junk".to_string(), "垃圾邮件".to_string());
        provider_maps.insert("imap.qq.com".to_string(), qq);
        provider_maps.insert("imap.foxmail.com".to_string(), {
            let mut m = HashMap::new();
            m.insert("INBOX".to_string(), "收件箱".to_string());
            m.insert("Sent Messages".to_string(), "已发送".to_string());
            m.insert("Drafts".to_string(), "草稿箱".to_string());
            m.insert("Deleted Messages".to_string(), "已删除".to_string());
            m.insert("Junk".to_string(), "垃圾邮件".to_string());
            m
        });

        let mut netease = HashMap::new();
        netease.insert("INBOX".to_string(), "收件箱".to_string());
        netease.insert("Sent".to_string(), "已发送".to_string());
        netease.insert("Drafts".to_string(), "草稿箱".to_string());
        netease.insert("Deleted".to_string(), "已删除".to_string());
        netease.insert("Junk".to_string(), "垃圾邮件".to_string());
        provider_maps.insert("imap.163.com".to_string(), netease.clone());
        provider_maps.insert("imap.126.com".to_string(), netease.clone());
        provider_maps.insert("imap.yeah.net".to_string(), netease);

        let mut sina = HashMap::new();
        sina.insert("INBOX".to_string(), "收件箱".to_string());
        sina.insert("Sent Items".to_string(), "已发送".to_string());
        sina.insert("Drafts".to_string(), "草稿箱".to_string());
        sina.insert("Trash".to_string(), "已删除".to_string());
        sina.insert("Spam".to_string(), "垃圾邮件".to_string());
        provider_maps.insert("imap.sina.com".to_string(), sina);

        let mut gmail = HashMap::new();
        gmail.insert("[Gmail]/已发邮件".to_string(), "已发送".to_string());
        gmail.insert("[Gmail]/草稿".to_string(), "草稿箱".to_string());
        gmail.insert("[Gmail]/已删除邮件".to_string(), "已删除".to_string());
        gmail.insert("[Gmail]/垃圾邮件".to_string(), "垃圾邮件".to_string());
        gmail.insert("[Gmail]/星标".to_string(), "星标邮件".to_string());
        gmail.insert("[Gmail]/重要".to_string(), "重要邮件".to_string());
        gmail.insert("[Gmail]/所有邮件".to_string(), "所有邮件".to_string());
        gmail.insert("[Gmail]/已加星标".to_string(), "星标邮件".to_string());
        gmail.insert("[Gmail]/Spam".to_string(), "垃圾邮件".to_string());
        gmail.insert("[Gmail]/Trash".to_string(), "已删除".to_string());
        gmail.insert("[Gmail]/Drafts".to_string(), "草稿箱".to_string());
        gmail.insert("[Gmail]/Sent Mail".to_string(), "已发送".to_string());
        gmail.insert("[Gmail]/All Mail".to_string(), "所有邮件".to_string());
        gmail.insert("[Gmail]/Starred".to_string(), "星标邮件".to_string());
        gmail.insert("[Gmail]/Important".to_string(), "重要邮件".to_string());
        provider_maps.insert("imap.gmail.com".to_string(), gmail);

        let mut outlook = HashMap::new();
        outlook.insert("Inbox".to_string(), "收件箱".to_string());
        outlook.insert("Sent Items".to_string(), "已发送".to_string());
        outlook.insert("Drafts".to_string(), "草稿箱".to_string());
        outlook.insert("Deleted Items".to_string(), "已删除".to_string());
        outlook.insert("Junk Email".to_string(), "垃圾邮件".to_string());
        outlook.insert("Archive".to_string(), "归档".to_string());
        provider_maps.insert("outlook.office365.com".to_string(), outlook);
        provider_maps.insert("imap-mail.outlook.com".to_string(), {
            let mut m = HashMap::new();
            m.insert("Inbox".to_string(), "收件箱".to_string());
            m.insert("Sent Items".to_string(), "已发送".to_string());
            m.insert("Drafts".to_string(), "草稿箱".to_string());
            m.insert("Deleted Items".to_string(), "已删除".to_string());
            m.insert("Junk Email".to_string(), "垃圾邮件".to_string());
            m
        });

        FolderLocalizer {
            locale: locale.to_string(),
            special_use_map,
            provider_maps,
        }
    }

    pub fn localize(&self, raw_name: &str, server_host: &str, special_use: &Option<SpecialUse>) -> String {
        if let Some(su) = special_use {
            let key = match su {
                SpecialUse::Inbox => "inbox",
                SpecialUse::Sent => "sent",
                SpecialUse::Drafts => "drafts",
                SpecialUse::Trash => "trash",
                SpecialUse::Junk => "junk",
                SpecialUse::Archive => "archive",
                SpecialUse::Flagged => "flagged",
                SpecialUse::Important => "important",
            };
            if let Some(display) = self.special_use_map.get(key) {
                return display.clone();
            }
        }

        if let Some(provider_map) = self.provider_maps.get(server_host) {
            if let Some(display) = provider_map.get(raw_name) {
                return display.clone();
            }
        }

        raw_name.to_string()
    }
}
