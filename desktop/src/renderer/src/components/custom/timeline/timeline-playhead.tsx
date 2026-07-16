import { useEffect, useRef, type RefObject } from "react";

import { useAppStore } from "@/stores/app-store";
import { useTransportStore } from "@/stores/transport-store";

interface TimelinePlayheadProps {
  leftOffset: number;
  markerRef: RefObject<HTMLDivElement | null>;
  pixelsPerTick: number;
}

export function TimelinePlayhead({ leftOffset, markerRef, pixelsPerTick }: TimelinePlayheadProps) {
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
      const timelinePosition = tick * pixelsPerTick;
      if (element.current) {
        element.current.style.transform = `translate3d(${leftOffset + timelinePosition}px, 0, 0)`;
      }
      if (markerRef.current) {
        markerRef.current.style.transform = `translate3d(${timelinePosition}px, 0, 0)`;
      }
      frame = requestAnimationFrame(update);
    }
    frame = requestAnimationFrame(update);
    return () => cancelAnimationFrame(frame);
  }, [leftOffset, markerRef, pixelsPerTick, timeline, transport]);

  return (
    <div
      ref={element}
      className="pointer-events-none absolute inset-y-0 left-0 z-20 w-px bg-foreground/80 will-change-transform"
    />
  );
}
