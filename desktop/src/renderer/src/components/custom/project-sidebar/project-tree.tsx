import { useEffect, useRef, useState, type DragEvent, type MouseEvent } from "react";

import type { ContextMenuItem, FileTreeDropResult, FileTreeRenameEvent } from "@pierre/trees";
import { FileTree, useFileTree } from "@pierre/trees/react";

import { ContextMenu, ContextMenuTrigger } from "@/components/ui/context-menu";
import { useAppStore } from "@/stores/app-store";

import { ProjectTreeMenu } from "./project-tree-menu";

const audioExtensions = new Set([
  "aif",
  "aiff",
  "flac",
  "m4a",
  "mp3",
  "oga",
  "ogg",
  "opus",
  "wav",
  "wave",
]);

const treeStyles = `
  [data-file-tree-search-container] {
    box-sizing: border-box;
    flex: 0 0 44px;
    align-items: center;
    margin-bottom: 0;
    padding-inline: 4px;
    border-bottom: 1px solid var(--trees-border-color);
  }

  [data-file-tree-search-input] {
    margin-block: 0;
  }

  [data-file-tree-virtualized-scroll='true'] {
    padding-top: 4px;
  }

  [data-external-audio-drop-target='true'] {
    background: var(--trees-bg-muted);
    box-shadow: inset 0 0 0 1px var(--trees-focus-ring-color);
  }
`;

const contextMenuFocusSettleMs = 200;
const audioDragType = "application/x-kickhatsnare-audio";

interface ProjectTreeProps {
  categoryPaths: string[];
  paths: string[];
  rootPath: string;
  pinnedRoots: PinnedTreeRoot[];
}

interface PinnedTreeRoot {
  id: string;
  rootPath: string;
}

interface DropTarget {
  element: HTMLElement;
  path: string;
}

