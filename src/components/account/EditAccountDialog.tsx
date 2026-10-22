import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useMailStore, MailAccount } from "../../stores/mailStore";

export function EditAccountDialog({ accountId, onClose }: { accountId: string; onClose: () => void }) {
  const { t } = useTranslation();
  const { accounts, updateAccount } = useMailStore();
  const existing = accounts.find((a) => a.id === accountId);
  const [displayName, setDisplayName] = useState(existing?.display_name || "");
  const [imapHost, setImapHost] = useState(existing?.imap_host || "");
  const [imapPort, setImapPort] = useState(existing?.imap_port || 993);
  const [smtpHost, setSmtpHost] = useState(existing?.smtp_host || "");
  const [smtpPort, setSmtpPort] = useState(existing?.smtp_port || 465);
  const [username, setUsername] = useState(existing?.username || "");
  const [password, setPassword] = useState(existing?.password || "");
  const [useSsl, setUseSsl] = useState(existing?.use_ssl ?? true);
  const [error, setError] = useState<string | null>(null);

  if (!existing) return null;

  const handleSave = async () => {
    setError(null);
    const updated: MailAccount = {
      ...existing,
      display_name: displayName,
      imap_host: imapHost,
      imap_port: imapPort,
      smtp_host: smtpHost,
      smtp_port: smtpPort,
      username,
      password,
      use_ssl: useSsl,
    };
    try {
      await updateAccount(updated);
      onClose();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog-content" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h3>{t("account.edit")}</h3>
          <button onClick={onClose}>✕</button>
        </div>
        <div className="credentials-form">
          <label>
            {t("account.displayName")}
            <input type="text" value={displayName} onChange={(e) => setDisplayName(e.target.value)} />
          </label>
          <label>
            IMAP Host
            <input type="text" value={imapHost} onChange={(e) => setImapHost(e.target.value)} />
          </label>
          <label>
            IMAP Port
            <input type="number" value={imapPort} onChange={(e) => setImapPort(Number(e.target.value))} />
          </label>
          <label>
            SMTP Host
            <input type="text" value={smtpHost} onChange={(e) => setSmtpHost(e.target.value)} />
          </label>
          <label>
            SMTP Port
            <input type="number" value={smtpPort} onChange={(e) => setSmtpPort(Number(e.target.value))} />
          </label>
          <label>
            {t("account.email")} / Username
            <input type="text" value={username} onChange={(e) => setUsername(e.target.value)} />
          </label>
          <label>
            {t("account.password")}
            <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} />
          </label>
          <label className="checkbox-label">
            <input type="checkbox" checked={useSsl} onChange={(e) => setUseSsl(e.target.checked)} />
            SSL/TLS
          </label>
          {error && <p className="error">{error}</p>}
          <div className="form-actions">
            <button onClick={onClose}>{t("account.cancel")}</button>
            <button onClick={handleSave}>{t("account.connect")}</button>
          </div>
        </div>
      </div>
    </div>
  );
}