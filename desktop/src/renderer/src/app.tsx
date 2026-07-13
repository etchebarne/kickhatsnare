import { useEffect } from "react";

import { ProjectSidebar } from "@/components/custom/project-sidebar";
import { TitleBar } from "@/components/custom/window-controls";
import { Button } from "@/components/ui/button";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { useAppStore } from "@/stores/app-store";

export function App() {
  const serverStatus = useAppStore((state) => state.serverStatus);
  const workspace = useAppStore((state) => state.workspace);
  const operationError = useAppStore((state) => state.operationError);
  const connect = useAppStore((state) => state.connect);
  const newProject = useAppStore((state) => state.newProject);
  const openProject = useAppStore((state) => state.openProject);
  const saveProject = useAppStore((state) => state.saveProject);
  const saveProjectAs = useAppStore((state) => state.saveProjectAs);

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
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [newProject, openProject, saveProject, saveProjectAs, serverStatus, workspace]);

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background text-foreground">
      <TitleBar />
      <SidebarProvider className="relative min-h-0 flex-1 overflow-hidden">
        <ProjectSidebar />
        <SidebarInset className="min-h-0 overflow-hidden">
          <header className="flex h-11 shrink-0 items-center border-b border-border">
            <div className="flex size-11 shrink-0 items-center justify-center border-r border-border">
              <SidebarTrigger />
            </div>
          </header>
          <main className="grid min-h-0 flex-1 place-items-center px-6">
            <section className="w-full max-w-xl border border-border bg-card p-8 shadow-2xl">
              <p className="mb-3 font-mono text-xs uppercase tracking-[0.3em] text-muted-foreground">
                {workspace?.rootPath ? "Project workspace" : "Working in memory"}
              </p>
              <h1 className="text-4xl font-semibold tracking-tight">
                {workspace?.name ?? "KickHatSnare"}
              </h1>
              <p className="mt-4 max-w-md text-sm leading-6 text-muted-foreground">
                {workspace?.rootPath
                  ? `Stored at ${workspace.rootPath}`
                  : "This project is ready to use without choosing a location. Save it when you want to create its portable workspace."}
              </p>
              <div className="mt-8 flex items-center justify-between gap-4 border-t border-border pt-5">
                <div className="flex items-center gap-3">
                  <span className="text-sm text-muted-foreground">Core process</span>
                  <span className="font-mono text-sm capitalize" aria-live="polite">
                    {serverStatus}
                  </span>
                </div>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={serverStatus === "connecting"}
                  onClick={() => void connect()}
                >
                  {serverStatus === "connecting" ? "Checking..." : "Check connection"}
                </Button>
              </div>
              {operationError ? (
                <p
                  className="mt-4 border-l-2 border-destructive pl-3 text-sm text-destructive"
                  role="alert"
                >
                  {operationError}
                </p>
              ) : null}
            </section>
          </main>
        </SidebarInset>
      </SidebarProvider>
    </div>
  );
}