export function ProjectTree({ categoryPaths, paths, rootPath, pinnedRoots }: ProjectTreeProps) {
  const dropTarget = useRef<DropTarget | null>(null);
  const pendingDirectories = useRef(new Set<string>());
  const menuActionTimer = useRef<number | null>(null);
  const [contextItem, setContextItem] = useState<ContextMenuItem | null>(null);
  const [contextMenuOpen, setContextMenuOpen] = useState(false);
  const createWorkspaceDirectory = useAppStore((state) => state.createWorkspaceDirectory);
  const deleteWorkspaceEntry = useAppStore((state) => state.deleteWorkspaceEntry);
  const importAudioFiles = useAppStore((state) => state.importAudioFiles);
  const moveWorkspaceEntry = useAppStore((state) => state.moveWorkspaceEntry);
  const pinFolder = useAppStore((state) => state.pinFolder);
  const unpinFolder = useAppStore((state) => state.unpinFolder);

  function resetTree() {
    model.resetPaths(paths);
  }

  function handleRename(event: FileTreeRenameEvent) {
    const sourcePath = workspaceRelativeEntry(event.sourcePath, rootPath);
    const destinationPath = workspaceRelativeEntry(event.destinationPath, rootPath);
    if (sourcePath === null || destinationPath === null) {
      resetTree();
      return;
    }

    if (pendingDirectories.current.delete(sourcePath)) {
      void createWorkspaceDirectory(destinationPath).then((success) => {
        if (!success) resetTree();
      });
      return;
    }

    void moveWorkspaceEntry(sourcePath, destinationPath).then((success) => {
      if (!success) resetTree();
    });
  }

  function handleTreeDrop(event: FileTreeDropResult) {
    const targetPath = event.target.directoryPath;
    const targetDirectory = targetPath ? workspaceRelativeDirectory(targetPath, rootPath) : null;
    if (targetDirectory === null) {
      resetTree();
      return;
    }

    const moves = event.draggedPaths.map((sourceTreePath) => {
      const sourcePath = workspaceRelativeEntry(sourceTreePath, rootPath);
      if (sourcePath === null) return Promise.resolve(false);
      const destinationPath = joinWorkspacePath(targetDirectory, treeBasename(sourceTreePath));
      return moveWorkspaceEntry(sourcePath, destinationPath);
    });
    void Promise.all(moves).then((results) => {
      if (results.some((success) => !success)) resetTree();
    });
  }

  const { model } = useFileTree({
    density: "compact",
    dragAndDrop: {
      canDrag: (draggedPaths) =>
        draggedPaths.every((path) => workspaceRelativeEntry(path, rootPath) !== null),
      canDrop: ({ target }) =>
        target.directoryPath !== null &&
        workspaceRelativeDirectory(target.directoryPath, rootPath) !== null,
      onDropComplete: handleTreeDrop,
      onDropError: resetTree,
    },
    fileTreeSearchMode: "hide-non-matches",
    flattenEmptyDirectories: false,
    icons: { colored: false, set: "standard" },
    initialExpansion: 2,
    paths,
    renaming: {
      canRename: ({ path }) => workspaceRelativeEntry(path, rootPath) !== null,
      onRename: handleRename,
    },
    search: true,
    unsafeCSS: treeStyles,
  });

  useEffect(() => model.resetPaths(paths), [model, paths]);
  useEffect(() => {
    return model.onMutation("remove", (event) => {
      const path = workspaceRelativeEntry(event.path, rootPath);
      if (path) pendingDirectories.current.delete(path);
    });
  }, [model, rootPath]);
  useEffect(
    () => () => {
      clearDropTarget(dropTarget);
      if (menuActionTimer.current !== null) window.clearTimeout(menuActionTimer.current);
    },
    [],
  );

  function handleDragEnter(event: DragEvent<HTMLElement>) {
    if (!hasFiles(event)) return;
    event.preventDefault();
    const target = findWorkspaceDropTarget(event, rootPath);
    updateDropTarget(dropTarget, target);
    event.dataTransfer.dropEffect = target ? "copy" : "none";
  }

  function handleDragOver(event: DragEvent<HTMLElement>) {
    if (!hasFiles(event)) return;
    event.preventDefault();
    const target = findWorkspaceDropTarget(event, rootPath);
    updateDropTarget(dropTarget, target);
    event.dataTransfer.dropEffect = target ? "copy" : "none";
  }

  function handleDragLeave(event: DragEvent<HTMLElement>) {
    if (!hasFiles(event)) return;
    const bounds = event.currentTarget.getBoundingClientRect();
    if (
      event.clientX <= bounds.left ||
      event.clientX >= bounds.right ||
      event.clientY <= bounds.top ||
      event.clientY >= bounds.bottom
    ) {
      clearDropTarget(dropTarget);
    }
  }

  function handleDrop(event: DragEvent<HTMLElement>) {
    if (!hasFiles(event)) return;
    event.preventDefault();
    const target = findWorkspaceDropTarget(event, rootPath) ?? dropTarget.current;
    clearDropTarget(dropTarget);
    if (!target) return;
    const targetDirectory = workspaceRelativeDirectory(target.path, rootPath);
    if (targetDirectory === null) return;
    const files = Array.from(event.dataTransfer.files).filter(isAudioFile);
    if (files.length > 0) void importAudioFiles(files, targetDirectory);
  }

  function handleDragStart(event: DragEvent<HTMLDivElement>) {
    const row = event.nativeEvent
      .composedPath()
      .find(
        (element): element is HTMLElement =>
          element instanceof HTMLElement && element.dataset.itemPath !== undefined,
      );
    const treePath = row?.dataset.itemPath;
    if (!treePath || row?.dataset.itemType === "folder") return;
    const relativePath = workspaceRelativeEntry(treePath, rootPath);
    if (!relativePath || !isAudioPath(relativePath)) return;
    event.dataTransfer.setData(audioDragType, relativePath);
    event.dataTransfer.effectAllowed = "copyMove";
  }

  function createDirectoryFromItem(item: ContextMenuItem) {
    const directoryPath = item.kind === "directory" ? item.path : treeParent(item.path);
    const relativePath = workspaceRelativeDirectory(directoryPath, rootPath);
    if (relativePath === null) return;
    const treePath = nextPlaceholderDirectoryPath(model, directoryPath);
    const workspacePath = workspaceRelativeEntry(treePath, rootPath);
    if (workspacePath === null) return;
    pendingDirectories.current.add(workspacePath);
    model.add(`${treePath}/`);
    if (!model.startRenaming(treePath, { removeIfCanceled: true })) {
      model.remove(treePath, { recursive: true });
      return;
    }
    clearRenameInput(model);
  }

  function deleteItem(item: ContextMenuItem) {
    const relativePath = workspaceRelativeEntry(item.path, rootPath);
    if (relativePath === null) return;
    if (!window.confirm(`Delete ${item.name}? This cannot be undone.`)) return;
    void deleteWorkspaceEntry(relativePath);
  }

  function handleContextMenu(event: MouseEvent<HTMLDivElement>) {
    const item = findContextMenuItem(event);
    setContextItem(item);
  }

  function handleContextMenuOpenChange(open: boolean) {
    setContextMenuOpen(open);
  }

  function runAfterContextMenuCloses(action: () => void) {
    setContextMenuOpen(false);
    if (menuActionTimer.current !== null) window.clearTimeout(menuActionTimer.current);
    // Radix keeps focus during its close animation; wait before focusing Pierre's editor.
    menuActionTimer.current = window.setTimeout(() => {
      menuActionTimer.current = null;
      action();
    }, contextMenuFocusSettleMs);
  }

  const pinnedFolderId = contextItem
    ? (pinnedFolderForPath(contextItem.path, pinnedRoots)?.id ?? null)
    : null;
  const isCategory =
    contextItem !== null && categoryPaths.includes(trimDirectorySlash(contextItem.path));

  return (
    <>
      <ContextMenu open={contextMenuOpen} onOpenChange={handleContextMenuOpenChange}>
        <ContextMenuTrigger asChild>
          <div
            className="h-full min-h-0"
            onContextMenuCapture={handleContextMenu}
            onDragStart={handleDragStart}
          >
            <FileTree
              className="project-tree"
              model={model}
              aria-label="Project and pinned files"
              onDragEnter={handleDragEnter}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
            />
          </div>
        </ContextMenuTrigger>
        <ProjectTreeMenu
          item={contextItem}
          isCategory={isCategory}
          isPinned={pinnedFolderId !== null}
          isRoot={
            contextItem !== null && workspaceRelativeEntry(contextItem.path, rootPath) === null
          }
          onCreateDirectory={() => {
            if (contextItem) runAfterContextMenuCloses(() => createDirectoryFromItem(contextItem));
          }}
          onDelete={() => {
            if (contextItem) deleteItem(contextItem);
          }}
          onPinFolder={() => void pinFolder()}
          onRename={() => {
            if (contextItem) runAfterContextMenuCloses(() => model.startRenaming(contextItem.path));
          }}
          onUnpin={() => {
            if (pinnedFolderId !== null) void unpinFolder(pinnedFolderId);
          }}
        />
      </ContextMenu>
    </>
  );
}

