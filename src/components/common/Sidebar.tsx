import { useTranslation } from "react-i18next";
import { useMailStore } from "../../stores/mailStore";
import { useSettingsStore } from "../../stores/settingsStore";

export function Sidebar({ onCompose, onSettings, onEditAccount }: { onCompose: () => void; onSettings: () => void; onEditAccount: (accountId: string) => void }) {
  const { t } = useTranslation();
  const {
    accounts, folders, selectedAccountId, selectedFolder, isUnifiedInbox,
    setSelectedAccount, setSelectedFolder, setUnifiedInbox, loadUnifiedInbox,
    loadFolders, loadMessages, syncFolder,
    syncing,
  } = useMailStore();
  const { sidebarWidth } = useSettingsStore();

  const handleSelectAccount = async (id: string) => {
    setSelectedAccount(id);
    await loadFolders(id);
  };

  const handleSelectFolder = async (name: string) => {
    setSelectedFolder(name);
    if (selectedAccountId) {
      const account = accounts.find((a) => a.id === selectedAccountId);
      if (account) {
        try {
          await syncFolder(account, name);
        } catch {
          // 同步失败时仍加载本地缓存
        }
        await loadMessages(selectedAccountId, name);
      }
    }
  };

  const handleUnifiedInbox = async () => {
    setUnifiedInbox(true);
    await loadUnifiedInbox();
  };

  const totalUnread = accounts.reduce((sum, acc) => {
    return sum;
  }, 0);

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

      <button className="compose-btn" onClick={onCompose}>
        ✉️ {t("sidebar.compose")}
      </button>

      {accounts.length > 1 && (
        <div
          className={`unified-inbox-item ${isUnifiedInbox ? "active" : ""}`}
          onClick={handleUnifiedInbox}
        >
          <span className="folder-icon">📬</span>
          <span className="folder-name">{t("sidebar.unifiedInbox")}</span>
        </div>
      )}

      <div className="account-list">
        {accounts.map((account) => (
          <div key={account.id} className="account-section">
            <div
              className={`account-header ${selectedAccountId === account.id ? "active" : ""}`}
              onClick={() => handleSelectAccount(account.id)}
            >
              <span className="account-icon">📧</span>
              <span className="account-name">{account.display_name || account.email_address}</span>
              <button className="account-edit-btn" onClick={(e) => { e.stopPropagation(); onEditAccount(account.id); }} title={t("account.edit")}>✏️</button>
            </div>
            {selectedAccountId === account.id && (
              <div className="folder-list">
                {folders.map((folder) => (
                  <div
                    key={folder.name}
                    className={`folder-item ${selectedFolder === folder.name ? "active" : ""}`}
                    onClick={() => handleSelectFolder(folder.name)}
                  >
                    <span className="folder-icon">{folderIcon(folder.special_use)}</span>
                    <span className="folder-name">{folder.display_name}</span>
                    {folder.unread_count > 0 && (
                      <span className="unread-badge">{folder.unread_count}</span>
                    )}
                    {syncing && selectedFolder === folder.name && (
                      <span className="sync-indicator">↻</span>
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
        <button className="settings-btn" onClick={onSettings}>⚙️ {t("sidebar.settings")}</button>
      </div>
    </aside>
  );
}