import { useEffect } from "react";

import { Button } from "@/components/ui/button";
import { useAppStore } from "@/stores/app-store";

export function App() {
  const serverStatus = useAppStore((state) => state.serverStatus);
  const connect = useAppStore((state) => state.connect);

  useEffect(() => {
    void connect();
  }, [connect]);

  return (
    <main className="grid min-h-screen place-items-center bg-background px-6 text-foreground">
      <section className="w-full max-w-xl border border-border bg-card p-8 shadow-2xl">
        <p className="mb-3 font-mono text-xs uppercase tracking-[0.3em] text-muted-foreground">
          Digital audio workstation
        </p>
        <h1 className="text-4xl font-semibold tracking-tight">KickHatSnare</h1>
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
      </section>
    </main>
  );
}
