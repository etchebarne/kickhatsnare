import { contextBridge, ipcRenderer } from "electron";

import { ipcChannels, type KickHatSnareApi } from "../shared/ipc";

const api: KickHatSnareApi = {
  ping: () => ipcRenderer.invoke(ipcChannels.ping),
};

contextBridge.exposeInMainWorld("kickHatSnare", api);
