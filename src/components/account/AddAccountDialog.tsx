import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

interface ProviderPreset {
  name: string;
  account_type: string;
  imap_host: string;
  imap_port: number;
  smtp_host: string;
  smtp_port: number;
}

export function AddAccountDialog({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const [step, setStep] = useState<"provider" | "credentials">("provider");
  const [selectedProvider, setSelectedProvider] = useState<string>("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [presets, setPresets] = useState<ProviderPreset[]>([]);

  const loadPresets = async () => {
    try {
      const result = await invoke<ProviderPreset[]>("get_provider_presets");
      setPresets(result);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleConnect = async () => {
    setLoading(true);
    setError(null);
    try {
      const cmd = `add_${selectedProvider}_account`;
      const account = await invoke(cmd, {
        email,
        password,
        displayName: displayName || email,
      });
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <h2>{t("account.add")}</h2>

        {step === "provider" && (
          <div className="provider-list">
            <p>{t("account.selectProvider")}</p>
            {presets.map((p) => (
              <button
                key={p.account_type}
                className={`provider-btn ${selectedProvider === p.account_type ? "active" : ""}`}
                onClick={() => { setSelectedProvider(p.account_type); setStep("credentials"); }}
              >
                {p.name}
              </button>
            ))}
            <button className="provider-btn custom" onClick={() => { setSelectedProvider("custom"); setStep("credentials"); }}>
              {t("account.custom")}
            </button>
          </div>
        )}

        {step === "credentials" && (
          <div className="credentials-form">
            {selectedProvider === "qq" && (
              <p className="hint">{t("account.qqAuthCodeHint")}</p>
            )}
            <label>
              {t("account.email")}
              <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} placeholder="example@qq.com" />
            </label>
            <label>
              {t("account.password")}
              <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} />
            </label>
            <label>
              {t("account.displayName")}
              <input type="text" value={displayName} onChange={(e) => setDisplayName(e.target.value)} />
            </label>
            {error && <p className="error">{error}</p>}
            <div className="form-actions">
              <button onClick={() => setStep("provider")}>← {t("account.cancel")}</button>
              <button onClick={handleConnect} disabled={loading || !email || !password}>
                {loading ? t("app.loading") : t("account.connect")}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
