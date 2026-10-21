import { useEffect } from "react";
import { Sidebar } from "./components/common/Sidebar";
import { MailList } from "./components/mail/MailList";
import { MailReader } from "./components/mail/MailReader";
import { useMailStore } from "./stores/mailStore";
import "./i18n";

export default function App() {
  const { selectedFolder, selectedMail } = useMailStore();

  return (
    <div className="app-container">
      <Sidebar />
      <div className="main-content">
        <div className="mail-list-panel">
          <MailList />
        </div>
        <div className="mail-reader-panel">
          {selectedMail ? <MailReader /> : <EmptyState />}
        </div>
      </div>
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
