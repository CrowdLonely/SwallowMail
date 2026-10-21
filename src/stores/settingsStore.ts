import { create } from "zustand";
import { persist } from "zustand/middleware";

interface SettingsState {
  locale: string;
  theme: "light" | "dark" | "system";
  sidebarWidth: number;
  mailListWidth: number;
  showPreview: boolean;
  setLocale: (locale: string) => void;
  setTheme: (theme: "light" | "dark" | "system") => void;
  setSidebarWidth: (width: number) => void;
  setMailListWidth: (width: number) => void;
  togglePreview: () => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      locale: "zh-CN",
      theme: "system",
      sidebarWidth: 220,
      mailListWidth: 360,
      showPreview: true,
      setLocale: (locale) => set({ locale }),
      setTheme: (theme) => set({ theme }),
      setSidebarWidth: (width) => set({ sidebarWidth: width }),
      setMailListWidth: (width) => set({ mailListWidth: width }),
      togglePreview: () => set((s) => ({ showPreview: !s.showPreview })),
    }),
    { name: "swallow-settings" }
  )
);
