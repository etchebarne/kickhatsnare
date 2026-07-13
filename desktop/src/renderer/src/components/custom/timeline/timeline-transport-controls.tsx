import { useEffect } from "react";
import { Pause, Play, SkipBack, Square } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useAppStore } from "@/stores/app-store";
import { useTransportStore } from "@/stores/transport-store";

export function TimelineTransportControls() {
  const workspace = useAppStore((state) => state.workspace);
  const transport = useTransportStore((state) => state.transport);
  const isPending = useTransportStore((state) => state.isPending);
  const error = useTransportStore((state) => state.error);
  const play = useTransportStore((state) => state.play);
  const pause = useTransportStore((state) => state.pause);
  const stop = useTransportStore((state) => state.stop);
  const seek = useTransportStore((state) => state.seek);
  const refresh = useTransportStore((state) => state.refresh);

  useEffect(() => {
    if (transport.state !== "playing") return;
    let canceled = false;
    let timer: number | undefined;
    async function poll() {
      await refresh();
      if (!canceled) timer = window.setTimeout(poll, 50);
    }
    timer = window.setTimeout(poll, 50);
    return () => {
      canceled = true;
      if (timer !== undefined) window.clearTimeout(timer);
    };
  }, [refresh, transport.state]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.code !== "Space" || isEditable(event.target)) return;
      event.preventDefault();
      void (transport.state === "playing" ? pause() : play());
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [pause, play, transport.state]);

  if (!workspace) return null;
  return (
    <div className="flex items-center gap-1 border-r border-border pr-3">
      <Button
        size="icon-xs"
        variant="ghost"
        aria-label="Return to start"
        disabled={isPending}
        onClick={() => void seek(0)}
      >
        <SkipBack />
      </Button>
      <Button
        size="icon-xs"
        variant="ghost"
        aria-label="Stop"
        disabled={isPending}
        onClick={() => void stop()}
      >
        <Square />
      </Button>
      <Button
        size="icon-sm"
        variant={transport.state === "playing" ? "secondary" : "default"}
        aria-label={transport.state === "playing" ? "Pause" : "Play"}
        disabled={isPending}
        onClick={() => void (transport.state === "playing" ? pause() : play())}
      >
        {transport.state === "playing" ? <Pause /> : <Play />}
      </Button>
      <span
        className="ml-2 min-w-20 truncate font-mono text-[10px] uppercase tracking-[0.12em] text-muted-foreground"
        title={error ?? undefined}
      >
        {error ?? transport.state}
      </span>
    </div>
  );
}

function isEditable(target: EventTarget | null) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || ["INPUT", "SELECT", "TEXTAREA"].includes(target.tagName))
  );
}
