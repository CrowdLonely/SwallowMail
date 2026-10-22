import { useEffect, useState } from "react";
import { Sidebar } from "./components/common/Sidebar";
import { MailList } from "./components/mail/MailList";
import { MailReader } from "./components/mail/MailReader";
import { ComposeWindow, ComposeInitialData } from "./components/compose/ComposeWindow";
import { ToastContainer } from "./components/common/ToastContainer";
import { SettingsDialog } from "./components/common/SettingsDialog";
import { EditAccountDialog } from "./components/account/EditAccountDialog";
import { useMailStore } from "./stores/mailStore";
import "./i18n";

export default function App() {
  const {
    selectedMail, storeInitialized, initStore, loadAccounts,
    startAutoSync, stopAutoSync, toggleRead, deleteMessage,
    messages, setSelectedMail,
  } = useMailStore();
  const [showCompose, setShowCompose] = useState(false);
  const [composeInitial, setComposeInitial] = useState<ComposeInitialData | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [editAccountId, setEditAccountId] = useState<string | null>(null);
  const [isOffline, setIsOffline] = useState(!navigator.onLine);

  useEffect(() => {
    const onOffline = () => setIsOffline(true);
    const onOnline = () => setIsOffline(false);
    window.addEventListener("offline", onOffline);
    window.addEventListener("online", onOnline);
    return () => {
      window.removeEventListener("offline", onOffline);
      window.removeEventListener("online", onOnline);
    };
  }, []);

  useEffect(() => {
    const bootstrap = async () => {
      if (!storeInitialized) {
        await initStore();
      }
      await loadAccounts();
      startAutoSync();
    };
    bootstrap();
    return () => {
      stopAutoSync();
    };
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if ((e.target as HTMLElement).isContentEditable) return;

      if (e.key === "Delete" || e.key === "Backspace") {
        if (selectedMail) {
          e.preventDefault();
          deleteMessage(selectedMail.id);
        }
      } else if (e.key === "r" && !e.ctrlKey && !e.metaKey) {
        if (selectedMail) {
          e.preventDefault();
          handleCompose({
            to: selectedMail.from.map((a) => a.name ? `${a.name} <${a.address}>` : a.address).join(", "),
            cc: "",
            subject: selectedMail.subject.startsWith("Re:") ? selectedMail.subject : `Re: ${selectedMail.subject}`,
            body: `\n\n--- 原始邮件 ---\n${selectedMail.body_text || selectedMail.preview || ""}`,
          });
        }
      } else if (e.key === "m") {
        if (selectedMail) {
          e.preventDefault();
          toggleRead(selectedMail.id, !selectedMail.is_read);
        }
      } else if (e.key === "j") {
        e.preventDefault();
        const idx = messages.findIndex((m) => m.id === selectedMail?.id);
        if (idx < messages.length - 1) setSelectedMail(messages[idx + 1]);
      } else if (e.key === "k") {
        e.preventDefault();
        const idx = messages.findIndex((m) => m.id === selectedMail?.id);
        if (idx > 0) setSelectedMail(messages[idx - 1]);
      } else if (e.key === "n" && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        handleCompose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [selectedMail, messages]);

  const handleCompose = (initialData?: ComposeInitialData | null) => {
    setComposeInitial(initialData || null);
    setShowCompose(true);
  };

  const handleCloseCompose = () => {
    setShowCompose(false);
    setComposeInitial(null);
  };

  return (
    <div className="app-container">
      {isOffline && <div className="offline-banner">⚠️ {t("app.offline")}</div>}
      <Sidebar onCompose={() => handleCompose()} onSettings={() => setShowSettings(true)} onEditAccount={(id) => setEditAccountId(id)} />
      <div className="main-content">
        <div className="mail-list-panel">
          <MailList />
        </div>
        <div className="mail-reader-panel">
          {selectedMail ? <MailReader onCompose={handleCompose} /> : <EmptyState />}
        </div>
      </div>
      {showCompose && <ComposeWindow onClose={handleCloseCompose} initialData={composeInitial} />}
      {showSettings && <SettingsDialog onClose={() => setShowSettings(false)} />}
      {editAccountId && <EditAccountDialog accountId={editAccountId} onClose={() => setEditAccountId(null)} />}
      <ToastContainer />
    </div>
  );
}

function EmptyState() {
  return (
    <div className="empty-state">
      <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
        <path d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
      </svg>
      <p>选择一封邮件开始阅读</p>
    </div>
  );
}