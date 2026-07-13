import { create } from "zustand";

import type { TransportSnapshot } from "@shared/ipc";

interface TransportStore {
  transport: TransportSnapshot;
  isPending: boolean;
  error: string | null;
  play(): Promise<void>;
  pause(): Promise<void>;
  stop(): Promise<void>;
  seek(positionTick: number): Promise<void>;
  refresh(): Promise<void>;
}

const initialTransport: TransportSnapshot = {
  state: "stopped",
  positionTick: 0,
  durationTicks: 0,
  lastError: null,
};

export const useTransportStore = create<TransportStore>((set) => ({
  transport: initialTransport,
  isPending: false,
  error: null,
  async play() {
    await command(set, () => window.kickHatSnare.playAudio());
  },
  async pause() {
    await command(set, () => window.kickHatSnare.pauseAudio());
  },
  async stop() {
    await command(set, () => window.kickHatSnare.stopAudio());
  },
  async seek(positionTick) {
    await command(set, () => window.kickHatSnare.seekAudio(positionTick));
  },
  async refresh() {
    try {
      const transport = await window.kickHatSnare.getTransport();
      set({ transport, error: transport.lastError });
    } catch (error) {
      set({ error: errorMessage(error) });
    }
  },
}));

async function command(
  set: (state: Partial<TransportStore>) => void,
  operation: () => Promise<TransportSnapshot>,
) {
  set({ isPending: true, error: null });
  try {
    const transport = await operation();
    set({ transport, isPending: false, error: transport.lastError });
  } catch (error) {
    set({ isPending: false, error: errorMessage(error) });
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
