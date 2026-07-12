/// <reference types="vite/client" />

import type { KickHatSnareApi } from "@shared/ipc";

declare global {
  interface Window {
    kickHatSnare: KickHatSnareApi;
  }
}
