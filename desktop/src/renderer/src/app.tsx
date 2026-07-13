import { useEffect, useState } from "react";
import { Network, Rows3 } from "lucide-react";

import { MixGraphEditor } from "@/components/custom/mix-graph";
import { ProjectSidebar } from "@/components/custom/project-sidebar";
import {
  TimelineEditor,
  TimelineHeaderControls,
  TimelineTransportControls,
} from "@/components/custom/timeline";
import { TitleBar } from "@/components/custom/window-controls";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { Button } from "@/components/ui/button";
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
      <SidebarProvider className="relative min-h-0 flex-1 overflow-hidden">
        <ProjectSidebar />
        <SidebarInset className="min-h-0 overflow-hidden">
          <header className="flex h-11 shrink-0 items-center border-b border-border bg-card">
            <div className="flex size-11 shrink-0 items-center justify-center border-r border-border">
              <SidebarTrigger />
            </div>
            <TimelineTransportControls />
            <TimelineHeaderControls />
            <div className="ml-auto flex items-center gap-1 px-2">
              <Button
                size="xs"
                variant={view === "arrangement" ? "secondary" : "ghost"}
                onClick={() => setView("arrangement")}
              >
                <Rows3 /> Arrangement
              </Button>
              <Button
                size="xs"
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
        </SidebarInset>
      </SidebarProvider>
    </div>
  );
}

function isEditable(target: EventTarget | null) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || ["INPUT", "SELECT", "TEXTAREA"].includes(target.tagName))
  );
}
