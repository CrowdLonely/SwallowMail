import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface Toast {
  id: string;
  message: string;
  type: "info" | "success" | "error";
  createdAt: number;
}

export interface MailAddress {
  name: string | null;
  address: string;
}

export interface MailAccount {
  id: string;
  display_name: string;
  email_address: string;
  account_type: string;
  imap_host: string;
  imap_port: number;
  smtp_host: string;
  smtp_port: number;
  username: string;
  use_ssl: boolean;
}

export interface MailFolder {
  name: string;
  display_name: string;
  delimiter: string;
  attributes: string[];
  unread_count: number;
  total_count: number;
  special_use: string | null;
}

export interface Attachment {
  filename: string;
  content_type: string;
  size: number;
}

export interface MailMessage {
  id: string;
  folder: string;
  uid: number;
  subject: string;
  from: MailAddress[];
  to: MailAddress[];
  cc: MailAddress[];
  date: string | null;
  is_read: boolean;
  is_flagged: boolean;
  has_attachments: boolean;
  preview: string;
  body_text: string | null;
  body_html: string | null;
  attachments: Attachment[];
}

export interface ComposeAttachment {
  filename: string;
  content_type: string;
  data: number[];
}

export interface MailRule {
  id: string;
  account_id: string;
  field: "subject" | "from" | "to";
  operator: "contains" | "equals" | "startswith" | "endswith";
  value: string;
  action: "move-to" | "mark-read" | "mark-flagged" | "delete";
  action_value: string;
  enabled: boolean;
}

export interface FolderMapping {
  sent: string | null;
  drafts: string | null;
  trash: string | null;
  junk: string | null;
}

export interface SyncResult {
  folder: string;
  total_fetched: number;
  new_saved: number;
}

interface MailState {
  accounts: MailAccount[];
  folders: MailFolder[];
  messages: MailMessage[];
  selectedAccountId: string | null;
  selectedFolder: string | null;
  selectedMail: MailMessage | null;
  isUnifiedInbox: boolean;
  loading: boolean;
  syncing: boolean;
  sending: boolean;
  searching: boolean;
  searchQuery: string;
  threadMessages: MailMessage[];
  threadLoading: boolean;
  error: string | null;
  storeInitialized: boolean;
  autoSyncTimer: ReturnType<typeof setInterval> | null;
  toasts: Toast[];
  hasMoreMessages: boolean;

  initStore: () => Promise<void>;
  setSelectedAccount: (id: string) => void;
  setSelectedFolder: (name: string) => void;
  setSelectedMail: (mail: MailMessage | null) => void;
  setUnifiedInbox: (enabled: boolean) => void;
  addAccount: (account: MailAccount) => Promise<void>;
  loadAccounts: () => Promise<void>;
  deleteAccount: (accountId: string) => Promise<void>;
  connectAndLoadFolders: (account: MailAccount) => Promise<MailFolder[]>;
  loadFolders: (accountId: string) => Promise<void>;
  syncFolder: (account: MailAccount, folderName: string) => Promise<SyncResult>;
  loadMessages: (accountId: string, folder: string, limit?: number, offset?: number, append?: boolean) => Promise<void>;
  loadUnifiedInbox: () => Promise<void>;
  fetchBody: (account: MailAccount, messageId: string, folderName: string, uid: number) => Promise<MailMessage>;
  sendMail: (account: MailAccount, from: MailAddress, to: MailAddress[], cc: MailAddress[], subject: string, bodyText: string, bodyHtml: string | null, attachments?: ComposeAttachment[]) => Promise<void>;
  toggleRead: (messageId: string, isRead: boolean) => Promise<void>;
  toggleFlagged: (messageId: string, isFlagged: boolean) => Promise<void>;
  deleteMessage: (messageId: string) => Promise<void>;
  searchMessages: (accountId: string, query: string) => Promise<void>;
  clearSearch: () => void;
  loadThread: (accountId: string, subject: string) => Promise<void>;
  clearThread: () => void;
  saveAttachment: (account: MailAccount, folderName: string, uid: number, filename: string) => Promise<void>;
  saveDraft: (accountId: string, subject: string, sender: string, recipients: string, bodyText: string | null, bodyHtml: string | null) => Promise<string>;
  updateAccount: (account: MailAccount) => Promise<void>;
  startAutoSync: (intervalMs?: number) => void;
  stopAutoSync: () => void;
  refreshUnreadCounts: (accountId: string) => Promise<void>;
  showToast: (message: string, type?: "info" | "success" | "error") => void;
  dismissToast: (id: string) => void;
  moveMessage: (messageId: string, destFolder: string) => Promise<void>;
  saveRule: (rule: MailRule) => Promise<void>;
  listRules: (accountId: string) => Promise<MailRule[]>;
  deleteRule: (ruleId: string) => Promise<void>;
  saveFolderMapping: (accountId: string, mapping: FolderMapping) => Promise<void>;
  getFolderMapping: (accountId: string) => Promise<FolderMapping>;
}

