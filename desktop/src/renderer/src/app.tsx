import { useEffect, useState } from "react";
import { Network, Rows3 } from "lucide-react";

import { MixGraphEditor } from "@/components/custom/mix-graph";
import { ProjectSidebar } from "@/components/custom/project-sidebar";
import { SettingsDialog } from "@/components/custom/settings";
import {
  TimelineEditor,
  TimelineHeaderControls,
  TimelineTransportControls,
} from "@/components/custom/timeline";
import { TitleBar } from "@/components/custom/window-controls";
import { Button } from "@/components/ui/button";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable";
import { useAppStore } from "@/stores/app-store";

export function App() {
  const [view, setView] = useState<"arrangement" | "mix">("arrangement");
  const serverStatus = useAppStore((state) => state.serverStatus);
  const workspace = useAppStore((state) => state.workspace);
  const operationError = useAppStore((state) => state.operationError);
  const connect = useAppStore((state) => state.connect);
  const newProject = useAppStore((state) => state.newProject);
  const openProject = useAppStore((state) => state.openProject);
  const redo = useAppStore((state) => state.redo);
  const saveProject = useAppStore((state) => state.saveProject);
  const saveProjectAs = useAppStore((state) => state.saveProjectAs);
  const undo = useAppStore((state) => state.undo);

  useEffect(() => {
    void connect();
  }, [connect]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (!(event.ctrlKey || event.metaKey) || event.altKey) return;

      switch (event.key.toLowerCase()) {
        case "n":
          if (serverStatus !== "ready") return;
          event.preventDefault();
          void newProject();
          break;
        case "o":
          if (serverStatus !== "ready") return;
          event.preventDefault();
          void openProject();
          break;
        case "s":
          if (!workspace) return;
          event.preventDefault();
          void (event.shiftKey ? saveProjectAs() : saveProject());
          break;
        case "y":
          if (isEditable(event.target)) return;
          if (!workspace?.history.canRedo) return;
          event.preventDefault();
          void redo();
          break;
        case "z":
          if (isEditable(event.target)) return;
          if (event.shiftKey) {
            if (!workspace?.history.canRedo) return;
            event.preventDefault();
            void redo();
          } else {
            if (!workspace?.history.canUndo) return;
            event.preventDefault();
            void undo();
          }
          break;
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [newProject, openProject, redo, saveProject, saveProjectAs, serverStatus, undo, workspace]);

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background text-foreground">
      <TitleBar />
      <ResizablePanelGroup className="min-h-0 flex-1">
        <ResizablePanel
          id="project-sidebar"
          defaultSize="16rem"
          minSize="12rem"
          maxSize="30rem"
          collapsedSize={0}
          collapsible
          groupResizeBehavior="preserve-pixel-size"
        >
          <ProjectSidebar />
        </ResizablePanel>
        <ResizableHandle aria-label="Resize project sidebar" />
        <ResizablePanel id="workspace" className="min-w-0">
          <section className="relative flex h-full min-h-0 w-full flex-col overflow-hidden bg-background">
            <header className="flex h-11 shrink-0 items-center border-b border-border bg-card">
              <TimelineTransportControls />
              <TimelineHeaderControls />
              <div className="ml-auto flex items-center gap-1 px-2">
                <Button
                  size="sm"
                  variant={view === "arrangement" ? "secondary" : "ghost"}
                  onClick={() => setView("arrangement")}
                >
                  <Rows3 /> Arrangement
                </Button>
                <Button
                  size="sm"
                  variant={view === "mix" ? "secondary" : "ghost"}
                  onClick={() => setView("mix")}
                >
                  <Network /> Mix
                </Button>
              </div>
              {operationError ? (
                <p className="truncate px-3 text-xs text-destructive" role="alert">
                  {operationError}
                </p>
              ) : null}
            </header>
            <main className="min-h-0 flex-1 overflow-hidden">
              {workspace ? (
                view === "arrangement" ? (
                  <TimelineEditor />
                ) : (
                  <MixGraphEditor />
                )
              ) : (
                <div className="grid h-full place-items-center px-6">
                  <p className="font-mono text-xs uppercase tracking-[0.25em] text-muted-foreground">
                    {serverStatus === "connecting" ? "Connecting to core" : "Core unavailable"}
                  </p>
                </div>
              )}
            </main>
          </section>
        </ResizablePanel>
      </ResizablePanelGroup>
      <SettingsDialog />
    </div>
  );
}

function isEditable(target: EventTarget | null) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || ["INPUT", "SELECT", "TEXTAREA"].includes(target.tagName))
  );
}
