import { create } from "zustand";

import type {
  AddAudioClipParams,
  ConnectMixPortsParams,
  DisconnectMixPortsParams,
  LibrarySnapshot,
  RecoverMissingMediaParams,
  SaveTimelineClipParams,
  SaveTimelineTrackParams,
  SetTimelineSettingsParams,
  SetMasterMixParams,
  SetMixNodePositionParams,
  SetTimelineClipPropertiesParams,
  SplitTimelineClipParams,
  WorkspaceSnapshot,
} from "@shared/ipc";

import { useTransportStore } from "./transport-store";

type ServerStatus = "connecting" | "ready" | "unavailable";

interface AppState {
  serverStatus: ServerStatus;
  library: LibrarySnapshot | null;
  workspace: WorkspaceSnapshot | null;
  operationError: string | null;
  applyWorkspaceUpdate(workspace: WorkspaceSnapshot): void;
  connect(): Promise<void>;
  createWorkspaceDirectory(path: string): Promise<boolean>;
  addAudioClip(params: AddAudioClipParams): Promise<boolean>;
  connectMixPorts(params: ConnectMixPortsParams): Promise<boolean>;
  deleteWorkspaceEntry(path: string): Promise<boolean>;
  deleteTimelineClip(id: string): Promise<boolean>;
  deleteTimelineTrack(id: string): Promise<boolean>;
  disconnectMixPorts(params: DisconnectMixPortsParams): Promise<boolean>;
  importAudioFiles(files: File[], targetDirectory: string): Promise<string[]>;
  locateMissingMedia(sourcePath: string): Promise<boolean>;
  moveWorkspaceEntry(sourcePath: string, destinationPath: string): Promise<boolean>;
  pinFolder(): Promise<boolean>;
  unpinFolder(id: string): Promise<boolean>;
  newProject(): Promise<void>;
  openProject(): Promise<void>;
  redo(): Promise<boolean>;
  recoverMissingMedia(
    sourcePath: string,
    action: Extract<RecoverMissingMediaParams["action"], "leaveEmpty" | "deleteClips">,
  ): Promise<boolean>;
  saveProject(): Promise<void>;
  saveProjectAs(): Promise<void>;
  saveTimelineClip(params: SaveTimelineClipParams): Promise<boolean>;
  saveTimelineTrack(params: SaveTimelineTrackParams): Promise<boolean>;
  setTimelineSettings(params: SetTimelineSettingsParams): Promise<boolean>;
  setMasterMix(params: SetMasterMixParams): Promise<boolean>;
  setMixNodePosition(params: SetMixNodePositionParams): Promise<boolean>;
  setTimelineClipProperties(params: SetTimelineClipPropertiesParams): Promise<boolean>;
  splitTimelineClip(params: SplitTimelineClipParams): Promise<boolean>;
  undo(): Promise<boolean>;
}

