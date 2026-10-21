import { useTranslation } from "react-i18next";
import { useMailStore } from "../../stores/mailStore";
import { useSettingsStore } from "../../stores/settingsStore";

export function Sidebar() {
  const { t } = useTranslation();
  const { accounts, folders, selectedAccountId, selectedFolder, setSelectedAccount, setSelectedFolder } = useMailStore();
  const { sidebarWidth } = useSettingsStore();

  const folderIcon = (specialUse: string | null): string => {
    switch (specialUse) {
      case "Inbox": return "📥";
      case "Sent": return "📤";
      case "Drafts": return "📝";
      case "Trash": return "🗑️";
      case "Junk": return "⚠️";
      case "Archive": return "📦";
      case "Flagged": return "⭐";
      case "Important": return "🔴";
      default: return "📁";
    }
  };

  return (
    <aside className="sidebar" style={{ width: sidebarWidth }}>
      <div className="sidebar-header">
        <h1 className="app-title">🕊️ SwallowMail</h1>
      </div>

      <button className="compose-btn">
        ✉️ {t("sidebar.compose")}
      </button>

      <div className="account-list">
        {accounts.map((account) => (
          <div key={account.id} className="account-section">
            <div
              className={`account-header ${selectedAccountId === account.id ? "active" : ""}`}
              onClick={() => setSelectedAccount(account.id)}
            >
              <span className="account-icon">📧</span>
              <span className="account-name">{account.display_name || account.email_address}</span>
            </div>
            {selectedAccountId === account.id && (
              <div className="folder-list">
                {folders.map((folder) => (
                  <div
                    key={folder.name}
                    className={`folder-item ${selectedFolder === folder.name ? "active" : ""}`}
                    onClick={() => setSelectedFolder(folder.name)}
                  >
                    <span className="folder-icon">{folderIcon(folder.special_use)}</span>
                    <span className="folder-name">{folder.display_name}</span>
                    {folder.unread_count > 0 && (
                      <span className="unread-badge">{folder.unread_count}</span>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>

      {accounts.length === 0 && (
        <div className="sidebar-empty">
          <p>{t("sidebar.addAccount")}</p>
        </div>
      )}

      <div className="sidebar-footer">
        <button className="settings-btn">⚙️ {t("sidebar.settings")}</button>
      </div>
    </aside>
  );
}
