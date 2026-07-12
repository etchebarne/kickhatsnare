import type { ResultFor } from "./generated/ipc";

export const ipcChannels = {
  ping: "core:ping",
} as const;

export interface KickHatSnareApi {
  ping(): Promise<ResultFor<"system.ping">>;
}
