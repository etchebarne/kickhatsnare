import path from "node:path";

import { app, BrowserWindow, dialog, ipcMain, shell, type IpcMainInvokeEvent } from "electron";

import { ipcChannels } from "../shared/ipc";
import { CoreServer } from "./server-process";

let mainWindow: BrowserWindow | null = null;
let coreServer: CoreServer | null = null;

app.setPath("userData", path.join(app.getPath("appData"), "kickhatsnare"));

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 1440,
    height: 900,
    minWidth: 960,
    minHeight: 640,
    show: false,
    frame: false,
    backgroundColor: "#0a0a0a",
    webPreferences: {
      preload: path.join(__dirname, "../preload/index.js"),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });

  mainWindow.once("ready-to-show", () => mainWindow?.show());
  mainWindow.on("closed", () => {
    mainWindow = null;
  });
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    void shell.openExternal(url);
    return { action: "deny" };
  });

  if (process.env.ELECTRON_RENDERER_URL) {
    void mainWindow.loadURL(process.env.ELECTRON_RENDERER_URL);
  } else {
    void mainWindow.loadFile(path.join(__dirname, "../renderer/index.html"));
  }
}

function windowForEvent(event: IpcMainInvokeEvent): BrowserWindow | null {
  return BrowserWindow.fromWebContents(event.sender);
}

function getCoreServer(): CoreServer {
  if (!coreServer) throw new Error("Core server is not running");
  return coreServer;
}

function importWorkspaceAudio(payload: unknown) {
  if (!payload || typeof payload !== "object") {
    throw new TypeError("Audio import requires a request payload");
  }
  const request = payload as Record<string, unknown>;
  if (
    !Array.isArray(request.sourcePaths) ||
    !request.sourcePaths.every((path) => typeof path === "string") ||
    typeof request.targetDirectory !== "string"
  ) {
    throw new TypeError("Audio import requires file paths and a target directory");
  }

  return getCoreServer().importWorkspaceAudio(request.sourcePaths, request.targetDirectory);
}

async function openProject(event: IpcMainInvokeEvent) {
  const window = windowForEvent(event);
  if (!window) throw new Error("Project dialog requires an application window");

  const selection = await dialog.showOpenDialog(window, {
    title: "Open KickHatSnare Project",
    properties: ["openFile"],
    filters: [{ name: "KickHatSnare project", extensions: ["khs"] }],
  });
  const projectFilePath = selection.filePaths[0];
  if (selection.canceled || !projectFilePath) return null;

  return getCoreServer().openWorkspace(projectFilePath);
}

async function pinFolder(event: IpcMainInvokeEvent) {
  const window = windowForEvent(event);
  if (!window) throw new Error("Folder picker requires an application window");

  const selection = await dialog.showOpenDialog(window, {
    title: "Pin Sample Folder",
    buttonLabel: "Pin Folder",
    properties: ["openDirectory"],
  });
  const directoryPath = selection.filePaths[0];
  if (selection.canceled || !directoryPath) return null;

  return getCoreServer().pinFolder(directoryPath);
}

async function saveProjectAs(event: IpcMainInvokeEvent) {
  const window = windowForEvent(event);
  if (!window) throw new Error("Project dialog requires an application window");

  const workspace = await getCoreServer().getWorkspace();
  const selection = await dialog.showSaveDialog(window, {
    title: "Choose Project Directory",
    buttonLabel: "Create Project",
    defaultPath: path.join(app.getPath("documents"), workspace.name),
    message: "KickHatSnare will create a project directory at this location.",
    properties: ["createDirectory", "showOverwriteConfirmation"],
  });
  if (selection.canceled || !selection.filePath) return null;

  return getCoreServer().saveWorkspaceAs(selection.filePath);
}

async function saveProject(event: IpcMainInvokeEvent) {
  const workspace = await getCoreServer().getWorkspace();
  return workspace.rootPath ? getCoreServer().saveWorkspace() : saveProjectAs(event);
}

