import path from "node:path";

import { app, BrowserWindow, ipcMain, shell, type IpcMainInvokeEvent } from "electron";

import { ipcChannels } from "../shared/ipc";
import { CoreServer } from "./server-process";

let mainWindow: BrowserWindow | null = null;
let coreServer: CoreServer | null = null;

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

app.whenReady().then(async () => {
  app.setAppUserModelId("com.kickhatsnare.desktop");

  coreServer = new CoreServer();
  await coreServer.start();
  ipcMain.handle(ipcChannels.ping, () => coreServer?.ping());
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
  ipcMain.removeHandler(ipcChannels.ping);
  ipcMain.removeHandler(ipcChannels.windowMinimize);
  ipcMain.removeHandler(ipcChannels.windowToggleMaximize);
  ipcMain.removeHandler(ipcChannels.windowClose);
  coreServer?.stop();
  coreServer = null;
});

app.on("window-all-closed", () => {
  app.quit();
});
