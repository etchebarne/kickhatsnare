import { useEffect, useRef } from "react";

import { useAppStore } from "@/stores/app-store";
import { useTransportStore } from "@/stores/transport-store";

interface TimelinePlayheadProps {
  leftOffset: number;
  pixelsPerTick: number;
}

export function TimelinePlayhead({ leftOffset, pixelsPerTick }: TimelinePlayheadProps) {
  const element = useRef<HTMLDivElement>(null);
  const transport = useTransportStore((state) => state.transport);
  const timeline = useAppStore((state) => state.workspace?.timeline ?? null);

  useEffect(() => {
    if (!timeline) return;
    const activeTimeline = timeline;
    const receivedAt = performance.now();
    let frame: number;
    function update(now: number) {
      let tick = transport.positionTick;
      if (transport.state === "playing") {
        tick +=
          (((now - receivedAt) / 1_000) * (activeTimeline.bpm * activeTimeline.ticksPerQuarter)) /
          60;
      }
      if (transport.durationTicks > 0) tick = Math.min(tick, transport.durationTicks);
      if (element.current) {
        element.current.style.transform = `translate3d(${leftOffset + tick * pixelsPerTick}px, 0, 0)`;
      }
      frame = requestAnimationFrame(update);
    }
    frame = requestAnimationFrame(update);
    return () => cancelAnimationFrame(frame);
  }, [leftOffset, pixelsPerTick, timeline, transport]);

  return (
    <div
      ref={element}
      className="pointer-events-none absolute inset-y-0 left-0 z-20 w-px bg-foreground/80 will-change-transform"
    >
      <div className="absolute -left-1 top-0 size-2 rotate-45 bg-foreground" />
    </div>
  );
}

export function TransportPosition() {
  const transport = useTransportStore((state) => state.transport);
  const timeline = useAppStore((state) => state.workspace?.timeline ?? null);
  if (!timeline) return null;
  const ticksPerBeat = (timeline.ticksPerQuarter * 4) / timeline.timeSignatureDenominator;
  const ticksPerBar = ticksPerBeat * timeline.timeSignatureNumerator;
  const bar = Math.floor(transport.positionTick / ticksPerBar) + 1;
  const withinBar = transport.positionTick % ticksPerBar;
  const beat = Math.floor(withinBar / ticksPerBeat) + 1;
  const tick = Math.round(withinBar % ticksPerBeat);
  return <span>{`${bar}.${beat}.${String(tick).padStart(3, "0")}`}</span>;
}
