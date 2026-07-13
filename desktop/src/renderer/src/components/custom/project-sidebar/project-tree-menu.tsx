import type { ContextMenuItem as TreeContextMenuItem } from "@pierre/trees";
import { FolderPlus, Pencil, Trash2 } from "lucide-react";

import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from "@/components/ui/context-menu";

interface ProjectTreeMenuProps {
  isRoot: boolean;
  item: TreeContextMenuItem;
  onCreateDirectory(): void;
  onDelete(): void;
  onRename(): void;
}

export function ProjectTreeMenu({
  isRoot,
  item,
  onCreateDirectory,
  onDelete,
  onRename,
}: ProjectTreeMenuProps) {
  return (
    <ContextMenuContent className="min-w-44" onCloseAutoFocus={(event) => event.preventDefault()}>
      <ContextMenuItem onClick={onCreateDirectory}>
        <FolderPlus />
        New folder
      </ContextMenuItem>
      <ContextMenuItem disabled={isRoot} onClick={onRename}>
        <Pencil />
        Rename
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem variant="destructive" disabled={isRoot} onClick={onDelete}>
        <Trash2 />
        Delete {item.kind === "directory" ? "folder" : "file"}
      </ContextMenuItem>
    </ContextMenuContent>
  );
}
