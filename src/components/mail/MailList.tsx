import { useState, useMemo, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useMailStore, MailMessage } from "../../stores/mailStore";
import { useSettingsStore } from "../../stores/settingsStore";

function cleanSubject(subject: string): string {
  return subject
    .replace(/^(Re|Fwd|Fw):\s*/i, "")
    .replace(/^(回复|转发):\s*/, "")
    .trim();
}

export function MailList() {
  const { t } = useTranslation();
  const {
    messages, selectedMail, setSelectedMail, selectedFolder,
    selectedAccountId, loading, searching, searchQuery, isUnifiedInbox,
    searchMessages, clearSearch, toggleFlagged,
    loadMessages, hasMoreMessages,
  } = useMailStore();
  const { listDensity, setListDensity } = useSettingsStore();
  const [searchInput, setSearchInput] = useState("");
  const [threadView, setThreadView] = useState(false);
  const listRef = useRef<HTMLDivElement>(null);

  const handleScroll = useCallback(() => {
    const el = listRef.current;
    if (!el || !hasMoreMessages || loading) return;
    if (el.scrollHeight - el.scrollTop - el.clientHeight < 100) {
      if (selectedAccountId && selectedFolder) {
        loadMessages(selectedAccountId, selectedFolder, 50, messages.length, true);
      }
    }
  }, [hasMoreMessages, loading, selectedAccountId, selectedFolder, messages.length, loadMessages]);

  const threadGroups = useMemo(() => {
    if (!threadView) return null;
    const groups = new Map<string, MailMessage[]>();
    for (const msg of messages) {
      const key = cleanSubject(msg.subject) || msg.subject || "(No Subject)";
      if (!groups.has(key)) {
        groups.set(key, []);
      }
      groups.get(key)!.push(msg);
    }
    return Array.from(groups.entries()).map(([subject, msgs]) => ({
      subject,
      messages: msgs,
      latest: msgs.reduce((a, b) =>
        (a.date && b.date && new Date(a.date) >= new Date(b.date)) ? a : b
      ),
      count: msgs.length,
    }));
  }, [messages, threadView]);

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

  const handleSearch = () => {
    if (!selectedAccountId) return;
    if (searchInput.trim()) {
      searchMessages(selectedAccountId, searchInput.trim());
    } else {
      clearSearch();
    }
  };

  const handleClearSearch = () => {
    setSearchInput("");
    clearSearch();
  };

  const handleToggleFlag = (e: React.MouseEvent, messageId: string, currentFlagged: boolean) => {
    e.stopPropagation();
    toggleFlagged(messageId, !currentFlagged);
  };

  if (loading) {
    return <div className="mail-list-loading">{t("app.loading")}</div>;
  }

  return (
    <div className={`mail-list ${listDensity === "compact" ? "compact" : ""}`}>
      <div className="mail-list-header">
        <h2>{searchQuery ? `${t("mail.searchResults")} "${searchQuery}"` : isUnifiedInbox ? t("sidebar.unifiedInbox") : selectedFolder || t("folder.inbox")}</h2>
      </div>

      <div className="mail-list-search">
        <input
          type="text"
          placeholder={t("mail.searchPlaceholder")}
          value={searchInput}
          onChange={(e) => setSearchInput(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSearch()}
          disabled={searching}
        />
        {searchInput && (
          <button className="search-clear-btn" onClick={handleClearSearch}>✕</button>
        )}
        <button className="search-btn" onClick={handleSearch} disabled={searching || !searchInput.trim()}>
          🔍
        </button>
      </div>

      <div className="mail-list-toolbar">
        <button
          className={`thread-toggle-btn ${threadView ? "active" : ""}`}
          onClick={() => setThreadView(!threadView)}
        >
          💬 {threadView ? t("mail.listView") : t("mail.threadView")}
        </button>
        <button
          className="density-toggle-btn"
          onClick={() => setListDensity(listDensity === "compact" ? "comfortable" : "compact")}
          title={listDensity === "compact" ? t("settings.comfortable") : t("settings.compact")}
        >
          {listDensity === "compact" ? "☰" : "≡"}
        </button>
      </div>

      <div className="mail-list-items" ref={listRef} onScroll={handleScroll}>
        {messages.length === 0 ? (
          <div className="mail-list-empty">
            <p>{searching ? t("app.loading") : (searchQuery ? t("mail.noSearchResults") : "没有邮件")}</p>
          </div>
        ) : threadView && threadGroups ? (
          threadGroups.map((group) => (
            <div
              key={group.subject}
              className={`mail-item thread-group ${!group.latest.is_read ? "unread" : ""} ${selectedMail?.id === group.latest.id ? "selected" : ""}`}
              onClick={() => setSelectedMail(group.latest)}
            >
              <div className="mail-item-star" onClick={(e) => handleToggleFlag(e, group.latest.id, group.latest.is_flagged)}>
                <span className={group.latest.is_flagged ? "star-icon active" : "star-icon"}>{group.latest.is_flagged ? "⭐" : "☆"}</span>
              </div>
              <div className="mail-item-content">
                <div className="mail-item-header">
                  <span className="mail-item-from">{formatSender(group.latest.from)}</span>
                  <span className="mail-item-date">{formatDate(group.latest.date)}</span>
                </div>
                <div className="mail-item-subject">
                  {group.subject || t("mail.noSubject")}
                  {group.count > 1 && <span className="thread-count">({group.count})</span>}
                </div>
                <div className="mail-item-preview">{group.latest.preview}</div>
              </div>
              {group.latest.has_attachments && <span className="attachment-icon">📎</span>}
            </div>
          ))
        ) : (
          messages.map((msg) => (
            <div
              key={msg.id}
              className={`mail-item ${!msg.is_read ? "unread" : ""} ${selectedMail?.id === msg.id ? "selected" : ""}`}
              onClick={() => setSelectedMail(msg)}
            >
              <div className="mail-item-star" onClick={(e) => handleToggleFlag(e, msg.id, msg.is_flagged)}>
                <span className={msg.is_flagged ? "star-icon active" : "star-icon"}>{msg.is_flagged ? "⭐" : "☆"}</span>
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
        {hasMoreMessages && (
          <div className="mail-list-load-more">{t("app.loading")}</div>
        )}
      </div>
    </div>
  );
}