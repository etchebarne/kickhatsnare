import type { ParamsFor, ResultFor } from "./generated/ipc";

export type LibrarySnapshot = ResultFor<"library.get">;
export type WorkspaceSnapshot = ResultFor<"workspace.get">;
export type DeleteTimelineClipParams = ParamsFor<"workspace.deleteTimelineClip">;
export type DeleteTimelineTrackParams = ParamsFor<"workspace.deleteTimelineTrack">;
export type SaveTimelineClipParams = ParamsFor<"workspace.saveTimelineClip">;
export type SaveTimelineTrackParams = ParamsFor<"workspace.saveTimelineTrack">;
export type SetTimelineSettingsParams = ParamsFor<"workspace.setTimelineSettings">;

export const ipcChannels = {
  libraryGet: "library:get",
  libraryPinFolder: "library:pin-folder",
  libraryUnpinFolder: "library:unpin-folder",
  ping: "core:ping",
  workspaceCreateDirectory: "workspace:create-directory",
  workspaceDeleteEntry: "workspace:delete-entry",
  workspaceDeleteTimelineClip: "workspace:delete-timeline-clip",
  workspaceDeleteTimelineTrack: "workspace:delete-timeline-track",
  workspaceGet: "workspace:get",
  workspaceImportAudio: "workspace:import-audio",
  workspaceMoveEntry: "workspace:move-entry",
  workspaceNew: "workspace:new",
  workspaceOpen: "workspace:open",
  workspaceSave: "workspace:save",
  workspaceSaveAs: "workspace:save-as",
  workspaceSaveTimelineClip: "workspace:save-timeline-clip",
  workspaceSaveTimelineTrack: "workspace:save-timeline-track",
  workspaceSetTimelineSettings: "workspace:set-timeline-settings",
  windowMinimize: "window:minimize",
  windowToggleMaximize: "window:toggle-maximize",
  windowClose: "window:close",
} as const;

export interface KickHatSnareApi {
  ping(): Promise<ResultFor<"system.ping">>;
  getLibrary(): Promise<LibrarySnapshot>;
  pinFolder(): Promise<LibrarySnapshot | null>;
  unpinFolder(id: string): Promise<LibrarySnapshot>;
  createWorkspaceDirectory(path: string): Promise<WorkspaceSnapshot>;
  deleteWorkspaceEntry(path: string): Promise<WorkspaceSnapshot>;
  deleteTimelineClip(params: DeleteTimelineClipParams): Promise<WorkspaceSnapshot>;
  deleteTimelineTrack(params: DeleteTimelineTrackParams): Promise<WorkspaceSnapshot>;
  getWorkspace(): Promise<WorkspaceSnapshot>;
  importAudioFiles(files: File[], targetDirectory: string): Promise<WorkspaceSnapshot>;
  moveWorkspaceEntry(sourcePath: string, destinationPath: string): Promise<WorkspaceSnapshot>;
  newProject(): Promise<WorkspaceSnapshot>;
  openProject(): Promise<WorkspaceSnapshot | null>;
  saveProject(): Promise<WorkspaceSnapshot | null>;
  saveProjectAs(): Promise<WorkspaceSnapshot | null>;
  saveTimelineClip(params: SaveTimelineClipParams): Promise<WorkspaceSnapshot>;
  saveTimelineTrack(params: SaveTimelineTrackParams): Promise<WorkspaceSnapshot>;
  setTimelineSettings(params: SetTimelineSettingsParams): Promise<WorkspaceSnapshot>;
  minimizeWindow(): Promise<void>;
  toggleMaximizeWindow(): Promise<void>;
  closeWindow(): Promise<void>;
}
