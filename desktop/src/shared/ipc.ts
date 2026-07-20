import type { ParamsFor, ResultFor } from "./generated/ipc";

export type LibrarySnapshot = ResultFor<"library.get">;
export type SettingsSnapshot = ResultFor<"settings.get">;
export type SetSettingParams = ParamsFor<"settings.set">;
export type WorkspaceSnapshot = ResultFor<"workspace.get">;
export type DeleteTimelineClipParams = ParamsFor<"workspace.deleteTimelineClip">;
export type DeleteTimelineTrackParams = ParamsFor<"workspace.deleteTimelineTrack">;
export type SaveTimelineClipParams = ParamsFor<"workspace.saveTimelineClip">;
export type SaveTimelineTrackParams = ParamsFor<"workspace.saveTimelineTrack">;
export type SetTimelineSettingsParams = ParamsFor<"workspace.setTimelineSettings">;
export type AddAudioClipParams = ParamsFor<"workspace.addAudioClip">;
export type ConnectMixPortsParams = ParamsFor<"workspace.connectMixPorts">;
export type DisconnectMixPortsParams = ParamsFor<"workspace.disconnectMixPorts">;
export type SetMasterMixParams = ParamsFor<"workspace.setMasterMix">;
export type SetMixNodePositionParams = ParamsFor<"workspace.setMixNodePosition">;
export type SetTimelineClipPropertiesParams = ParamsFor<"workspace.setTimelineClipProperties">;
export type SplitTimelineClipParams = ParamsFor<"workspace.splitTimelineClip">;
export type RecoverMissingMediaParams = ParamsFor<"workspace.recoverMissingMedia">;
export type WorkspaceFileMove = ParamsFor<"workspace.reconcileMovedFiles">["moves"][number];
export type TransportSnapshot = ResultFor<"audio.getTransport">;
export type SetLoopRegionParams = ParamsFor<"audio.setLoopRegion">;
export type GetWaveformPeaksParams = ParamsFor<"audio.getWaveformPeaks">;
export type WaveformPeaks = ResultFor<"audio.getWaveformPeaks">;

export const ipcChannels = {
  audioGetTransport: "audio:get-transport",
  audioGetWaveformPeaks: "audio:get-waveform-peaks",
  audioPause: "audio:pause",
  audioPlay: "audio:play",
  audioSeek: "audio:seek",
  audioSetLoopRegion: "audio:set-loop-region",
  audioStop: "audio:stop",
  libraryGet: "library:get",
  libraryPinFolder: "library:pin-folder",
  libraryUnpinFolder: "library:unpin-folder",
  ping: "core:ping",
  settingsGet: "settings:get",
  settingsSet: "settings:set",
  workspaceCreateDirectory: "workspace:create-directory",
  workspaceAddAudioClip: "workspace:add-audio-clip",
  workspaceConnectMixPorts: "workspace:connect-mix-ports",
  workspaceDeleteEntry: "workspace:delete-entry",
  workspaceDeleteTimelineClip: "workspace:delete-timeline-clip",
  workspaceDeleteTimelineTrack: "workspace:delete-timeline-track",
  workspaceDisconnectMixPorts: "workspace:disconnect-mix-ports",
  workspaceGet: "workspace:get",
  workspaceImportAudio: "workspace:import-audio",
  workspaceMoveEntry: "workspace:move-entry",
  workspaceChanged: "workspace:changed",
  workspaceLocateMissingMedia: "workspace:locate-missing-media",
  workspaceNew: "workspace:new",
  workspaceOpen: "workspace:open",
  workspaceRedo: "workspace:redo",
  workspaceRecoverMissingMedia: "workspace:recover-missing-media",
  workspaceSave: "workspace:save",
  workspaceSaveAs: "workspace:save-as",
  workspaceSaveTimelineClip: "workspace:save-timeline-clip",
  workspaceSaveTimelineTrack: "workspace:save-timeline-track",
  workspaceSetTimelineSettings: "workspace:set-timeline-settings",
  workspaceSetMasterMix: "workspace:set-master-mix",
  workspaceSetMixNodePosition: "workspace:set-mix-node-position",
  workspaceSetTimelineClipProperties: "workspace:set-timeline-clip-properties",
  workspaceSplitTimelineClip: "workspace:split-timeline-clip",
  workspaceUndo: "workspace:undo",
  windowMinimize: "window:minimize",
  windowToggleMaximize: "window:toggle-maximize",
  windowClose: "window:close",
} as const;

