import path from "node:path";

import { app, BrowserWindow, ipcMain, shell } from "electron";

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

app.whenReady().then(async () => {
  app.setAppUserModelId("com.kickhatsnare.desktop");

  coreServer = new CoreServer();
  await coreServer.start();
  ipcMain.handle(ipcChannels.ping, () => coreServer?.ping());

  createWindow();
});

app.on("before-quit", () => {
  ipcMain.removeHandler(ipcChannels.ping);
  coreServer?.stop();
  coreServer = null;
});

app.on("window-all-closed", () => {
  app.quit();
});
