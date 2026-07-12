import { create } from "zustand";

type ServerStatus = "connecting" | "ready" | "unavailable";

interface AppState {
  serverStatus: ServerStatus;
  connect(): Promise<void>;
}

export const useAppStore = create<AppState>((set) => ({
  serverStatus: "connecting",
  async connect() {
    set({ serverStatus: "connecting" });
    try {
      const response = await window.kickHatSnare.ping();
      set({ serverStatus: response === "ready" ? "ready" : "unavailable" });
    } catch {
      set({ serverStatus: "unavailable" });
    }
  },
}));
