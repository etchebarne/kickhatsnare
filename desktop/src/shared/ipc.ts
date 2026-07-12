import type { ResultFor } from "./generated/ipc";

export const ipcChannels = {
  ping: "core:ping",
  windowMinimize: "window:minimize",
  windowToggleMaximize: "window:toggle-maximize",
  windowClose: "window:close",
} as const;

export interface KickHatSnareApi {
  ping(): Promise<ResultFor<"system.ping">>;
  minimizeWindow(): Promise<void>;
  toggleMaximizeWindow(): Promise<void>;
  closeWindow(): Promise<void>;
}
