import { contextBridge, ipcRenderer, webUtils } from "electron";

import { ipcChannels, type KickHatSnareApi } from "../shared/ipc";

const api: KickHatSnareApi = {
  ping: () => ipcRenderer.invoke(ipcChannels.ping),
  getLibrary: () => ipcRenderer.invoke(ipcChannels.libraryGet),
  pinFolder: () => ipcRenderer.invoke(ipcChannels.libraryPinFolder),
  unpinFolder: (id) => ipcRenderer.invoke(ipcChannels.libraryUnpinFolder, id),
  createWorkspaceDirectory: (path) =>
    ipcRenderer.invoke(ipcChannels.workspaceCreateDirectory, path),
  deleteWorkspaceEntry: (path) => ipcRenderer.invoke(ipcChannels.workspaceDeleteEntry, path),
  getWorkspace: () => ipcRenderer.invoke(ipcChannels.workspaceGet),
  importAudioFiles: (files, targetDirectory) =>
    ipcRenderer.invoke(ipcChannels.workspaceImportAudio, {
      sourcePaths: files.map((file) => webUtils.getPathForFile(file)),
      targetDirectory,
    }),
  moveWorkspaceEntry: (sourcePath, destinationPath) =>
    ipcRenderer.invoke(ipcChannels.workspaceMoveEntry, sourcePath, destinationPath),
  newProject: () => ipcRenderer.invoke(ipcChannels.workspaceNew),
  openProject: () => ipcRenderer.invoke(ipcChannels.workspaceOpen),
  saveProject: () => ipcRenderer.invoke(ipcChannels.workspaceSave),
  saveProjectAs: () => ipcRenderer.invoke(ipcChannels.workspaceSaveAs),
  minimizeWindow: () => ipcRenderer.invoke(ipcChannels.windowMinimize),
  toggleMaximizeWindow: () => ipcRenderer.invoke(ipcChannels.windowToggleMaximize),
  closeWindow: () => ipcRenderer.invoke(ipcChannels.windowClose),
};

contextBridge.exposeInMainWorld("kickHatSnare", api);
