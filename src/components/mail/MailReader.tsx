import { useTranslation } from "react-i18next";
import { useMailStore } from "../../stores/mailStore";

export function MailReader() {
  const { t } = useTranslation();
  const { selectedMail } = useMailStore();

  if (!selectedMail) return null;

  const formatAddressList = (addrs: { name: string | null; address: string }[]): string => {
    return addrs.map((a) => a.name ? `${a.name} <${a.address}>` : a.address).join(", ");
  };

  return (
    <div className="mail-reader">
      <div className="mail-reader-toolbar">
        <button title={t("mail.reply")}>↩ {t("mail.reply")}</button>
        <button title={t("mail.replyAll")}>↩↩ {t("mail.replyAll")}</button>
        <button title={t("mail.forward")}>↗ {t("mail.forward")}</button>
        <button title={t("mail.delete")}>🗑️</button>
        <button title={selectedMail.is_read ? t("mail.markUnread") : t("mail.markRead")}>
          {selectedMail.is_read ? "📭" : "📬"}
        </button>
      </div>

      <div className="mail-reader-header">
        <h1 className="mail-subject">{selectedMail.subject || t("mail.noSubject")}</h1>
        <div className="mail-meta">
          <div className="mail-from">
            <span className="label">{t("mail.from")}:</span>
            <span>{formatAddressList(selectedMail.from)}</span>
          </div>
          <div className="mail-to">
            <span className="label">{t("mail.to")}:</span>
            <span>{formatAddressList(selectedMail.to)}</span>
          </div>
          {selectedMail.cc.length > 0 && (
            <div className="mail-cc">
              <span className="label">{t("mail.cc")}:</span>
              <span>{formatAddressList(selectedMail.cc)}</span>
            </div>
          )}
          <div className="mail-date">
            <span className="label">{t("mail.date")}:</span>
            <span>{selectedMail.date ? new Date(selectedMail.date).toLocaleString("zh-CN") : ""}</span>
          </div>
        </div>
      </div>

      <div className="mail-reader-body">
        {selectedMail.body_html ? (
          <iframe
            srcDoc={selectedMail.body_html}
            className9="mail-html-frame"
            sandbox="allow-same-origin"
            title="邮件内容"
          />
        ) : (
          <pre className="mail-text-body">{selectedMail.body_text || selectedMail.preview}</pre>
        )}
      </div>
    </div>
  );
}
