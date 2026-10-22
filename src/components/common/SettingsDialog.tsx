import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useMailStore, MailRule, FolderMapping } from "../../stores/mailStore";
import { useSettingsStore } from "../../stores/settingsStore";

export function SettingsDialog({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const { accounts, listRules, saveRule, deleteRule, getFolderMapping, saveFolderMapping, folders } = useMailStore();
  const {
    locale, theme, signatures, syncIntervalMs, defaultHtmlMode,
    setLocale, setTheme, setSignature, removeSignature,
    setSyncInterval, setDefaultHtmlMode,
  } = useSettingsStore();
  const [selectedAccountId, setSelectedAccountId] = useState<string>(
    accounts.length > 0 ? accounts[0].id : ""
  );
  const [sigContent, setSigContent] = useState<string>(() => {
    if (accounts.length === 0) return "";
    const existing = signatures.find((s) => s.accountId === accounts[0].id);
    return existing?.content || "";
  });

  const handleAccountChange = (accountId: string) => {
    setSelectedAccountId(accountId);
    const existing = signatures.find((s) => s.accountId === accountId);
    setSigContent(existing?.content || "");
  };

  const handleSaveSignature = () => {
    if (!selectedAccountId) return;
    if (sigContent.trim()) {
      setSignature(selectedAccountId, sigContent.trim());
    } else {
      removeSignature(selectedAccountId);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog-content settings-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h3>{t("settings.title")}</h3>
          <button onClick={onClose}>✕</button>
        </div>
        <div className="settings-body">
          <section className="settings-section">
            <h4>{t("settings.appearance")}</h4>
            <div className="settings-row">
              <label>{t("settings.language")}</label>
              <select value={locale} onChange={(e) => setLocale(e.target.value)}>
                <option value="zh-CN">中文</option>
                <option value="en">English</option>
              </select>
            </div>
            <div className="settings-row">
              <label>{t("settings.theme")}</label>
              <select value={theme} onChange={(e) => setTheme(e.target.value as "light" | "dark" | "system")}>
                <option value="system">{t("settings.system")}</option>
                <option value="light">{t("settings.light")}</option>
                <option value="dark">{t("settings.dark")}</option>
              </select>
            </div>
          </section>

          <section className="settings-section">
            <h4>{t("settings.general")}</h4>
            <div className="settings-row">
              <label>{t("settings.syncInterval")}</label>
              <select
                value={syncIntervalMs}
                onChange={(e) => setSyncInterval(Number(e.target.value))}
              >
                <option value={30000}>30s</option>
                <option value={60000}>1min</option>
                <option value={120000}>2min</option>
                <option value={300000}>5min</option>
              </select>
            </div>
            <div className="settings-row">
              <label>{t("settings.defaultEditor")}</label>
              <select
                value={defaultHtmlMode ? "html" : "plain"}
                onChange={(e) => setDefaultHtmlMode(e.target.value === "html")}
              >
                <option value="plain">{t("compose.plainText")}</option>
                <option value="html">{t("compose.richText")}</option>
              </select>
            </div>
          </section>

          <section className="settings-section">
            <h4>{t("settings.signature")}</h4>
            {accounts.length > 0 ? (
              <>
                <div className="settings-row">
                  <label>{t("account.selectProvider")}</label>
                  <select
                    value={selectedAccountId}
                    onChange={(e) => handleAccountChange(e.target.value)}
                  >
                    {accounts.map((acc) => (
                      <option key={acc.id} value={acc.id}>
                        {acc.display_name || acc.email_address}
                      </option>
                    ))}
                  </select>
                </div>
                <textarea
                  className="signature-editor"
                  value={sigContent}
                  onChange={(e) => setSigContent(e.target.value)}
                  placeholder={t("settings.signaturePlaceholder")}
                  rows={5}
                />
                <button className="save-sig-btn" onClick={handleSaveSignature}>
                  {t("settings.saveSignature")}
                </button>
              </>
            ) : (
              <p className="settings-hint">{t("sidebar.addAccount")}</p>
            )}
          </section>

          <section className="settings-section">
            <h4>{t("settings.rules")}</h4>
            {accounts.length > 0 ? (
              <RulesManager
                accounts={accounts}
                folders={folders}
                listRules={listRules}
                saveRule={saveRule}
                deleteRule={deleteRule}
                t={t}
              />
            ) : (
              <p className="settings-hint">{t("sidebar.addAccount")}</p>
            )}
          </section>

          <section className="settings-section">
            <h4>{t("settings.folderMapping")}</h4>
            {accounts.length > 0 ? (
              <FolderMappingManager
                accounts={accounts}
                folders={folders}
                getFolderMapping={getFolderMapping}
                saveFolderMapping={saveFolderMapping}
                t={t}
              />
            ) : (
              <p className="settings-hint">{t("sidebar.addAccount")}</p>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}

function RulesManager({ accounts, folders, listRules, saveRule, deleteRule, t }: {
  accounts: { id: string; display_name: string; email_address: string }[];
  folders: { name: string; display_name: string }[];
  listRules: (accountId: string) => Promise<MailRule[]>;
  saveRule: (rule: MailRule) => Promise<void>;
  deleteRule: (ruleId: string) => Promise<void>;
  t: (key: string) => string;
}) {
  const [ruleAccountId, setRuleAccountId] = useState(accounts[0]?.id || "");
  const [rules, setRules] = useState<MailRule[]>([]);
  const [newField, setNewField] = useState<"subject" | "from" | "to">("subject");
  const [newOperator, setNewOperator] = useState<"contains" | "equals" | "startswith" | "endswith">("contains");
  const [newValue, setNewValue] = useState("");
  const [newAction, setNewAction] = useState<"move-to" | "mark-read" | "mark-flagged" | "delete">("mark-read");
  const [newActionValue, setNewActionValue] = useState("");

  useEffect(() => {
    if (ruleAccountId) listRules(ruleAccountId).then(setRules);
  }, [ruleAccountId]);

  const handleAddRule = async () => {
    if (!newValue.trim()) return;
    const rule: MailRule = {
      id: crypto.randomUUID(),
      account_id: ruleAccountId,
      field: newField,
      operator: newOperator,
      value: newValue.trim(),
      action: newAction,
      action_value: newActionValue,
      enabled: true,
    };
    await saveRule(rule);
    setRules(await listRules(ruleAccountId));
    setNewValue("");
    setNewActionValue("");
  };

  return (
    <div className="rules-manager">
      <select value={ruleAccountId} onChange={(e) => setRuleAccountId(e.target.value)}>
        {accounts.map((a) => <option key={a.id} value={a.id}>{a.display_name || a.email_address}</option>)}
      </select>
      <div className="rule-list">
        {rules.map((r) => (
          <div key={r.id} className="rule-item">
            <span>{t(`settings.ruleField.${r.field}`)} {t(`settings.ruleOp.${r.operator}`)} "{r.value}" → {t(`settings.ruleAction.${r.action}`)}{r.action === "move-to" ? ` ${r.action_value}` : ""}</span>
            <button onClick={async () => { await deleteRule(r.id); setRules(await listRules(ruleAccountId)); }}>✕</button>
          </div>
        ))}
      </div>
      <div className="rule-add-form">
        <select value={newField} onChange={(e) => setNewField(e.target.value as typeof newField)}>
          <option value="subject">{t("settings.ruleField.subject")}</option>
          <option value="from">{t("settings.ruleField.from")}</option>
          <option value="to">{t("settings.ruleField.to")}</option>
        </select>
        <select value={newOperator} onChange={(e) => setNewOperator(e.target.value as typeof newOperator)}>
          <option value="contains">{t("settings.ruleOp.contains")}</option>
          <option value="equals">{t("settings.ruleOp.equals")}</option>
          <option value="startswith">{t("settings.ruleOp.startswith")}</option>
          <option value="endswith">{t("settings.ruleOp.endswith")}</option>
        </select>
        <input value={newValue} onChange={(e) => setNewValue(e.target.value)} placeholder={t("settings.ruleValuePlaceholder")} />
        <select value={newAction} onChange={(e) => setNewAction(e.target.value as typeof newAction)}>
          <option value="mark-read">{t("settings.ruleAction.mark-read")}</option>
          <option value="mark-flagged">{t("settings.ruleAction.mark-flagged")}</option>
          <option value="move-to">{t("settings.ruleAction.move-to")}</option>
          <option value="delete">{t("settings.ruleAction.delete")}</option>
        </select>
        {newAction === "move-to" && (
          <select value={newActionValue} onChange={(e) => setNewActionValue(e.target.value)}>
            <option value="">--</option>
            {folders.map((f) => <option key={f.name} value={f.name}>{f.display_name}</option>)}
          </select>
        )}
        <button onClick={handleAddRule} disabled={!newValue.trim()}>+</button>
      </div>
    </div>
  );
}

function FolderMappingManager({ accounts, folders, getFolderMapping, saveFolderMapping, t }: {
  accounts: { id: string; display_name: string; email_address: string }[];
  folders: { name: string; display_name: string }[];
  getFolderMapping: (accountId: string) => Promise<FolderMapping>;
  saveFolderMapping: (accountId: string, mapping: FolderMapping) => Promise<void>;
  t: (key: string) => string;
}) {
  const [mappingAccountId, setMappingAccountId] = useState(accounts[0]?.id || "");
  const [mapping, setMapping] = useState<FolderMapping>({ sent: null, drafts: null, trash: null, junk: null });

  useEffect(() => {
    if (mappingAccountId) getFolderMapping(mappingAccountId).then(setMapping);
  }, [mappingAccountId]);

  const handleSave = () => {
    saveFolderMapping(mappingAccountId, mapping);
  };

  const accountFolders = folders;

  return (
    <div className="folder-mapping-manager">
      <select value={mappingAccountId} onChange={(e) => setMappingAccountId(e.target.value)}>
        {accounts.map((a) => <option key={a.id} value={a.id}>{a.display_name || a.email_address}</option>)}
      </select>
      {(["sent", "drafts", "trash", "junk"] as const).map((key) => (
        <div key={key} className="settings-row">
          <label>{t(`settings.folderMap.${key}`)}</label>
          <select
            value={mapping[key] || ""}
            onChange={(e) => setMapping((m) => ({ ...m, [key]: e.target.value || null }))}
          >
            <option value="">({t("settings.auto")})</option>
            {accountFolders.map((f) => <option key={f.name} value={f.name}>{f.display_name}</option>)}
          </select>
        </div>
      ))}
      <button className="save-sig-btn" onClick={handleSave}>{t("settings.saveSignature")}</button>
    </div>
  );
}