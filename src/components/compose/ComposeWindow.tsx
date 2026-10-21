import { useState } from "react";
import { useTranslation } from "react-i18next";

export function ComposeWindow({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const [to, setTo] = useState("");
  const [cc, setCc] = useState("");
  const [subject, setSubject] = useState("");
  const [body, setBody] = useState("");

  return (
    <div className="compose-window">
      <div className="compose-header">
        <h3>{t("compose.title")}</h3>
        <button onClick={onClose}>✕</button>
      </div>
      <div className="compose-form">
        <input placeholder={t("compose.toPlaceholder")} value={to} onChange={(e) => setTo(e.target.value)} />
        <input placeholder={t("compose.ccPlaceholder")} value={cc} onChange={(e) => setCc(e.target.value)} />
        <input placeholder={t("compose.subjectPlaceholder")} value={subject} onChange={(e) => setSubject(e.target.value)} />
        <textarea placeholder={t("compose.bodyPlaceholder")} value={body} onChange={(e) => setBody(e.target.value)} rows={12} />
      </div>
      <div className="compose-actions">
        <button className="send-btn">{t("compose.send")}</button>
        <button className="draft-btn">{t("compose.saveDraft")}</button>
        <button className="discard-btn" onClick={onClose}>{t("compose.discard")}</button>
      </div>
    </div>
  );
}
