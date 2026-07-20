import { useState } from "react";
import { FileQuestion } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useAppStore } from "@/stores/app-store";

export function MissingMediaDialog() {
  const [pendingAction, setPendingAction] = useState<
    "locate" | "leaveEmpty" | "deleteClips" | null
  >(null);
  const workspace = useAppStore((state) => state.workspace);
  const operationError = useAppStore((state) => state.operationError);
  const locateMissingMedia = useAppStore((state) => state.locateMissingMedia);
  const recoverMissingMedia = useAppStore((state) => state.recoverMissingMedia);
  const missing = workspace?.missingMedia[0];
  const remaining = workspace?.missingMedia.length ?? 0;
  const clipCount = missing?.clipIds.length ?? 0;

  async function recover(action: "locate" | "leaveEmpty" | "deleteClips") {
    if (!missing) return;
    setPendingAction(action);
    try {
      if (action === "locate") {
        await locateMissingMedia(missing.sourcePath);
      } else {
        await recoverMissingMedia(missing.sourcePath, action);
      }
    } finally {
      setPendingAction(null);
    }
  }

  return (
    <Dialog open={Boolean(missing)}>
      <DialogContent role="alertdialog" showCloseButton={false} className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileQuestion className="size-5 text-destructive" />
            Audio file not found
          </DialogTitle>
          <DialogDescription className="leading-relaxed">
            {clipCount === 1
              ? "A timeline clip can no longer find its audio file."
              : `${clipCount} timeline clips can no longer find their audio file.`}{" "}
            Locate a replacement inside this project, leave the affected clips empty, or delete
            them.
          </DialogDescription>
        </DialogHeader>

        <div className="rounded-md border bg-muted/30 px-4 py-3">
          <p className="truncate font-mono text-xs">{missing?.sourcePath}</p>
          {remaining > 1 ? (
            <p className="mt-2 text-[11px] text-muted-foreground">
              {remaining - 1} more missing {remaining === 2 ? "file" : "files"} will be handled
              next.
            </p>
          ) : null}
        </div>

        {!workspace?.rootPath ? (
          <p className="text-xs text-muted-foreground">
            Save the project before locating a replacement file. The other recovery options are
            still available.
          </p>
        ) : null}
        {operationError ? (
          <p className="text-xs text-destructive" role="alert">
            {operationError}
          </p>
        ) : null}

        <DialogFooter className="sm:justify-between">
          <Button
            type="button"
            variant="destructive"
            disabled={pendingAction !== null}
            onClick={() => void recover("deleteClips")}
          >
            {pendingAction === "deleteClips" ? "Deleting..." : "Skip and delete clips"}
          </Button>
          <div className="flex flex-col-reverse gap-2 sm:flex-row">
            <Button
              type="button"
              variant="outline"
              disabled={pendingAction !== null}
              onClick={() => void recover("leaveEmpty")}
            >
              {pendingAction === "leaveEmpty" ? "Updating..." : "Skip and leave empty"}
            </Button>
            <Button
              type="button"
              disabled={pendingAction !== null || !workspace?.rootPath}
              onClick={() => void recover("locate")}
            >
              {pendingAction === "locate" ? "Locating..." : "Locate replacement..."}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
