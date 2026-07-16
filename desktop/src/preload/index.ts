import { contextBridge, ipcRenderer, webUtils } from "electron";

import { ipcChannels, type KickHatSnareApi } from "../shared/ipc";

const api: KickHatSnareApi = {
  getTransport: () => ipcRenderer.invoke(ipcChannels.audioGetTransport),
  pauseAudio: () => ipcRenderer.invoke(ipcChannels.audioPause),
  playAudio: () => ipcRenderer.invoke(ipcChannels.audioPlay),
  seekAudio: (positionTick) => ipcRenderer.invoke(ipcChannels.audioSeek, positionTick),
  stopAudio: () => ipcRenderer.invoke(ipcChannels.audioStop),
  ping: () => ipcRenderer.invoke(ipcChannels.ping),
  getLibrary: () => ipcRenderer.invoke(ipcChannels.libraryGet),
  pinFolder: () => ipcRenderer.invoke(ipcChannels.libraryPinFolder),
  unpinFolder: (id) => ipcRenderer.invoke(ipcChannels.libraryUnpinFolder, id),
  getSettings: () => ipcRenderer.invoke(ipcChannels.settingsGet),
  setSetting: (params) => ipcRenderer.invoke(ipcChannels.settingsSet, params),
  createWorkspaceDirectory: (path) =>
    ipcRenderer.invoke(ipcChannels.workspaceCreateDirectory, path),
  addAudioClip: (params) => ipcRenderer.invoke(ipcChannels.workspaceAddAudioClip, params),
  connectMixPorts: (params) => ipcRenderer.invoke(ipcChannels.workspaceConnectMixPorts, params),
  deleteWorkspaceEntry: (path) => ipcRenderer.invoke(ipcChannels.workspaceDeleteEntry, path),
  deleteTimelineClip: (params) =>
    ipcRenderer.invoke(ipcChannels.workspaceDeleteTimelineClip, params),
  deleteTimelineTrack: (params) =>
    ipcRenderer.invoke(ipcChannels.workspaceDeleteTimelineTrack, params),
  disconnectMixPorts: (params) =>
    ipcRenderer.invoke(ipcChannels.workspaceDisconnectMixPorts, params),
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
  redoWorkspace: () => ipcRenderer.invoke(ipcChannels.workspaceRedo),
  saveProject: () => ipcRenderer.invoke(ipcChannels.workspaceSave),
  saveProjectAs: () => ipcRenderer.invoke(ipcChannels.workspaceSaveAs),
  saveTimelineClip: (params) => ipcRenderer.invoke(ipcChannels.workspaceSaveTimelineClip, params),
  saveTimelineTrack: (params) => ipcRenderer.invoke(ipcChannels.workspaceSaveTimelineTrack, params),
  setTimelineSettings: (params) =>
    ipcRenderer.invoke(ipcChannels.workspaceSetTimelineSettings, params),
  setMasterMix: (params) => ipcRenderer.invoke(ipcChannels.workspaceSetMasterMix, params),
  setMixNodePosition: (params) =>
    ipcRenderer.invoke(ipcChannels.workspaceSetMixNodePosition, params),
  setTimelineClipProperties: (params) =>
    ipcRenderer.invoke(ipcChannels.workspaceSetTimelineClipProperties, params),
  splitTimelineClip: (params) => ipcRenderer.invoke(ipcChannels.workspaceSplitTimelineClip, params),
  undoWorkspace: () => ipcRenderer.invoke(ipcChannels.workspaceUndo),
  minimizeWindow: () => ipcRenderer.invoke(ipcChannels.windowMinimize),
  toggleMaximizeWindow: () => ipcRenderer.invoke(ipcChannels.windowToggleMaximize),
  closeWindow: () => ipcRenderer.invoke(ipcChannels.windowClose),
};

contextBridge.exposeInMainWorld("kickHatSnare", api);
