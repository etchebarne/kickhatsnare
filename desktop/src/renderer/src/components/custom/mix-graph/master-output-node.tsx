import { useEffect, useState } from "react";
import { Handle, Position, type Node, type NodeProps } from "@xyflow/react";

import { Button } from "@/components/ui/button";
import { useAppStore } from "@/stores/app-store";
import type { WorkspaceSnapshot } from "@shared/ipc";

type Timeline = WorkspaceSnapshot["timeline"];
type MixNode = Timeline["mixGraph"]["nodes"][number];
export type MasterOutputNodeType = Node<{ timeline: Timeline; mixNode: MixNode }, "masterOutput">;

export function MasterOutputNode({ data }: NodeProps<MasterOutputNodeType>) {
  const setMasterMix = useAppStore((state) => state.setMasterMix);
  const [gain, setGain] = useState(data.timeline.masterGainDb);
  useEffect(() => setGain(data.timeline.masterGainDb), [data.timeline.masterGainDb]);

  function commit(nextGain = gain, muted = data.timeline.isMasterMuted) {
    void setMasterMix({ gainDb: nextGain, isMuted: muted });
  }

  const input = data.mixNode.ports.find((port) => port.direction === "input");

  return (
    <article className="w-56 overflow-hidden rounded-3xl border-2 border-foreground/60 bg-card text-card-foreground shadow-xl">
      <header className="flex items-center justify-between border-b border-border px-4 py-3">
        <div>
          <p className="text-xs font-semibold">Master Output</p>
          <p className="font-mono text-[9px] uppercase tracking-[0.18em] text-muted-foreground">
            Stereo bus
          </p>
        </div>
        <Button
          className="nodrag"
          size="icon-xs"
          variant={data.timeline.isMasterMuted ? "secondary" : "ghost"}
          onClick={() => commit(gain, !data.timeline.isMasterMuted)}
        >
          M
        </Button>
      </header>
      {input ? (
        <div className="relative flex items-center justify-between border-b border-border px-4 py-2 font-mono text-[9px] uppercase tracking-[0.16em] text-muted-foreground">
          <Handle
            id={input.id}
            type="target"
            position={Position.Left}
            className="!left-0 !size-3 !border-2 !border-card !bg-foreground shadow-sm"
          />
          <span>{input.label}</span>
          <span>{input.signalType}</span>
        </div>
      ) : null}
      <label className="nodrag nowheel grid gap-1 px-4 py-3">
        <span className="flex justify-between font-mono text-[10px] uppercase tracking-[0.14em] text-muted-foreground">
          Gain <span>{gain.toFixed(1)} dB</span>
        </span>
        <input
          className="accent-foreground"
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
