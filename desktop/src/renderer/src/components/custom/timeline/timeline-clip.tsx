import { useEffect, useRef, type PointerEvent } from "react";

import { cn } from "@/lib/utils";
import type { SaveTimelineClipParams, WorkspaceSnapshot } from "@shared/ipc";

import { ClipWaveform } from "./clip-waveform";

type TimelineClipData = WorkspaceSnapshot["timeline"]["tracks"][number]["clips"][number];
type DragMode = "move" | "trim-start" | "trim-end";

interface ClipDraft {
  startTick: number;
  durationTicks: number;
  sourceOffsetTicks: number;
}

interface DragState extends ClipDraft {
  mode: DragMode;
  originClientX: number;
  sourceLaneTop: number;
  targetLaneTop: number;
  targetTrackId: string;
}

interface TimelineClipProps {
  clip: TimelineClipData;
  trackId: string;
  pixelsPerTick: number;
  gridTicks: number;
  sourceDurationTicks: number | null;
  selected: boolean;
  onSelect(id: string): void;
  onCommit(params: SaveTimelineClipParams): Promise<boolean>;
  onDelete(id: string): void;
}

export function TimelineClip({
  clip,
  trackId,
  pixelsPerTick,
  gridTicks,
  sourceDurationTicks,
  selected,
  onSelect,
  onCommit,
  onDelete,
}: TimelineClipProps) {
  const element = useRef<HTMLDivElement>(null);
  const drag = useRef<DragState | null>(null);
  const draft = useRef<ClipDraft | null>(null);
  const animationFrame = useRef<number | null>(null);
  const highlightedLane = useRef<HTMLElement | null>(null);

  useEffect(() => {
    resetVisual();
  }, [clip.durationTicks, clip.startTick, pixelsPerTick, trackId]);

  useEffect(() => {
    return () => {
      if (animationFrame.current !== null) window.cancelAnimationFrame(animationFrame.current);
      highlightedLane.current?.removeAttribute("data-clip-drop-target");
    };
  }, []);

  function resetVisual() {
    const node = element.current;
    if (!node) return;
    node.style.left = `${clip.startTick * pixelsPerTick}px`;
    node.style.width = `${Math.max(16, clip.durationTicks * pixelsPerTick)}px`;
    node.style.removeProperty("transform");
    node.style.removeProperty("will-change");
    node.style.removeProperty("z-index");
    node.removeAttribute("data-dragging");
  }

  function snap(tick: number, enabled: boolean) {
    const rounded = Math.round(tick);
    return enabled ? Math.round(rounded / gridTicks) * gridTicks : rounded;
  }

  function draftAt(active: DragState, clientX: number, snapEnabled: boolean): ClipDraft {
    const deltaTicks = (clientX - active.originClientX) / pixelsPerTick;
    const originalEnd = active.startTick + active.durationTicks;

    if (active.mode === "move") {
      return {
        startTick: Math.max(0, snap(active.startTick + deltaTicks, snapEnabled)),
        durationTicks: active.durationTicks,
        sourceOffsetTicks: active.sourceOffsetTicks,
      };
    }

    if (active.mode === "trim-end") {
      const maximumEnd =
        sourceDurationTicks === null
          ? Number.POSITIVE_INFINITY
          : active.startTick + sourceDurationTicks - active.sourceOffsetTicks;
      const endTick = Math.min(
        maximumEnd,
        Math.max(active.startTick + 1, snap(originalEnd + deltaTicks, snapEnabled)),
      );
      return {
        startTick: active.startTick,
        durationTicks: endTick - active.startTick,
        sourceOffsetTicks: active.sourceOffsetTicks,
      };
    }

    const minimumStart = clip.sourcePath
      ? Math.max(0, active.startTick - active.sourceOffsetTicks)
      : 0;
    const nextStart = Math.min(
      originalEnd - 1,
      Math.max(minimumStart, snap(active.startTick + deltaTicks, snapEnabled)),
    );
    return {
      startTick: nextStart,
      durationTicks: originalEnd - nextStart,
      sourceOffsetTicks: active.sourceOffsetTicks + nextStart - active.startTick,
    };
  }

  function setDropTarget(lane: HTMLElement | null) {
    if (highlightedLane.current === lane) return;
    highlightedLane.current?.removeAttribute("data-clip-drop-target");
    highlightedLane.current = lane;
    lane?.setAttribute("data-clip-drop-target", "");
  }

  function renderDraft(next: ClipDraft, active: DragState) {
    draft.current = next;
    if (animationFrame.current !== null) window.cancelAnimationFrame(animationFrame.current);
    animationFrame.current = window.requestAnimationFrame(() => {
      animationFrame.current = null;
      const node = element.current;
      if (!node) return;
      node.style.willChange = "transform, left, width";
      node.style.zIndex = "50";
      node.setAttribute("data-dragging", "");
      if (active.mode === "move") {
        const x = (next.startTick - clip.startTick) * pixelsPerTick;
        const y = active.targetLaneTop - active.sourceLaneTop;
        node.style.transform = `translate3d(${x}px, ${y}px, 0)`;
      } else {
        node.style.left = `${next.startTick * pixelsPerTick}px`;
        node.style.width = `${Math.max(16, next.durationTicks * pixelsPerTick)}px`;
      }
    });
  }

  function handlePointerDown(event: PointerEvent<HTMLDivElement>) {
    if (event.button !== 0) return;
    const lane = event.currentTarget.closest<HTMLElement>("[data-timeline-track-id]");
    if (!lane) return;
    const edge = (event.target as HTMLElement).closest<HTMLElement>("[data-edge]")?.dataset.edge;
    const mode: DragMode = edge === "start" ? "trim-start" : edge === "end" ? "trim-end" : "move";
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    onSelect(clip.id);
    const initial = {
      startTick: clip.startTick,
      durationTicks: clip.durationTicks,
      sourceOffsetTicks: clip.sourceOffsetTicks,
    };
    const laneTop = lane.getBoundingClientRect().top;
    drag.current = {
      ...initial,
      mode,
      originClientX: event.clientX,
      sourceLaneTop: laneTop,
      targetLaneTop: laneTop,
      targetTrackId: trackId,
    };
    draft.current = initial;
  }

  function handlePointerMove(event: PointerEvent<HTMLDivElement>) {
    const active = drag.current;
    if (!active) return;

    if (active.mode === "move") {
      const targetLane = trackLaneAt(event.clientX, event.clientY);
      if (targetLane) {
        active.targetTrackId = targetLane.dataset.timelineTrackId ?? trackId;
        active.targetLaneTop = targetLane.getBoundingClientRect().top;
        setDropTarget(targetLane);
      }
    }
    renderDraft(draftAt(active, event.clientX, !event.altKey), active);
  }

  function finishDrag(event: PointerEvent<HTMLDivElement>) {
    const active = drag.current;
    if (!active) return;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    let next = draft.current;
    const draftChanged =
      next &&
      (next.startTick !== active.startTick ||
        next.durationTicks !== active.durationTicks ||
        next.sourceOffsetTicks !== active.sourceOffsetTicks ||
        active.targetTrackId !== trackId);
    if (draftChanged) next = draftAt(active, event.clientX, !event.altKey);
    drag.current = null;
    draft.current = null;
    setDropTarget(null);
    if (
      next &&
      (next.startTick !== clip.startTick ||
        next.durationTicks !== clip.durationTicks ||
        next.sourceOffsetTicks !== clip.sourceOffsetTicks ||
        active.targetTrackId !== trackId)
    ) {
      void onCommit({
        id: clip.id,
        trackId: active.targetTrackId,
        name: clip.name,
        ...next,
      }).then((success) => {
        if (!success) resetVisual();
      });
    } else {
      resetVisual();
    }
  }

  function cancelDrag(event: PointerEvent<HTMLDivElement>) {
    if (!drag.current) return;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    drag.current = null;
    draft.current = null;
    setDropTarget(null);
    resetVisual();
  }

  return (
    <div
      ref={element}
      role="button"
      tabIndex={0}
      aria-label={`${clip.name}, starts at tick ${clip.startTick}`}
      aria-pressed={selected}
      className={cn(
        "group absolute top-2 min-w-4 touch-none overflow-hidden rounded-sm border shadow-sm outline-none data-[dragging]:cursor-grabbing",
        "focus-visible:ring-2 focus-visible:ring-ring/70",
        selected
          ? "border-primary-foreground/30 bg-primary text-primary-foreground"
          : "border-border bg-secondary text-secondary-foreground hover:border-foreground/30",
      )}
      style={{
        left: clip.startTick * pixelsPerTick,
        width: Math.max(16, clip.durationTicks * pixelsPerTick),
        height: "calc(var(--timeline-track-height) - 16px)",
      }}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={finishDrag}
      onPointerCancel={cancelDrag}
      onFocus={() => onSelect(clip.id)}
      onContextMenu={(event) => {
        event.preventDefault();
        onDelete(clip.id);
      }}
    >
      {clip.sourcePath && sourceDurationTicks ? (
        <ClipWaveform
          peaks={clip.waveform}
          sourceOffsetTicks={clip.sourceOffsetTicks}
          durationTicks={clip.durationTicks}
          sourceDurationTicks={sourceDurationTicks}
        />
      ) : null}
      <div
        data-edge="start"
        className="absolute inset-y-0 left-0 z-10 w-2 cursor-ew-resize border-l-2 border-transparent group-hover:border-current/40"
      />
      <div className="pointer-events-none flex h-full flex-col justify-between px-3 py-2">
        <span className="truncate text-xs font-medium">{clip.name}</span>
        <span className="font-mono text-[10px] uppercase tracking-[0.18em] opacity-55">
          {clip.sourcePath
            ? `${clip.sourceSampleRate} Hz / ${clip.sourceChannels === 1 ? "mono" : "stereo"}`
            : "Empty region"}
        </span>
      </div>
      <div
        data-edge="end"
        className="absolute inset-y-0 right-0 z-10 w-2 cursor-ew-resize border-r-2 border-transparent group-hover:border-current/40"
      />
    </div>
  );
}

function trackLaneAt(x: number, y: number) {
  return document
    .elementsFromPoint(x, y)
    .find(
      (candidate): candidate is HTMLElement =>
        candidate instanceof HTMLElement && candidate.dataset.timelineTrackId !== undefined,
    );
}
