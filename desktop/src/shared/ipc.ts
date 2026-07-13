import type { ResultFor } from "./generated/ipc";

export type LibrarySnapshot = ResultFor<"library.get">;
export type WorkspaceSnapshot = ResultFor<"workspace.get">;

export const ipcChannels = {
  libraryGet: "library:get",
  libraryPinFolder: "library:pin-folder",
  libraryUnpinFolder: "library:unpin-folder",
  ping: "core:ping",
  workspaceCreateDirectory: "workspace:create-directory",
  workspaceDeleteEntry: "workspace:delete-entry",
  workspaceGet: "workspace:get",
  workspaceImportAudio: "workspace:import-audio",
  workspaceMoveEntry: "workspace:move-entry",
  workspaceNew: "workspace:new",
  workspaceOpen: "workspace:open",
  workspaceSave: "workspace:save",
  workspaceSaveAs: "workspace:save-as",
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
  getWorkspace(): Promise<WorkspaceSnapshot>;
  importAudioFiles(files: File[], targetDirectory: string): Promise<WorkspaceSnapshot>;
  moveWorkspaceEntry(sourcePath: string, destinationPath: string): Promise<WorkspaceSnapshot>;
  newProject(): Promise<WorkspaceSnapshot>;
  openProject(): Promise<WorkspaceSnapshot | null>;
  saveProject(): Promise<WorkspaceSnapshot | null>;
  saveProjectAs(): Promise<WorkspaceSnapshot | null>;
  minimizeWindow(): Promise<void>;
  toggleMaximizeWindow(): Promise<void>;
  closeWindow(): Promise<void>;
}
