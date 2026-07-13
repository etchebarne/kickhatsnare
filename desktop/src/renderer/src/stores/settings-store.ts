import { create } from "zustand";

import type { SetSettingParams, SettingsSnapshot } from "@shared/ipc";

interface SettingsState {
  isOpen: boolean;
  snapshot: SettingsSnapshot | null;
  isLoading: boolean;
  pendingSettingId: string | null;
  error: string | null;
  open(): void;
  setOpen(open: boolean): void;
  load(): Promise<void>;
  update(params: SetSettingParams): Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  isOpen: false,
  snapshot: null,
  isLoading: false,
  pendingSettingId: null,
  error: null,
  open() {
    set({ isOpen: true });
    void get().load();
  },
  setOpen(isOpen) {
    set({ isOpen });
  },
  async load() {
    if (get().isLoading) return;
    set({ isLoading: true, error: null });
    try {
      const snapshot = await window.kickHatSnare.getSettings();
      set({ snapshot });
    } catch (error) {
      set({ error: errorMessage(error) });
    } finally {
      set({ isLoading: false });
    }
  },
  async update(params) {
    set({ pendingSettingId: params.id, error: null });
    try {
      const snapshot = await window.kickHatSnare.setSetting(params);
      set({ snapshot });
    } catch (error) {
      set({ error: errorMessage(error) });
    } finally {
      set({ pendingSettingId: null });
    }
  },
}));

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