export interface KickHatSnareApi {
  getTransport(): Promise<TransportSnapshot>;
  getWaveformPeaks(params: GetWaveformPeaksParams): Promise<WaveformPeaks>;
  pauseAudio(): Promise<TransportSnapshot>;
  playAudio(): Promise<TransportSnapshot>;
  seekAudio(positionTick: number): Promise<TransportSnapshot>;
  setLoopRegion(region: SetLoopRegionParams["region"]): Promise<TransportSnapshot>;
  stopAudio(): Promise<TransportSnapshot>;
  ping(): Promise<ResultFor<"system.ping">>;
  getLibrary(): Promise<LibrarySnapshot>;
  pinFolder(): Promise<LibrarySnapshot | null>;
  unpinFolder(id: string): Promise<LibrarySnapshot>;
  getSettings(): Promise<SettingsSnapshot>;
  setSetting(params: SetSettingParams): Promise<SettingsSnapshot>;
  createWorkspaceDirectory(path: string): Promise<WorkspaceSnapshot>;
  addAudioClip(params: AddAudioClipParams): Promise<WorkspaceSnapshot>;
  connectMixPorts(params: ConnectMixPortsParams): Promise<WorkspaceSnapshot>;
  deleteWorkspaceEntry(path: string): Promise<WorkspaceSnapshot>;
  deleteTimelineClip(params: DeleteTimelineClipParams): Promise<WorkspaceSnapshot>;
  deleteTimelineTrack(params: DeleteTimelineTrackParams): Promise<WorkspaceSnapshot>;
  disconnectMixPorts(params: DisconnectMixPortsParams): Promise<WorkspaceSnapshot>;
  getWorkspace(): Promise<WorkspaceSnapshot>;
  importAudioFiles(files: File[], targetDirectory: string): Promise<WorkspaceSnapshot>;
  moveWorkspaceEntry(sourcePath: string, destinationPath: string): Promise<WorkspaceSnapshot>;
  onWorkspaceChanged(listener: (workspace: WorkspaceSnapshot) => void): () => void;
  locateMissingMedia(sourcePath: string): Promise<WorkspaceSnapshot | null>;
  newProject(): Promise<WorkspaceSnapshot>;
  openProject(): Promise<WorkspaceSnapshot | null>;
  redoWorkspace(): Promise<WorkspaceSnapshot>;
  recoverMissingMedia(params: RecoverMissingMediaParams): Promise<WorkspaceSnapshot>;
  saveProject(): Promise<WorkspaceSnapshot | null>;
  saveProjectAs(): Promise<WorkspaceSnapshot | null>;
  saveTimelineClip(params: SaveTimelineClipParams): Promise<WorkspaceSnapshot>;
  saveTimelineTrack(params: SaveTimelineTrackParams): Promise<WorkspaceSnapshot>;
  setTimelineSettings(params: SetTimelineSettingsParams): Promise<WorkspaceSnapshot>;
  setMasterMix(params: SetMasterMixParams): Promise<WorkspaceSnapshot>;
  setMixNodePosition(params: SetMixNodePositionParams): Promise<WorkspaceSnapshot>;
  setTimelineClipProperties(params: SetTimelineClipPropertiesParams): Promise<WorkspaceSnapshot>;
  splitTimelineClip(params: SplitTimelineClipParams): Promise<WorkspaceSnapshot>;
  undoWorkspace(): Promise<WorkspaceSnapshot>;
  minimizeWindow(): Promise<void>;
  toggleMaximizeWindow(): Promise<void>;
  closeWindow(): Promise<void>;
}