function pinnedFolderForPath(path: string, pinnedRoots: PinnedTreeRoot[]): PinnedTreeRoot | null {
  const normalizedPath = trimDirectorySlash(path);
  return (
    pinnedRoots.find(
      (folder) =>
        normalizedPath === folder.rootPath || normalizedPath.startsWith(`${folder.rootPath}/`),
    ) ?? null
  );
}

function findContextMenuItem(event: MouseEvent): ContextMenuItem | null {
  const row = event.nativeEvent
    .composedPath()
    .find(
      (element): element is HTMLElement =>
        element instanceof HTMLElement && element.dataset.itemPath !== undefined,
    );
  const path = row?.dataset.itemPath;
  if (!row || !path) return null;
  return {
    kind: row.dataset.itemType === "folder" ? "directory" : "file",
    name: treeBasename(path),
    path,
  };
}

function findDropTarget(event: DragEvent<HTMLElement>): DropTarget | null {
  const hoveredRow = event.nativeEvent
    .composedPath()
    .find(
      (element): element is HTMLElement =>
        element instanceof HTMLElement && element.dataset.itemPath !== undefined,
    );
  if (!hoveredRow) return null;
  if (hoveredRow.dataset.itemType === "folder") {
    return { element: hoveredRow, path: hoveredRow.dataset.itemPath ?? "" };
  }

  const parentPath = hoveredRow.dataset.itemParentPath;
  const shadowRoot = hoveredRow.getRootNode();
  if (!parentPath || !(shadowRoot instanceof ShadowRoot)) return null;
  const parentRow = Array.from(
    shadowRoot.querySelectorAll<HTMLElement>("[data-item-path][data-item-type='folder']"),
  ).find((element) => element.dataset.itemPath === parentPath && element.role === "treeitem");
  return parentRow ? { element: parentRow, path: parentPath } : null;
}

