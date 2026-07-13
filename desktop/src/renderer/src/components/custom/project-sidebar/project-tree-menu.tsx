import type { ContextMenuItem as TreeContextMenuItem } from "@pierre/trees";
import { FolderPlus, Pencil, Pin, PinOff, Trash2 } from "lucide-react";

import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from "@/components/ui/context-menu";

interface ProjectTreeMenuProps {
  isCategory: boolean;
  isRoot: boolean;
  isPinned: boolean;
  item: TreeContextMenuItem | null;
  onCreateDirectory(): void;
  onDelete(): void;
  onPinFolder(): void;
  onRename(): void;
  onUnpin(): void;
}

export function ProjectTreeMenu({
  isCategory,
  isRoot,
  isPinned,
  item,
  onCreateDirectory,
  onDelete,
  onPinFolder,
  onRename,
  onUnpin,
}: ProjectTreeMenuProps) {
  return (
    <ContextMenuContent className="min-w-44" onCloseAutoFocus={(event) => event.preventDefault()}>
      {item && !isPinned && !isCategory ? (
        <>
          <ContextMenuItem onClick={onCreateDirectory}>
            <FolderPlus />
            New folder
          </ContextMenuItem>
          <ContextMenuItem disabled={isRoot} onClick={onRename}>
            <Pencil />
            Rename
          </ContextMenuItem>
          <ContextMenuSeparator />
        </>
      ) : null}
      <ContextMenuItem onClick={onPinFolder}>
        <Pin />
        Add pinned folder...
      </ContextMenuItem>
      {item && !isCategory ? <ContextMenuSeparator /> : null}
      {item && isPinned ? (
        <ContextMenuItem variant="destructive" onClick={onUnpin}>
          <PinOff />
          Unpin folder
        </ContextMenuItem>
      ) : null}
      {item && !isPinned && !isCategory ? (
        <ContextMenuItem variant="destructive" disabled={isRoot} onClick={onDelete}>
          <Trash2 />
          Delete {item.kind === "directory" ? "folder" : "file"}
        </ContextMenuItem>
      ) : null}
    </ContextMenuContent>
  );
}
