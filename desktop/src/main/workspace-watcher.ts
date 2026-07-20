import { watch, type BigIntStats, type FSWatcher } from "node:fs";
import { lstat, readdir, stat } from "node:fs/promises";
import path from "node:path";

import type { WorkspaceFileMove } from "../shared/ipc";

interface IndexedEntry {
  path: string;
}

interface WorkspaceIndex {
  byIdentity: Map<string, IndexedEntry[]>;
  fingerprint: string;
}

interface RemovedFile {
  path: string;
  expiresAt: number;
}

type ChangeHandler = (moves: WorkspaceFileMove[]) => Promise<void>;

const moveTombstoneDurationMs = 5_000;

export class WorkspaceWatcher {
  readonly #handleChange: ChangeHandler;
  #rootPath: string | null = null;
  #index: WorkspaceIndex | null = null;
  #watcher: FSWatcher | null = null;
  #pollTimer: NodeJS.Timeout | null = null;
  #debounceTimer: NodeJS.Timeout | null = null;
  #isScanning = false;
  #scanRequested = false;
  #forceRefresh = false;
  #generation = 0;
  #removedFiles = new Map<string, RemovedFile>();

  constructor(handleChange: ChangeHandler) {
    this.#handleChange = handleChange;
  }

  async setRoot(rootPath: string | null): Promise<void> {
    if (rootPath === this.#rootPath) return;

    const generation = (this.#generation += 1);
    this.#closeHandles();
    this.#rootPath = rootPath;
    this.#index = null;
    this.#removedFiles.clear();
    if (!rootPath) return;

    try {
      const index = await scanWorkspace(rootPath);
      if (generation !== this.#generation || rootPath !== this.#rootPath) return;
      this.#index = index;
    } catch (error) {
      if (generation !== this.#generation || rootPath !== this.#rootPath) return;
      console.error(`[workspace] Failed to index project files: ${errorMessage(error)}`);
    }

    try {
      const watcher = watch(rootPath, { recursive: true }, (_event, filename) => {
        if (filename && isWaveformSidecar(String(filename))) return;
        this.#scheduleScan(true);
      });
      watcher.on("error", (error) => {
        console.error(`[workspace] File watcher failed: ${error.message}`);
        watcher.close();
        if (this.#watcher === watcher) this.#watcher = null;
      });
      if (generation !== this.#generation || rootPath !== this.#rootPath) {
        watcher.close();
        return;
      }
      this.#watcher = watcher;
    } catch (error) {
      if (generation !== this.#generation || rootPath !== this.#rootPath) return;
      console.error(`[workspace] File watcher could not start: ${errorMessage(error)}`);
    }

    if (generation !== this.#generation || rootPath !== this.#rootPath) return;
    this.#pollTimer = setInterval(() => this.#scheduleScan(false), 1_000);
    this.#pollTimer.unref();
  }

  refresh(): void {
    this.#scheduleScan(true);
  }

  close(): void {
    this.#generation += 1;
    this.#closeHandles();
    this.#rootPath = null;
    this.#index = null;
    this.#removedFiles.clear();
  }

  #scheduleScan(forceRefresh: boolean): void {
    if (!this.#rootPath) return;
    this.#forceRefresh ||= forceRefresh;
    if (this.#isScanning) {
      this.#scanRequested = true;
      return;
    }
    if (this.#debounceTimer) clearTimeout(this.#debounceTimer);
    this.#debounceTimer = setTimeout(() => {
      this.#debounceTimer = null;
      void this.#scan();
    }, 200);
  }

  async #scan(): Promise<void> {
    const rootPath = this.#rootPath;
    if (!rootPath || this.#isScanning) return;
    const generation = this.#generation;
    const forceRefresh = this.#forceRefresh;
    this.#forceRefresh = false;
    this.#isScanning = true;
    try {
      const current = await scanWorkspace(rootPath);
      if (generation !== this.#generation || rootPath !== this.#rootPath) return;
      pruneRemovedFiles(this.#removedFiles);

      const previous = this.#index;
      if (!previous || forceRefresh || previous.fingerprint !== current.fingerprint) {
        const removedFiles = new Map(this.#removedFiles);
        const moves = previous ? detectMovedFiles(previous, current, removedFiles) : [];
        await this.#handleChange(moves);
        if (generation === this.#generation && rootPath === this.#rootPath) {
          this.#index = current;
          this.#removedFiles = removedFiles;
        }
      }
    } catch (error) {
      console.error(`[workspace] Failed to refresh project files: ${errorMessage(error)}`);
    } finally {
      this.#isScanning = false;
      if (this.#scanRequested) {
        this.#scanRequested = false;
        this.#scheduleScan(this.#forceRefresh);
      }
    }
  }

  #closeHandles(): void {
    this.#watcher?.close();
    this.#watcher = null;
    if (this.#pollTimer) clearInterval(this.#pollTimer);
    this.#pollTimer = null;
    if (this.#debounceTimer) clearTimeout(this.#debounceTimer);
    this.#debounceTimer = null;
    this.#scanRequested = false;
    this.#forceRefresh = false;
  }
}

async function scanWorkspace(rootPath: string): Promise<WorkspaceIndex> {
  const byIdentity = new Map<string, IndexedEntry[]>();
  const records: string[] = [];

  async function visit(directoryPath: string, relativeDirectory: string): Promise<void> {
    const entries = await readdir(directoryPath, { withFileTypes: true });
    entries.sort((left, right) => left.name.localeCompare(right.name));
    for (const entry of entries) {
      const absolutePath = path.join(directoryPath, entry.name);
      const relativePath = relativeDirectory
        ? path.posix.join(relativeDirectory, entry.name)
        : entry.name;
      if (isWaveformSidecar(relativePath)) continue;
      let stats;
      try {
        stats = await lstat(absolutePath, { bigint: true });
      } catch (error) {
        if (isMissingFileError(error)) continue;
        throw error;
      }

      const isDirectory = stats.isDirectory();
      let isFile = stats.isFile();
      if (stats.isSymbolicLink()) {
        try {
          isFile = (await stat(absolutePath)).isFile();
        } catch (error) {
          if (!isMissingFileError(error)) throw error;
        }
      }
      const kind = isDirectory ? "directory" : isFile ? "file" : "other";
      const identity = fileIdentity(stats);
      records.push(`${kind}:${relativePath}:${identity}`);
      if (isFile) {
        const indexed = byIdentity.get(identity) ?? [];
        indexed.push({ path: relativePath });
        byIdentity.set(identity, indexed);
      } else if (isDirectory) {
        await visit(absolutePath, relativePath);
      }
    }
  }

  await visit(rootPath, "");
  records.sort();
  return { byIdentity, fingerprint: records.join("\n") };
}

function isWaveformSidecar(filePath: string): boolean {
  return filePath.includes(".khs-waveform");
}

function detectMovedFiles(
  previous: WorkspaceIndex,
  current: WorkspaceIndex,
  removedFiles: Map<string, RemovedFile>,
): WorkspaceFileMove[] {
  const moves: WorkspaceFileMove[] = [];
  const now = Date.now();
  pruneRemovedFiles(removedFiles, now);

  const movedIdentities = new Set<string>();
  for (const [identity, currentEntries] of current.byIdentity) {
    const previousEntries = previous.byIdentity.get(identity);
    if (!previousEntries) continue;
    const previousPaths = new Set(previousEntries.map((entry) => entry.path));
    const currentPaths = new Set(currentEntries.map((entry) => entry.path));
    const removed = previousEntries.filter((entry) => !currentPaths.has(entry.path));
    const added = currentEntries.filter((entry) => !previousPaths.has(entry.path));
    if (removed.length === 1 && added.length === 1) {
      const [removedEntry] = removed;
      const [addedEntry] = added;
      if (removedEntry && addedEntry) {
        moves.push({ sourcePath: removedEntry.path, destinationPath: addedEntry.path });
        movedIdentities.add(identity);
        removedFiles.delete(identity);
      }
    }
  }

  for (const [identity, previousEntries] of previous.byIdentity) {
    if (movedIdentities.has(identity)) continue;
    const currentPaths = new Set(
      (current.byIdentity.get(identity) ?? []).map((entry) => entry.path),
    );
    const removed = previousEntries.filter((entry) => !currentPaths.has(entry.path));
    if (removed.length === 1 && removed[0]) {
      removedFiles.set(identity, {
        path: removed[0].path,
        expiresAt: now + moveTombstoneDurationMs,
      });
    }
  }

  for (const [identity, currentEntries] of current.byIdentity) {
    if (movedIdentities.has(identity)) continue;
    const removed = removedFiles.get(identity);
    if (!removed || currentEntries.length !== 1 || !currentEntries[0]) continue;
    const [currentEntry] = currentEntries;
    removedFiles.delete(identity);
    if (currentEntry.path !== removed.path) {
      moves.push({ sourcePath: removed.path, destinationPath: currentEntry.path });
    }
  }
  return moves;
}

function pruneRemovedFiles(removedFiles: Map<string, RemovedFile>, now = Date.now()): void {
  for (const [identity, removed] of removedFiles) {
    if (removed.expiresAt <= now) removedFiles.delete(identity);
  }
}

function fileIdentity(stats: BigIntStats): string {
  return `${stats.dev.toString()}:${stats.ino.toString()}:${stats.birthtimeNs.toString()}:${stats.size.toString()}:${stats.mtimeNs.toString()}`;
}

function isMissingFileError(error: unknown): boolean {
  return error instanceof Error && "code" in error && error.code === "ENOENT";
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