app.whenReady().then(async () => {
  app.setAppUserModelId("com.kickhatsnare.desktop");

  coreServer = new CoreServer(app.getPath("userData"));
  await coreServer.start();
  ipcMain.handle(ipcChannels.audioGetTransport, () => getCoreServer().getTransport());
  ipcMain.handle(ipcChannels.audioPause, () => getCoreServer().pauseAudio());
  ipcMain.handle(ipcChannels.audioPlay, () => getCoreServer().playAudio());
  ipcMain.handle(ipcChannels.audioSeek, (_event, positionTick: number) =>
    getCoreServer().seekAudio({ positionTick }),
  );
  ipcMain.handle(ipcChannels.audioStop, () => getCoreServer().stopAudio());
  ipcMain.handle(ipcChannels.ping, () => getCoreServer().ping());
  ipcMain.handle(ipcChannels.libraryGet, () => getCoreServer().getLibrary());
  ipcMain.handle(ipcChannels.libraryPinFolder, pinFolder);
  ipcMain.handle(ipcChannels.libraryUnpinFolder, (_event, id: string) =>
    getCoreServer().unpinFolder(id),
  );
  ipcMain.handle(ipcChannels.settingsGet, () => getCoreServer().getSettings());
  ipcMain.handle(ipcChannels.settingsSet, (_event, params) => getCoreServer().setSetting(params));
  ipcMain.handle(ipcChannels.workspaceCreateDirectory, (_event, path: string) =>
    getCoreServer().createWorkspaceDirectory(path),
  );
  ipcMain.handle(ipcChannels.workspaceAddAudioClip, (_event, params) =>
    getCoreServer().addAudioClip(params),
  );
  ipcMain.handle(ipcChannels.workspaceConnectMixPorts, (_event, params) =>
    getCoreServer().connectMixPorts(params),
  );
  ipcMain.handle(ipcChannels.workspaceDeleteEntry, (_event, path: string) =>
    getCoreServer().deleteWorkspaceEntry(path),
  );
  ipcMain.handle(ipcChannels.workspaceDeleteTimelineClip, (_event, params) =>
    getCoreServer().deleteTimelineClip(params),
  );
  ipcMain.handle(ipcChannels.workspaceDeleteTimelineTrack, (_event, params) =>
    getCoreServer().deleteTimelineTrack(params),
  );
  ipcMain.handle(ipcChannels.workspaceDisconnectMixPorts, (_event, params) =>
    getCoreServer().disconnectMixPorts(params),
  );
  ipcMain.handle(ipcChannels.workspaceGet, () => getCoreServer().getWorkspace());
  ipcMain.handle(ipcChannels.workspaceImportAudio, (_event, payload: unknown) =>
    importWorkspaceAudio(payload),
  );
  ipcMain.handle(
    ipcChannels.workspaceMoveEntry,
    (_event, sourcePath: string, destinationPath: string) =>
      getCoreServer().moveWorkspaceEntry(sourcePath, destinationPath),
  );
  ipcMain.handle(ipcChannels.workspaceNew, () => getCoreServer().newWorkspace());
  ipcMain.handle(ipcChannels.workspaceOpen, openProject);
  ipcMain.handle(ipcChannels.workspaceRedo, () => getCoreServer().redoWorkspace());
  ipcMain.handle(ipcChannels.workspaceSave, saveProject);
  ipcMain.handle(ipcChannels.workspaceSaveAs, saveProjectAs);
  ipcMain.handle(ipcChannels.workspaceSaveTimelineClip, (_event, params) =>
    getCoreServer().saveTimelineClip(params),
  );
  ipcMain.handle(ipcChannels.workspaceSaveTimelineTrack, (_event, params) =>
    getCoreServer().saveTimelineTrack(params),
  );
  ipcMain.handle(ipcChannels.workspaceSetTimelineSettings, (_event, params) =>
    getCoreServer().setTimelineSettings(params),
  );
  ipcMain.handle(ipcChannels.workspaceSetMasterMix, (_event, params) =>
    getCoreServer().setMasterMix(params),
  );
  ipcMain.handle(ipcChannels.workspaceSetMixNodePosition, (_event, params) =>
    getCoreServer().setMixNodePosition(params),
  );
  ipcMain.handle(ipcChannels.workspaceSetTimelineClipProperties, (_event, params) =>
    getCoreServer().setTimelineClipProperties(params),
  );
  ipcMain.handle(ipcChannels.workspaceSplitTimelineClip, (_event, params) =>
    getCoreServer().splitTimelineClip(params),
  );
  ipcMain.handle(ipcChannels.workspaceUndo, () => getCoreServer().undoWorkspace());
  ipcMain.handle(ipcChannels.windowMinimize, (event) => windowForEvent(event)?.minimize());
  ipcMain.handle(ipcChannels.windowToggleMaximize, (event) => {
    const window = windowForEvent(event);
    if (!window) return;

    if (window.isMaximized()) {
      window.unmaximize();
    } else {
      window.maximize();
    }
  });
  ipcMain.handle(ipcChannels.windowClose, (event) => windowForEvent(event)?.close());

  createWindow();
});