export const useAppStore = create<AppState>((set, get) => ({
  serverStatus: "connecting",
  library: null,
  workspace: null,
  operationError: null,
  applyWorkspaceUpdate(workspace) {
    set({ workspace, operationError: null });
    void useTransportStore.getState().refresh();
  },
  async connect() {
    set({ serverStatus: "connecting", operationError: null });
    try {
      const response = await window.kickHatSnare.ping();
      if (response !== "ready") throw new Error("Core server did not become ready");

      const [library, workspace] = await Promise.all([
        window.kickHatSnare.getLibrary(),
        window.kickHatSnare.getWorkspace(),
      ]);
      set({ serverStatus: "ready", library, workspace });
    } catch (error) {
      set({ serverStatus: "unavailable", operationError: errorMessage(error) });
    }
  },
  async createWorkspaceDirectory(path) {
    return updateWorkspace(set, () => window.kickHatSnare.createWorkspaceDirectory(path));
  },
  async addAudioClip(params) {
    return updateWorkspace(set, () => window.kickHatSnare.addAudioClip(params));
  },
  async connectMixPorts(params) {
    return updateWorkspace(set, () => window.kickHatSnare.connectMixPorts(params));
  },
  async deleteWorkspaceEntry(path) {
    return updateWorkspace(set, () => window.kickHatSnare.deleteWorkspaceEntry(path));
  },
  async deleteTimelineClip(id) {
    return updateWorkspace(set, () => window.kickHatSnare.deleteTimelineClip({ id }));
  },
  async deleteTimelineTrack(id) {
    return updateWorkspace(set, () => window.kickHatSnare.deleteTimelineTrack({ id }));
  },
  async disconnectMixPorts(params) {
    return updateWorkspace(set, () => window.kickHatSnare.disconnectMixPorts(params));
  },
  async importAudioFiles(files, targetDirectory) {
    const previousFiles = new Set(get().workspace?.files ?? []);
    set({ operationError: null });
    try {
      const workspace = await window.kickHatSnare.importAudioFiles(files, targetDirectory);
      set({ workspace });
      await useTransportStore.getState().refresh();
      return workspace.files.filter((path) => !path.endsWith("/") && !previousFiles.has(path));
    } catch (error) {
      set({ operationError: errorMessage(error) });
      return [];
    }
  },
  async locateMissingMedia(sourcePath) {
    return updateWorkspace(set, () => window.kickHatSnare.locateMissingMedia(sourcePath));
  },
  async moveWorkspaceEntry(sourcePath, destinationPath) {
    return updateWorkspace(set, () =>
      window.kickHatSnare.moveWorkspaceEntry(sourcePath, destinationPath),
    );
  },
  async pinFolder() {
    return updateLibrary(set, () => window.kickHatSnare.pinFolder());
  },
  async unpinFolder(id) {
    return updateLibrary(set, () => window.kickHatSnare.unpinFolder(id));
  },
  async newProject() {
    await updateWorkspace(set, () => window.kickHatSnare.newProject());
  },
  async openProject() {
    await updateWorkspace(set, () => window.kickHatSnare.openProject());
  },
  async redo() {
    if (!get().workspace?.history.canRedo) return false;
    return updateWorkspace(set, () => window.kickHatSnare.redoWorkspace());
  },
  async recoverMissingMedia(sourcePath, action) {
    return updateWorkspace(set, () =>
      window.kickHatSnare.recoverMissingMedia({
        sourcePath,
        action,
        replacementPath: null,
      }),
    );
  },
  async saveProject() {
    await updateWorkspace(set, () => window.kickHatSnare.saveProject());
  },
  async saveProjectAs() {
    await updateWorkspace(set, () => window.kickHatSnare.saveProjectAs());
  },
  async saveTimelineClip(params) {
    return updateWorkspace(set, () => window.kickHatSnare.saveTimelineClip(params));
  },
  async saveTimelineTrack(params) {
    return updateWorkspace(set, () => window.kickHatSnare.saveTimelineTrack(params));
  },
  async setTimelineSettings(params) {
    return updateWorkspace(set, () => window.kickHatSnare.setTimelineSettings(params));
  },
  async setMasterMix(params) {
    return updateWorkspace(set, () => window.kickHatSnare.setMasterMix(params));
  },
  async setMixNodePosition(params) {
    return updateWorkspace(set, () => window.kickHatSnare.setMixNodePosition(params));
  },
  async setTimelineClipProperties(params) {
    return updateWorkspace(set, () => window.kickHatSnare.setTimelineClipProperties(params));
  },
  async splitTimelineClip(params) {
    return updateWorkspace(set, () => window.kickHatSnare.splitTimelineClip(params));
  },
  async undo() {
    if (!get().workspace?.history.canUndo) return false;
    return updateWorkspace(set, () => window.kickHatSnare.undoWorkspace());
  },
}));

async function updateLibrary(
  set: (state: Partial<AppState>) => void,
  operation: () => Promise<LibrarySnapshot | null>,
): Promise<boolean> {
  set({ operationError: null });
  try {
    const library = await operation();
    if (library) set({ library });
    return library !== null;
  } catch (error) {
    set({ operationError: errorMessage(error) });
    return false;
  }
}

async function updateWorkspace(
  set: (state: Partial<AppState>) => void,
  operation: () => Promise<WorkspaceSnapshot | null>,
): Promise<boolean> {
  set({ operationError: null });
  try {
    const workspace = await operation();
    if (workspace) {
      set({ workspace });
      await useTransportStore.getState().refresh();
    }
    return workspace !== null;
  } catch (error) {
    set({ operationError: errorMessage(error) });
    return false;
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
