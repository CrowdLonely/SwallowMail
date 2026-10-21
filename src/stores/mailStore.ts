import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

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

export interface MailMessage {
  id: string;
  folder: string;
  uid: number;
  subject: string;
  from: { name: string | null; address: string }[];
  to: { name: string | null; address: string }[];
  cc: { name: string | null; address: string }[];
  date: string | null;
  is_read: boolean;
  is_flagged: boolean;
  has_attachments: boolean;
  preview: string;
  body_text: string | null;
  body_html: string | null;
}

interface MailState {
  accounts: MailAccount[];
  folders: MailFolder[];
  messages: MailMessage[];
  selectedAccountId: string | null;
  selectedFolder: string | null;
  selectedMail: MailMessage | null;
  loading: boolean;
  error: string | null;

  setSelectedAccount: (id: string) => void;
  setSelectedFolder: (name: string) => void;
  setSelectedMail: (mail: MailMessage | null) => void;
  addAccount: (account: MailAccount) => void;
  loadFolders: (accountId: string) => Promise<void>;
  loadMessages: (accountId: string, folder: string) => Promise<void>;
}

export const useMailStore = create<MailState>((set, get) => ({
  accounts: [],
  folders: [],
  messages: [],
  selectedAccountId: null,
  selectedFolder: null,
  selectedMail: null,
  loading: false,
  error: null,

  setSelectedAccount: (id) => set({ selectedAccountId: id, selectedFolder: null, selectedMail: null }),
  setSelectedFolder: (name) => set({ selectedFolder: name, selectedMail: null }),
  setSelectedMail: (mail) => set({ selectedMail: mail }),

  addAccount: (account) => set((state) => ({ accounts: [...state.accounts, account] })),

  loadFolders: async (accountId) => {
    set({ loading: true, error: null });
    try {
      set({ loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },

  loadMessages: async (accountId, folder) => {
    set({ loading: true, error: null });
    try {
      set({ loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },
}));
