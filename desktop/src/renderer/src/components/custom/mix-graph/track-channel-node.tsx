import { useEffect, useState } from "react";
import { Handle, Position, type Node, type NodeProps } from "@xyflow/react";

import { Button } from "@/components/ui/button";
import { useAppStore } from "@/stores/app-store";
import type { WorkspaceSnapshot } from "@shared/ipc";

type Track = WorkspaceSnapshot["timeline"]["tracks"][number];
export type TrackChannelNodeType = Node<{ track: Track }, "trackChannel">;

export function TrackChannelNode({ data }: NodeProps<TrackChannelNodeType>) {
  const saveTrack = useAppStore((state) => state.saveTimelineTrack);
  const [gain, setGain] = useState(data.track.gainDb);
  const [pan, setPan] = useState(data.track.pan);

  useEffect(() => setGain(data.track.gainDb), [data.track.gainDb]);
  useEffect(() => setPan(data.track.pan), [data.track.pan]);

  function commit(update: Partial<Track>) {
    const track = data.track;
    void saveTrack({
      id: track.id,
      name: track.name,
      isMuted: update.isMuted ?? track.isMuted,
      isSoloed: update.isSoloed ?? track.isSoloed,
      gainDb: update.gainDb ?? gain,
      pan: update.pan ?? pan,
      isConnected: update.isConnected ?? track.isConnected,
    });
  }

  return (
    <article className="w-64 overflow-hidden rounded-md border border-border bg-card text-card-foreground shadow-xl">
      <header className="flex items-center justify-between border-b border-border px-3 py-2">
        <div className="min-w-0">
          <p className="truncate text-xs font-semibold">{data.track.name}</p>
          <p className="font-mono text-[9px] uppercase tracking-[0.18em] text-muted-foreground">
            Channel
          </p>
        </div>
        <div className="nodrag flex gap-1">
          <Button
            size="icon-xs"
            variant={data.track.isMuted ? "secondary" : "ghost"}
            onClick={() => commit({ isMuted: !data.track.isMuted })}
          >
            M
          </Button>
          <Button
            size="icon-xs"
            variant={data.track.isSoloed ? "secondary" : "ghost"}
            onClick={() => commit({ isSoloed: !data.track.isSoloed })}
          >
            S
          </Button>
        </div>
      </header>
      <div className="nodrag nowheel grid gap-3 p-3">
        <MixSlider
          label="Gain"
          value={gain}
          min={-60}
          max={12}
          step={0.5}
          suffix="dB"
          onChange={setGain}
          onCommit={() => commit({ gainDb: gain })}
        />
        <MixSlider
          label="Pan"
          value={pan}
          min={-1}
          max={1}
          step={0.01}
          suffix={
            pan === 0 ? "C" : pan < 0 ? `L${Math.round(-pan * 100)}` : `R${Math.round(pan * 100)}`
          }
          onChange={setPan}
          onCommit={() => commit({ pan })}
        />
      </div>
      {!data.track.isConnected ? (
        <p className="border-t border-destructive/30 bg-destructive/10 px-3 py-1.5 text-[10px] text-destructive">
          Unrouted
        </p>
      ) : null}
      <Handle
        id="audio-out"
        type="source"
        position={Position.Right}
        className="!size-3 !border-2 !border-card !bg-foreground"
      />
    </article>
  );
}

interface MixSliderProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  suffix: string;
  onChange(value: number): void;
  onCommit(): void;
}

function MixSlider({ label, value, min, max, step, suffix, onChange, onCommit }: MixSliderProps) {
  return (
    <label className="grid gap-1">
      <span className="flex items-center justify-between font-mono text-[10px] uppercase tracking-[0.14em] text-muted-foreground">
        {label}
        <span className="text-foreground">
          {typeof suffix === "string" && suffix === "dB" ? `${value.toFixed(1)} dB` : suffix}
        </span>
      </span>
      <input
        className="accent-foreground"
        type="range"
        value={value}
        min={min}
        max={max}
        step={step}
        onChange={(event) => onChange(Number(event.currentTarget.value))}
        onPointerUp={onCommit}
        onKeyUp={(event) => {
          if (["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) onCommit();
        }}
      />
    </label>
  );
}
