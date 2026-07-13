import { create } from "zustand";

import type { LibrarySnapshot, WorkspaceSnapshot } from "@shared/ipc";

type ServerStatus = "connecting" | "ready" | "unavailable";

interface AppState {
  serverStatus: ServerStatus;
  library: LibrarySnapshot | null;
  workspace: WorkspaceSnapshot | null;
  operationError: string | null;
  connect(): Promise<void>;
  createWorkspaceDirectory(path: string): Promise<boolean>;
  deleteWorkspaceEntry(path: string): Promise<boolean>;
  importAudioFiles(files: File[], targetDirectory: string): Promise<void>;
  moveWorkspaceEntry(sourcePath: string, destinationPath: string): Promise<boolean>;
  pinFolder(): Promise<boolean>;
  unpinFolder(id: string): Promise<boolean>;
  newProject(): Promise<void>;
  openProject(): Promise<void>;
  saveProject(): Promise<void>;
  saveProjectAs(): Promise<void>;
}

export const useAppStore = create<AppState>((set) => ({
  serverStatus: "connecting",
  library: null,
  workspace: null,
  operationError: null,
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
  async deleteWorkspaceEntry(path) {
    return updateWorkspace(set, () => window.kickHatSnare.deleteWorkspaceEntry(path));
  },
  async importAudioFiles(files, targetDirectory) {
    await updateWorkspace(set, () => window.kickHatSnare.importAudioFiles(files, targetDirectory));
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
  async saveProject() {
    await updateWorkspace(set, () => window.kickHatSnare.saveProject());
  },
  async saveProjectAs() {
    await updateWorkspace(set, () => window.kickHatSnare.saveProjectAs());
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
    if (workspace) set({ workspace });
    return workspace !== null;
  } catch (error) {
    set({ operationError: errorMessage(error) });
    return false;
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