export const useMailStore = create<MailState>((set, get) => ({
  accounts: [],
  folders: [],
  messages: [],
  selectedAccountId: null,
  selectedFolder: null,
  selectedMail: null,
  isUnifiedInbox: false,
  loading: false,
  syncing: false,
  sending: false,
  searching: false,
  searchQuery: "",
  threadMessages: [],
  threadLoading: false,
  error: null,
  storeInitialized: false,
  autoSyncTimer: null,
  toasts: [],
  hasMoreMessages: false,

  initStore: async () => {
    try {
      await invoke("init_store");
      set({ storeInitialized: true });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setSelectedAccount: (id) => set({ selectedAccountId: id, selectedFolder: null, selectedMail: null, folders: [], messages: [], searchQuery: "", isUnifiedInbox: false }),
  setSelectedFolder: (name) => set({ selectedFolder: name, selectedMail: null, messages: [], searchQuery: "", isUnifiedInbox: false }),
  setSelectedMail: (mail) => set({ selectedMail: mail }),
  setUnifiedInbox: (enabled) => set({ isUnifiedInbox: enabled, selectedAccountId: null, selectedFolder: null, selectedMail: null, messages: [], searchQuery: "" }),

  loadUnifiedInbox: async () => {
    set({ loading: true, error: null });
    try {
      const allMessages: MailMessage[] = [];
      for (const account of get().accounts) {
        const msgs = await invoke<MailMessage[]>("list_all_inbox_messages", { accountId: account.id, limit: 50 });
        allMessages.push(...msgs);
      }
      allMessages.sort((a, b) => {
        const da = a.date ? new Date(a.date).getTime() : 0;
        const db = b.date ? new Date(b.date).getTime() : 0;
        return db - da;
      });
      set({ messages: allMessages, loading: false, hasMoreMessages: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },

  addAccount: async (account) => {
    try {
      await invoke("save_account_to_db", { account });
      set((state) => ({ accounts: [...state.accounts, account] }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  loadAccounts: async () => {
    set({ loading: true, error: null });
    try {
      const accounts = await invoke<MailAccount[]>("list_accounts_from_db");
      set({ accounts, loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },

  deleteAccount: async (accountId) => {
    try {
      await invoke("delete_account_from_db", { accountId });
      set((state) => ({
        accounts: state.accounts.filter((a) => a.id !== accountId),
        selectedAccountId: state.selectedAccountId === accountId ? null : state.selectedAccountId,
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  connectAndLoadFolders: async (account) => {
    set({ loading: true, error: null });
    try {
      const folders = await invoke<MailFolder[]>("connect_and_fetch_folders", { account });
      await invoke("save_folders_to_db", { accountId: account.id, folders });
      set({ folders, loading: false });
      return folders;
    } catch (e) {
      set({ loading: false, error: String(e) });
      throw e;
    }
  },

  loadFolders: async (accountId) => {
    set({ loading: true, error: null });
    try {
      const folders = await invoke<MailFolder[]>("list_folders_from_db", { accountId });
      set({ folders, loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },

  syncFolder: async (account, folderName) => {
    set({ syncing: true, error: null });
    try {
      const result = await invoke<SyncResult>("sync_folder_messages", { account, folderName });
      set({ syncing: false });
      await get().refreshUnreadCounts(account.id);
      if (result.new_saved > 0) {
        get().showToast(`${folderName}: ${result.new_saved} 封新邮件`, "success");
        try {
          const { sendNotification, isPermissionGranted, requestPermission } = await import("@tauri-apps/plugin-notification");
          let permitted = await isPermissionGranted();
          if (!permitted) {
            const permission = await requestPermission();
            permitted = permission === "granted";
          }
          if (permitted) {
            sendNotification({
              title: "SwallowMail",
              body: `${folderName}: ${result.new_saved} 封新邮件`,
            });
          }
        } catch {}
      }
      return result;
    } catch (e) {
      set({ syncing: false, error: String(e) });
      get().showToast(`同步失败: ${String(e)}`, "error");
      throw e;
    }
  },

  loadMessages: async (accountId, folder, limit = 50, offset = 0, append = false) => {
    if (!append) set({ loading: true, error: null });
    try {
      const messages = await invoke<MailMessage[]>("list_messages_from_db", { accountId, folderName: folder, limit, offset });
      set((state) => ({
        messages: append ? [...state.messages, ...messages] : messages,
        hasMoreMessages: messages.length === limit,
        loading: false,
      }));
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },

  fetchBody: async (account, messageId, folderName, uid) => {
    const msg = await invoke<MailMessage>("fetch_message_body", { account, messageId, folderName, uid });
    set((state) => ({
      selectedMail: state.selectedMail?.id === messageId ? msg : state.selectedMail,
      messages: state.messages.map((m) => m.id === messageId ? msg : m),
    }));
    return msg;
  },

  sendMail: async (account, from, to, cc, subject, bodyText, bodyHtml, attachments) => {
    set({ sending: true, error: null });
    try {
      await invoke("send_mail", { account, from, to, cc, subject, bodyText, bodyHtml, attachments: attachments || [] });
      set({ sending: false });
      get().showToast("邮件已发送", "success");
    } catch (e) {
      set({ sending: false, error: String(e) });
      get().showToast(`发送失败: ${String(e)}`, "error");
      throw e;
    }
  },

  toggleRead: async (messageId, isRead) => {
    const state = get();
    const account = state.accounts.find((a) => a.id === state.selectedAccountId);
    if (account) {
      await invoke("toggle_message_read", { account, messageId, isRead });
      if (state.selectedAccountId) {
        get().refreshUnreadCounts(state.selectedAccountId);
      }
    }
    set((s) => ({
      messages: s.messages.map((m) => m.id === messageId ? { ...m, is_read: isRead } : m),
      selectedMail: s.selectedMail?.id === messageId ? { ...s.selectedMail, is_read: isRead } : s.selectedMail,
    }));
  },

  toggleFlagged: async (messageId, isFlagged) => {
    const state = get();
    const account = state.accounts.find((a) => a.id === state.selectedAccountId);
    if (account) {
      await invoke("toggle_message_flagged", { account, messageId, isFlagged });
    }
    set((s) => ({
      messages: s.messages.map((m) => m.id === messageId ? { ...m, is_flagged: isFlagged } : m),
      selectedMail: s.selectedMail?.id === messageId ? { ...s.selectedMail, is_flagged: isFlagged } : s.selectedMail,
    }));
  },

  deleteMessage: async (messageId) => {
    const state = get();
    const account = state.accounts.find((a) => a.id === state.selectedAccountId);
    if (account) {
      await invoke("delete_message", { account, messageId });
      if (state.selectedAccountId) {
        get().refreshUnreadCounts(state.selectedAccountId);
      }
    }
    set((s) => ({
      messages: s.messages.filter((m) => m.id !== messageId),
      selectedMail: s.selectedMail?.id === messageId ? null : s.selectedMail,
    }));
  },

  searchMessages: async (accountId, query) => {
    if (!query.trim()) {
      get().clearSearch();
      return;
    }
    set({ searching: true, searchQuery: query, error: null });
    try {
      const messages = await invoke<MailMessage[]>("search_messages", { accountId, query });
      set({ messages, searching: false });
    } catch (e) {
      set({ searching: false, error: String(e) });
    }
  },

  clearSearch: () => set({ searchQuery: "", searching: false }),

  loadThread: async (accountId, subject) => {
    set({ threadLoading: true, error: null });
    try {
      const threadMessages = await invoke<MailMessage[]>("get_thread_messages", { accountId, subject });
      set({ threadMessages, threadLoading: false });
    } catch (e) {
      set({ threadLoading: false, error: String(e) });
    }
  },

  clearThread: () => set({ threadMessages: [], threadLoading: false }),

  saveAttachment: async (account, folderName, uid, filename) => {
    try {
      await invoke("save_attachment", { account, folderName, uid, filename });
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  saveDraft: async (accountId, subject, sender, recipients, bodyText, bodyHtml) => {
    try {
      const id = await invoke<string>("save_draft", { accountId, subject, sender, recipients, bodyText, bodyHtml });
      return id;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  updateAccount: async (account) => {
    try {
      await invoke("update_account_in_db", { account });
      set((state) => ({
        accounts: state.accounts.map((a) => a.id === account.id ? account : a),
      }));
      get().showToast("账户已更新", "success");
    } catch (e) {
      set({ error: String(e) });
      get().showToast(`更新失败: ${String(e)}`, "error");
    }
  },

  startAutoSync: (intervalMs = 60000) => {
    const state = get();
    if (state.autoSyncTimer) return;

    const timer = setInterval(async () => {
      const s = get();
      if (s.syncing) return;

      await Promise.all(s.accounts.map(async (account) => {
        try {
          const folders = await invoke<MailFolder[]>("list_folders_from_db", { accountId: account.id });
          const inbox = folders.find((f) => f.special_use === "Inbox");
          if (inbox) {
            await get().syncFolder(account, inbox.name);
          }
          await get().refreshUnreadCounts(account.id);
        } catch {}
      }));

      const s2 = get();
      if (s2.selectedAccountId && s2.selectedFolder) {
        const acc = s2.accounts.find((a) => a.id === s2.selectedAccountId);
        if (acc) await get().loadMessages(acc.id, s2.selectedFolder);
      }
    }, intervalMs);

    set({ autoSyncTimer: timer });
  },

  stopAutoSync: () => {
    const timer = get().autoSyncTimer;
    if (timer) {
      clearInterval(timer);
      set({ autoSyncTimer: null });
    }
  },

  refreshUnreadCounts: async (accountId) => {
    try {
      const counts = await invoke<[string, number][]>("refresh_unread_counts", { accountId });
      set((state) => ({
        folders: state.folders.map((f) => {
          const entry = counts.find(([name]) => name === f.name);
          return entry ? { ...f, unread_count: entry[1] } : f;
        }),
      }));
    } catch {}
  },

  showToast: (message, type = "info") => {
    const id = Date.now().toString(36) + Math.random().toString(36).slice(2);
    const toast: Toast = { id, message, type, createdAt: Date.now() };
    set((state) => ({ toasts: [...state.toasts, toast] }));
    setTimeout(() => get().dismissToast(id), 4000);
  },

  dismissToast: (id) => {
    set((state) => ({ toasts: state.toasts.filter((t) => t.id !== id) }));
  },

  moveMessage: async (messageId, destFolder) => {
    const state = get();
    const account = state.accounts.find((a) => a.id === state.selectedAccountId);
    if (!account) return;
    try {
      await invoke("move_message", { account, messageId, destFolder });
      set((s) => ({
        messages: s.messages.filter((m) => m.id !== messageId),
        selectedMail: s.selectedMail?.id === messageId ? null : s.selectedMail,
      }));
      await get().refreshUnreadCounts(account.id);
      get().showToast(`邮件已移动到 ${destFolder}`, "success");
    } catch (e) {
      get().showToast(`移动失败: ${String(e)}`, "error");
    }
  },

  saveRule: async (rule) => {
    try {
      await invoke("save_rule", { rule });
    } catch (e) {
      get().showToast(`保存规则失败: ${String(e)}`, "error");
    }
  },

  listRules: async (accountId) => {
    try {
      return await invoke<MailRule[]>("list_rules", { accountId });
    } catch (e) {
      get().showToast(`获取规则失败: ${String(e)}`, "error");
      return [];
    }
  },

  deleteRule: async (ruleId) => {
    try {
      await invoke("delete_rule", { ruleId });
    } catch (e) {
      get().showToast(`删除规则失败: ${String(e)}`, "error");
    }
  },

  saveFolderMapping: async (accountId, mapping) => {
    try {
      await invoke("save_folder_mapping", { accountId, mapping });
      get().showToast("文件夹映射已保存", "success");
    } catch (e) {
      get().showToast(`保存映射失败: ${String(e)}`, "error");
    }
  },

  getFolderMapping: async (accountId) => {
    try {
      return await invoke<FolderMapping>("get_folder_mapping", { accountId });
    } catch {
      return { sent: null, drafts: null, trash: null, junk: null };
    }
  },
}));