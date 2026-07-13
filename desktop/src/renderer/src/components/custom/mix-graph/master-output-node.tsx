import { useEffect, useState } from "react";
import { Handle, Position, type Node, type NodeProps } from "@xyflow/react";

import { Button } from "@/components/ui/button";
import { useAppStore } from "@/stores/app-store";
import type { WorkspaceSnapshot } from "@shared/ipc";

type Timeline = WorkspaceSnapshot["timeline"];
export type MasterOutputNodeType = Node<{ timeline: Timeline }, "masterOutput">;

export function MasterOutputNode({ data }: NodeProps<MasterOutputNodeType>) {
  const setMasterMix = useAppStore((state) => state.setMasterMix);
  const [gain, setGain] = useState(data.timeline.masterGainDb);
  useEffect(() => setGain(data.timeline.masterGainDb), [data.timeline.masterGainDb]);

  function commit(nextGain = gain, muted = data.timeline.isMasterMuted) {
    void setMasterMix({ gainDb: nextGain, isMuted: muted });
  }

  return (
    <article className="w-64 overflow-hidden rounded-md border border-foreground/30 bg-foreground text-background shadow-2xl">
      <Handle
        id="audio-in"
        type="target"
        position={Position.Left}
        className="!size-3 !border-2 !border-foreground !bg-background"
      />
      <header className="flex items-center justify-between border-b border-background/20 px-3 py-2">
        <div>
          <p className="text-xs font-semibold">Master Output</p>
          <p className="font-mono text-[9px] uppercase tracking-[0.18em] opacity-60">Stereo</p>
        </div>
        <Button
          className="nodrag border-background/20 bg-transparent text-background hover:bg-background/10 hover:text-background"
          size="icon-xs"
          variant="outline"
          onClick={() => commit(gain, !data.timeline.isMasterMuted)}
        >
          M
        </Button>
      </header>
      <label className="nodrag nowheel grid gap-1 p-3">
        <span className="flex justify-between font-mono text-[10px] uppercase tracking-[0.14em] opacity-65">
          Gain <span>{gain.toFixed(1)} dB</span>
        </span>
        <input
          className="accent-background"
          type="range"
          value={gain}
          min={-60}
          max={12}
          step={0.5}
          onChange={(event) => setGain(Number(event.currentTarget.value))}
          onPointerUp={() => commit()}
        />
      </label>
    </article>
  );
}
