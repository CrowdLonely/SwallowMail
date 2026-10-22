import { useState, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useMailStore, MailAddress, ComposeAttachment } from "../../stores/mailStore";
import { useSettingsStore } from "../../stores/settingsStore";

export interface ComposeInitialData {
  to: string;
  cc: string;
  subject: string;
  body: string;
  bodyHtml?: string;
}

export function ComposeWindow({
  onClose,
  initialData,
}: {
  onClose: () => void;
  initialData?: ComposeInitialData | null;
}) {
  const { t } = useTranslation();
  const { accounts, selectedAccountId, sendMail, sending, saveDraft } = useMailStore();
  const { getSignature, defaultHtmlMode } = useSettingsStore();
  const account = accounts.find((a) => a.id === selectedAccountId);
  const signature = account ? getSignature(account.id) : "";
  const [to, setTo] = useState(initialData?.to || "");
  const [cc, setCc] = useState(initialData?.cc || "");
  const [subject, setSubject] = useState(initialData?.subject || "");
  const [body, setBody] = useState(initialData?.body || "");
  const [bodyHtml, setBodyHtml] = useState(initialData?.bodyHtml || "");
  const [htmlMode, setHtmlMode] = useState(initialData?.bodyHtml ? true : defaultHtmlMode);
  const [error, setError] = useState<string | null>(null);
  const [attachments, setAttachments] = useState<ComposeAttachment[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const parseAddresses = (text: string): MailAddress[] => {
    return text
      .split(/[,;]/)
      .map((s) => s.trim())
      .filter((s) => s.length > 0)
      .map((s) => {
        const match = s.match(/^(.+?)\s*<(.+?)>$/);
        if (match) {
          return { name: match[1].trim(), address: match[2].trim() };
        }
        return { name: null, address: s };
      });
  };

  const handleSend = async () => {
    setError(null);
    const account = accounts.find((a) => a.id === selectedAccountId);
    if (!account) {
      setError("请先选择一个账户");
      return;
    }

    const toAddrs = parseAddresses(to);
    if (toAddrs.length === 0) {
      setError("请输入收件人地址");
      return;
    }

    const ccAddrs = parseAddresses(cc);
    const fromAddr: MailAddress = {
      name: account.display_name || null,
      address: account.email_address,
    };

    try {
      const finalBody = signature ? `${body}\n\n-- \n${signature}` : body;
      const finalHtml = htmlMode
        ? (signature ? `${bodyHtml}<br><br><div style="color:#666;border-top:1px solid #ccc;padding-top:4px">${signature.replace(/\n/g, "<br>")}</div>` : bodyHtml)
        : null;
      await sendMail(account, fromAddr, toAddrs, ccAddrs, subject, finalBody, finalHtml, attachments);
      onClose();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleSaveDraft = async () => {
    setError(null);
    const account = accounts.find((a) => a.id === selectedAccountId);
    if (!account) {
      setError("请先选择一个账户");
      return;
    }

    const sender = JSON.stringify([{ name: account.display_name || null, address: account.email_address }]);
    const recipients = JSON.stringify(parseAddresses(to));

    try {
      const htmlContent = htmlMode ? bodyHtml : null;
      await saveDraft(account.id, subject, sender, recipients, body, htmlContent);
      onClose();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="compose-window">
      <div className="compose-header">
        <h3>{t("compose.title")}</h3>
        <button onClick={onClose}>✕</button>
      </div>
      <div className="compose-form">
        <input placeholder={t("compose.toPlaceholder")} value={to} onChange={(e) => setTo(e.target.value)} disabled={sending} />
        <input placeholder={t("compose.ccPlaceholder")} value={cc} onChange={(e) => setCc(e.target.value)} disabled={sending} />
        <input placeholder={t("compose.subjectPlaceholder")} value={subject} onChange={(e) => setSubject(e.target.value)} disabled={sending} />
        <div className="compose-body-wrapper">
          <div className="compose-body-toolbar">
            <button
              className={`mode-btn ${!htmlMode ? "active" : ""}`}
              onClick={() => setHtmlMode(false)}
              type="button"
            >
              {t("compose.plainText")}
            </button>
            <button
              className={`mode-btn ${htmlMode ? "active" : ""}`}
              onClick={() => setHtmlMode(true)}
              type="button"
            >
              {t("compose.richText")}
            </button>
          </div>
          {htmlMode ? (
            <div
              className="compose-html-editor"
              contentEditable
              suppressContentEditableWarning
              onInput={(e) => setBodyHtml(e.currentTarget.innerHTML)}
              dangerouslySetInnerHTML={{ __html: bodyHtml }}
            />
          ) : (
            <textarea placeholder={t("compose.bodyPlaceholder")} value={body} onChange={(e) => setBody(e.target.value)} rows={12} disabled={sending} />
          )}
        </div>
      </div>
      {error && <p className="compose-error">{error}</p>}
      {attachments.length > 0 && (
        <div className="compose-attachments">
          {attachments.map((att, idx) => (
            <span key={idx} className="compose-attachment-badge">
              📎 {att.filename}
              <button className="remove-att-btn" onClick={() => setAttachments((prev) => prev.filter((_, i) => i !== idx))}>✕</button>
            </span>
          ))}
        </div>
      )}
      <div className="compose-actions">
        <button className="send-btn" onClick={handleSend} disabled={sending || !to}>
          {sending ? t("app.loading") : t("compose.send")}
        </button>
        <button className="draft-btn" onClick={handleSaveDraft}>{t("compose.saveDraft")}</button>
        <button className="attach-btn" onClick={() => fileInputRef.current?.click()} title={t("compose.addAttachment")}>📎</button>
        <button className="discard-btn" onClick={onClose}>{t("compose.discard")}</button>
        <input
          ref={fileInputRef}
          type="file"
          multiple
          style={{ display: "none" }}
          onChange={async (e) => {
            const files = e.target.files;
            if (!files) return;
            const newAtts: ComposeAttachment[] = [];
            for (let i = 0; i < files.length; i++) {
              const file = files[i];
              const buffer = await file.arrayBuffer();
              newAtts.push({
                filename: file.name,
                content_type: file.type || "application/octet-stream",
                data: Array.from(new Uint8Array(buffer)),
              });
            }
            setAttachments((prev) => [...prev, ...newAtts]);
            e.target.value = "";
          }}
        />
      </div>
    </div>
  );
}