import { contextBridge, ipcRenderer } from "electron";

import { ipcChannels, type KickHatSnareApi } from "../shared/ipc";

const api: KickHatSnareApi = {
  ping: () => ipcRenderer.invoke(ipcChannels.ping),
  minimizeWindow: () => ipcRenderer.invoke(ipcChannels.windowMinimize),
  toggleMaximizeWindow: () => ipcRenderer.invoke(ipcChannels.windowToggleMaximize),
  closeWindow: () => ipcRenderer.invoke(ipcChannels.windowClose),
};

contextBridge.exposeInMainWorld("kickHatSnare", api);
