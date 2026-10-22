import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface Signature {
  accountId: string;
  content: string;
}

interface SettingsState {
  locale: string;
  theme: "light" | "dark" | "system";
  sidebarWidth: number;
  mailListWidth: number;
  showPreview: boolean;
  signatures: Signature[];
  syncIntervalMs: number;
  defaultHtmlMode: boolean;
  listDensity: "compact" | "comfortable";
  setLocale: (locale: string) => void;
  setTheme: (theme: "light" | "dark" | "system") => void;
  setSidebarWidth: (width: number) => void;
  setMailListWidth: (width: number) => void;
  togglePreview: () => void;
  setSignature: (accountId: string, content: string) => void;
  removeSignature: (accountId: string) => void;
  getSignature: (accountId: string) => string;
  setSyncInterval: (ms: number) => void;
  setDefaultHtmlMode: (enabled: boolean) => void;
  setListDensity: (density: "compact" | "comfortable") => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set, get) => ({
      locale: "zh-CN",
      theme: "system",
      sidebarWidth: 220,
      mailListWidth: 360,
      showPreview: true,
      signatures: [],
      syncIntervalMs: 60000,
      defaultHtmlMode: false,
      listDensity: "comfortable" as const,
      setLocale: (locale) => set({ locale }),
      setTheme: (theme) => set({ theme }),
      setSidebarWidth: (width) => set({ sidebarWidth: width }),
      setMailListWidth: (width) => set({ mailListWidth: width }),
      togglePreview: () => set((s) => ({ showPreview: !s.showPreview })),
      setSignature: (accountId, content) =>
        set((s) => {
          const filtered = s.signatures.filter((sig) => sig.accountId !== accountId);
          return { signatures: [...filtered, { accountId, content }] };
        }),
      removeSignature: (accountId) =>
        set((s) => ({ signatures: s.signatures.filter((sig) => sig.accountId !== accountId) })),
      getSignature: (accountId) => {
        const sig = get().signatures.find((s) => s.accountId === accountId);
        return sig?.content || "";
      },
      setSyncInterval: (ms) => set({ syncIntervalMs: ms }),
      setDefaultHtmlMode: (enabled) => set({ defaultHtmlMode: enabled }),
      setListDensity: (density) => set({ listDensity: density }),
    }),
    { name: "swallow-settings" }
  )
);