app.on("before-quit", () => {
  ipcMain.removeHandler(ipcChannels.audioGetTransport);
  ipcMain.removeHandler(ipcChannels.audioPause);
  ipcMain.removeHandler(ipcChannels.audioPlay);
  ipcMain.removeHandler(ipcChannels.audioSeek);
  ipcMain.removeHandler(ipcChannels.audioStop);
  ipcMain.removeHandler(ipcChannels.ping);
  ipcMain.removeHandler(ipcChannels.libraryGet);
  ipcMain.removeHandler(ipcChannels.libraryPinFolder);
  ipcMain.removeHandler(ipcChannels.libraryUnpinFolder);
  ipcMain.removeHandler(ipcChannels.settingsGet);
  ipcMain.removeHandler(ipcChannels.settingsSet);
  ipcMain.removeHandler(ipcChannels.workspaceCreateDirectory);
  ipcMain.removeHandler(ipcChannels.workspaceAddAudioClip);
  ipcMain.removeHandler(ipcChannels.workspaceConnectMixPorts);
  ipcMain.removeHandler(ipcChannels.workspaceDeleteEntry);
  ipcMain.removeHandler(ipcChannels.workspaceDeleteTimelineClip);
  ipcMain.removeHandler(ipcChannels.workspaceDeleteTimelineTrack);
  ipcMain.removeHandler(ipcChannels.workspaceDisconnectMixPorts);
  ipcMain.removeHandler(ipcChannels.workspaceGet);
  ipcMain.removeHandler(ipcChannels.workspaceImportAudio);
  ipcMain.removeHandler(ipcChannels.workspaceMoveEntry);
  ipcMain.removeHandler(ipcChannels.workspaceNew);
  ipcMain.removeHandler(ipcChannels.workspaceOpen);
  ipcMain.removeHandler(ipcChannels.workspaceRedo);
  ipcMain.removeHandler(ipcChannels.workspaceSave);
  ipcMain.removeHandler(ipcChannels.workspaceSaveAs);
  ipcMain.removeHandler(ipcChannels.workspaceSaveTimelineClip);
  ipcMain.removeHandler(ipcChannels.workspaceSaveTimelineTrack);
  ipcMain.removeHandler(ipcChannels.workspaceSetTimelineSettings);
  ipcMain.removeHandler(ipcChannels.workspaceSetMasterMix);
  ipcMain.removeHandler(ipcChannels.workspaceSetMixNodePosition);
  ipcMain.removeHandler(ipcChannels.workspaceSetTimelineClipProperties);
  ipcMain.removeHandler(ipcChannels.workspaceSplitTimelineClip);
  ipcMain.removeHandler(ipcChannels.workspaceUndo);
  ipcMain.removeHandler(ipcChannels.windowMinimize);
  ipcMain.removeHandler(ipcChannels.windowToggleMaximize);
  ipcMain.removeHandler(ipcChannels.windowClose);
  coreServer?.stop();
  coreServer = null;
});

app.on("window-all-closed", () => {
  app.quit();
});