function findWorkspaceDropTarget(
  event: DragEvent<HTMLElement>,
  rootPath: string,
): DropTarget | null {
  const target = findDropTarget(event);
  return target && workspaceRelativeDirectory(target.path, rootPath) !== null ? target : null;
}

function updateDropTarget(targetRef: { current: DropTarget | null }, target: DropTarget | null) {
  if (targetRef.current?.element === target?.element) return;
  clearDropTarget(targetRef);
  target?.element.setAttribute("data-external-audio-drop-target", "true");
  targetRef.current = target;
}

function clearDropTarget(targetRef: { current: DropTarget | null }) {
  targetRef.current?.element.removeAttribute("data-external-audio-drop-target");
  targetRef.current = null;
}

function workspaceRelativeDirectory(targetPath: string, rootPath: string): string | null {
  const normalizedTarget = trimDirectorySlash(targetPath);
  if (normalizedTarget === rootPath) return "";
  const rootPrefix = `${rootPath}/`;
  return normalizedTarget.startsWith(rootPrefix) ? normalizedTarget.slice(rootPrefix.length) : null;
}

function workspaceRelativeEntry(targetPath: string, rootPath: string): string | null {
  const relativePath = workspaceRelativeDirectory(targetPath, rootPath);
  return relativePath ? relativePath : null;
}

function trimDirectorySlash(path: string): string {
  return path.endsWith("/") ? path.slice(0, -1) : path;
}

function treeParent(path: string): string {
  const normalizedPath = trimDirectorySlash(path);
  const separator = normalizedPath.lastIndexOf("/");
  return separator < 0 ? "" : normalizedPath.slice(0, separator);
}

function treeBasename(path: string): string {
  const normalizedPath = trimDirectorySlash(path);
  return normalizedPath.slice(normalizedPath.lastIndexOf("/") + 1);
}

function joinWorkspacePath(parent: string, name: string): string {
  return parent ? `${parent}/${name}` : name;
}

function nextPlaceholderDirectoryPath(
  model: ReturnType<typeof useFileTree>["model"],
  parentPath: string,
) {
  const normalizedParent = trimDirectorySlash(parentPath);
  for (let suffix = 1; ; suffix += 1) {
    const name = `.kickhatsnare-new-directory-${suffix}`;
    const path = `${normalizedParent}/${name}`;
    if (!model.getItem(path)) return path;
  }
}

function clearRenameInput(model: ReturnType<typeof useFileTree>["model"]) {
  // Pierre exposes rename startup but not the draft value, so clear its rendered editor.
  let attempts = 3;
  function clear() {
    const input = model
      .getFileTreeContainer()
      ?.shadowRoot?.querySelector<HTMLInputElement>("[data-item-rename-input]");
    if (!input) {
      attempts -= 1;
      if (attempts > 0) requestAnimationFrame(clear);
      return;
    }
    input.value = "";
    input.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
    input.focus();
  }
  requestAnimationFrame(clear);
}

function hasFiles(event: DragEvent): boolean {
  return (
    event.dataTransfer.files.length > 0 || Array.from(event.dataTransfer.types).includes("Files")
  );
}

function isAudioFile(file: File): boolean {
  const extension = file.name.split(".").pop()?.toLowerCase();
  return extension !== undefined && audioExtensions.has(extension);
}

function isAudioPath(path: string): boolean {
  const extension = path.split(".").pop()?.toLowerCase();
  return extension !== undefined && audioExtensions.has(extension);
}
