import { useEffect, useState, useRef } from "react";
import { useTranslation } from "react-i18next";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useMailStore, MailMessage, Attachment } from "../../stores/mailStore";
import { ComposeInitialData } from "../compose/ComposeWindow";

export function MailReader({ onCompose }: { onCompose: (data?: ComposeInitialData | null) => void }) {
  const { t } = useTranslation();
  const {
    selectedMail, selectedAccountId, accounts, folders,
    fetchBody, loading, toggleRead, toggleFlagged, deleteMessage,
    threadMessages, threadLoading, loadThread, clearThread, saveAttachment,
    moveMessage,
  } = useMailStore();
  const [showMoveMenu, setShowMoveMenu] = useState(false);

  useEffect(() => {
    if (!selectedMail || !selectedAccountId) return;
    if (selectedMail.body_text || selectedMail.body_html) return;

    const account = accounts.find((a) => a.id === selectedAccountId);
    if (!account) return;

    fetchBody(account, selectedMail.id, selectedMail.folder, selectedMail.uid).catch(() => {});
  }, [selectedMail?.id]);

  useEffect(() => {
    if (!selectedMail || !selectedAccountId) {
      clearThread();
      return;
    }
    loadThread(selectedAccountId, selectedMail.subject);
  }, [selectedMail?.id]);

  if (!selectedMail) return null;

  const formatAddressList = (addrs: { name: string | null; address: string }[]): string => {
    return addrs.map((a) => a.name ? `${a.name} <${a.address}>` : a.address).join(", ");
  };

  const addressToString = (addr: { name: string | null; address: string }): string => {
    return addr.name ? `${addr.name} <${addr.address}>` : addr.address;
  };

  const showLoading = loading || (!selectedMail.body_text && !selectedMail.body_html);
  const htmlFrameRef = useRef<HTMLIFrameElement>(null);

  useEffect(() => {
    const frame = htmlFrameRef.current;
    if (!frame) return;
    const handler = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      const anchor = target.closest("a") as HTMLAnchorElement | null;
      if (anchor?.href) {
        e.preventDefault();
        e.stopPropagation();
        openUrl(anchor.href).catch(() => { window.open(anchor.href, "_blank"); });
      }
    };
    const doc = frame.contentDocument;
    if (doc) {
      doc.addEventListener("click", handler, true);
      return () => doc.removeEventListener("click", handler, true);
    }
    const onLoad = () => {
      const d = frame.contentDocument;
      if (d) d.addEventListener("click", handler, true);
    };
    frame.addEventListener("load", onLoad);
    return () => frame.removeEventListener("load", onLoad);
  }, [selectedMail?.body_html]);

  const handleToggleRead = () => {
    toggleRead(selectedMail.id, !selectedMail.is_read);
  };

  const handleToggleFlag = () => {
    toggleFlagged(selectedMail.id, !selectedMail.is_flagged);
  };

  const handleDelete = () => {
    deleteMessage(selectedMail.id);
  };

  const handleReply = () => {
    const toStr = selectedMail.from.map(addressToString).join(", ");
    const subjectStr = selectedMail.subject.startsWith("Re:")
      ? selectedMail.subject
      : `Re: ${selectedMail.subject}`;
    const quotedBody = selectedMail.body_text || selectedMail.preview || "";
    const bodyStr = `\n\n--- 原始邮件 ---\n${quotedBody}`;
    const bodyHtmlStr = selectedMail.body_html
      ? `<br><br><blockquote style="border-left:2px solid #ccc;padding-left:8px;margin-left:4px;color:#666">${selectedMail.body_html}</blockquote>`
      : undefined;
    onCompose({ to: toStr, cc: "", subject: subjectStr, body: bodyStr, bodyHtml: bodyHtmlStr });
  };

  const handleReplyAll = () => {
    const account = accounts.find((a) => a.id === selectedAccountId);
    const myEmail = account?.email_address.toLowerCase() || "";
    const fromAddrs = selectedMail.from.map(addressToString);
    const toAddrs = selectedMail.to
      .filter((a) => a.address.toLowerCase() !== myEmail)
      .map(addressToString);
    const allTo = [...fromAddrs, ...toAddrs].join(", ");
    const ccAddrs = selectedMail.cc
      .filter((a) => a.address.toLowerCase() !== myEmail)
      .map(addressToString)
      .join(", ");
    const subjectStr = selectedMail.subject.startsWith("Re:")
      ? selectedMail.subject
      : `Re: ${selectedMail.subject}`;
    const quotedBody = selectedMail.body_text || selectedMail.preview || "";
    const bodyStr = `\n\n--- 原始邮件 ---\n${quotedBody}`;
    const bodyHtmlStr = selectedMail.body_html
      ? `<br><br><blockquote style="border-left:2px solid #ccc;padding-left:8px;margin-left:4px;color:#666">${selectedMail.body_html}</blockquote>`
      : undefined;
    onCompose({ to: allTo, cc: ccAddrs, subject: subjectStr, body: bodyStr, bodyHtml: bodyHtmlStr });
  };

  const handleForward = () => {
    const subjectStr = selectedMail.subject.startsWith("Fwd:")
      ? selectedMail.subject
      : `Fwd: ${selectedMail.subject}`;
    const fromStr = formatAddressList(selectedMail.from);
    const toStr = formatAddressList(selectedMail.to);
    const dateStr = selectedMail.date ? new Date(selectedMail.date).toLocaleString("zh-CN") : "";
    const quotedBody = selectedMail.body_text || selectedMail.preview || "";
    const bodyStr = `\n\n--- 转发邮件 ---\n发件人: ${fromStr}\n收件人: ${toStr}\n日期: ${dateStr}\n主题: ${selectedMail.subject}\n\n${quotedBody}`;
    const fwdHeader = `<div style="color:#666;font-size:0.9em">---------- 转发邮件 ----------<br>发件人: ${fromStr}<br>收件人: ${toStr}<br>日期: ${dateStr}<br>主题: ${selectedMail.subject}</div>`;
    const bodyHtmlStr = selectedMail.body_html
      ? `<br><br>${fwdHeader}<br><br>${selectedMail.body_html}`
      : undefined;
    onCompose({ to: "", cc: "", subject: subjectStr, body: bodyStr, bodyHtml: bodyHtmlStr });
  };

  const showThread = threadMessages.length > 1;

  return (
    <div className="mail-reader">
      <div className="mail-reader-toolbar">
        <button title={t("mail.reply")} onClick={handleReply}>↩ {t("mail.reply")}</button>
        <button title={t("mail.replyAll")} onClick={handleReplyAll}>↩↩ {t("mail.replyAll")}</button>
        <button title={t("mail.forward")} onClick={handleForward}>↗ {t("mail.forward")}</button>
        <button title={t("mail.delete")} onClick={handleDelete}>🗑️</button>
        <button
          title={selectedMail.is_read ? t("mail.markUnread") : t("mail.markRead")}
          onClick={handleToggleRead}
        >
          {selectedMail.is_read ? "📭" : "📬"}
        </button>
        <button
          title={selectedMail.is_flagged ? t("mail.unflag") : t("mail.flag")}
          onClick={handleToggleFlag}
        >
          {selectedMail.is_flagged ? "⭐" : "☆"}
        </button>
        <div className="move-menu-wrapper">
          <button title={t("mail.moveTo")} onClick={() => setShowMoveMenu(!showMoveMenu)}>📁</button>
          {showMoveMenu && (
            <div className="move-menu">
              {folders
                .filter((f) => f.name !== selectedMail.folder)
                .map((f) => (
                  <button
                    key={f.name}
                    className="move-menu-item"
                    onClick={() => {
                      moveMessage(selectedMail.id, f.name);
                      setShowMoveMenu(false);
                    }}
                  >
                    {f.display_name || f.name}
                  </button>
                ))}
            </div>
          )}
        </div>
      </div>

      {showThread && (
        <div className="mail-thread-header">
          <span className="thread-count">{t("mail.threadCount", { count: threadMessages.length })}</span>
        </div>
      )}

      <div className="mail-reader-content">
        {showThread ? (
          <div className="mail-thread">
            {threadLoading ? (
              <div className="mail-body-loading">{t("app.loading")}</div>
            ) : (
              threadMessages.map((msg) => (
                <ThreadMessage
                  key={msg.id}
                  message={msg}
                  isCurrent={msg.id === selectedMail.id}
                />
              ))
            )}
          </div>
        ) : (
          <>
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

            {selectedMail.attachments.length > 0 && (
              <div className="mail-attachments">
                <span className="attachments-label">📎 {t("mail.attachments")}</span>
                <div className="attachment-list">
                  {selectedMail.attachments.map((att) => (
                    <AttachmentItem
                      key={att.filename}
                      attachment={att}
                      onSave={() => {
                        const account = accounts.find((a) => a.id === selectedAccountId);
                        if (account) {
                          saveAttachment(account, selectedMail.folder, selectedMail.uid, att.filename);
                        }
                      }}
                    />
                  ))}
                </div>
              </div>
            )}

            <div className="mail-reader-body">
              {showLoading ? (
                <div className="mail-body-loading">{t("app.loading")}</div>
              ) : selectedMail.body_html ? (
                <iframe
                  ref={htmlFrameRef}
                  srcDoc={selectedMail.body_html}
                  className="mail-html-frame"
                  sandbox="allow-same-origin"
                  title="邮件内容"
                />
              ) : (
                <pre className="mail-text-body">{selectedMail.body_text || selectedMail.preview}</pre>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

function ThreadMessage({ message, isCurrent }: { message: MailMessage; isCurrent: boolean }) {
  const formatSender = (from: { name: string | null; address: string }[]): string => {
    if (from.length === 0) return "";
    return from[0].name || from[0].address;
  };

  return (
    <div className={`thread-message ${isCurrent ? "current" : ""}`}>
      <div className="thread-message-header">
        <span className="thread-sender">{formatSender(message.from)}</span>
        <span className="thread-date">
          {message.date ? new Date(message.date).toLocaleString("zh-CN", {
            month: "short", day: "numeric", hour: "2-digit", minute: "2-digit",
          }) : ""}
        </span>
      </div>
      <div className="thread-message-body">
        {message.body_html ? (
          <iframe
            srcDoc={message.body_html}
            className="mail-html-frame"
            sandbox="allow-same-origin"
            title="邮件内容"
          />
        ) : (
          <pre className="mail-text-body">{message.body_text || message.preview}</pre>
        )}
      </div>
      {message.attachments.length > 0 && (
        <div className="thread-attachments">
          {message.attachments.map((att) => (
            <span key={att.filename} className="thread-attachment-badge">
              📎 {att.filename} ({formatFileSize(att.size)})
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

function AttachmentItem({ attachment, onSave }: { attachment: Attachment; onSave: () => void }) {
  return (
    <div className="attachment-item">
      <span className="attachment-icon">📎</span>
      <span className="attachment-filename">{attachment.filename}</span>
      <span className="attachment-size">({formatFileSize(attachment.size)})</span>
      <button className="attachment-save-btn" onClick={onSave}>
        💾
      </button>
    </div>
  );
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}