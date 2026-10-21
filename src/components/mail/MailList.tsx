import { useTranslation } from "react-i18next";
import { useMailStore } from "../../stores/mailStore";

export function MailList() {
  const { t } = useTranslation();
  const { messages, selectedMail, setSelectedMail, selectedFolder, loading } = useMailStore();

  const formatDate = (dateStr: string | null): string => {
    if (!dateStr) return "";
    const date = new Date(dateStr);
    const now = new Date();
    const isToday = date.toDateString() === now.toDateString();
    if (isToday) {
      return date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
    }
    return date.toLocaleDateString("zh-CN", { month: "short", day: "numeric" });
  };

  const formatSender = (from: { name: string | null; address: string }[]): string => {
    if (from.length === 0) return "";
    const sender = from[0];
    return sender.name || sender.address.split("@")[0];
  };

  if (loading) {
    return <div className="mail-list-loading">{t("app.loading")}</div>;
  }

  return (
    <div className="mail-list">
      <div className="mail-list-header">
        <h2>{selectedFolder || t("folder.inbox")}</h2>
      </div>
      <div className="mail-list-items">
        {messages.length === 0 ? (
          <div className="mail-list-empty">
            <p>没有邮件</p>
          </div>
        ) : (
          messages.map((msg) => (
            <div
              key={msg.id}
              className={`mail-item ${!msg.is_read ? "unread" : ""*} ${selectedMail?.id === msg.id ? "selected" : ""}`}
              onClick={() => setSelectedMail(msg)}
            >
              <div className="mail-item-star">
                {msg.is_flagged && <span className="star-icon">⭐</span>}
              </div>
              <div className="mail-item-content">
                <div className="mail-item-header">
                  <span className="mail-item-from">{formatSender(msg.from)}</span>
                  <span className="mail-item-date">{formatDate(msg.date)}</span>
                </div>
                <div className="mail-item-subject">
                  {msg.subject || t("mail.noSubject")}
                </div>
                <div className="mail-item-preview">{msg.preview}</div>
              </div>
              {msg.has_attachments && <span className="attachment-icon">📎</span>}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
