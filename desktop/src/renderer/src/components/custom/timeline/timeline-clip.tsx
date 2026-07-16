import { useEffect, useRef, useState, type PointerEvent } from "react";
import { Ellipsis } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuRadioGroup,
  ContextMenuRadioItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { cn } from "@/lib/utils";
import type { TimelineResizeMode, TimelineTool } from "@/stores/timeline-store";
import type {
  SaveTimelineClipParams,
  SetTimelineClipPropertiesParams,
  WorkspaceSnapshot,
} from "@shared/ipc";

import { AudioClipSettingsDialog } from "./audio-clip-settings-dialog";
import { ClipWaveform } from "./clip-waveform";

type TimelineClipData = WorkspaceSnapshot["timeline"]["tracks"][number]["clips"][number];
type DragMode = "move" | "trim-start" | "trim-end";

interface ClipDraft {
  startTick: number;
  durationTicks: number;
  sourceOffsetTicks: number;
  sourceDurationTicks: number;
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
  tool: TimelineTool;
  resizeMode: TimelineResizeMode;
  onSelect(id: string): void;
  onCommit(params: SaveTimelineClipParams): Promise<boolean>;
  onSetProperties(params: SetTimelineClipPropertiesParams): Promise<boolean>;
  onDelete(id: string): void;
  onSplit(id: string, splitTick: number): Promise<boolean>;
}

export function TimelineClip({
  clip,
  trackId,
  pixelsPerTick,
  gridTicks,
  sourceDurationTicks,
  selected,
  tool,
  resizeMode,
  onSelect,
  onCommit,
  onSetProperties,
  onDelete,
  onSplit,
}: TimelineClipProps) {
  const element = useRef<HTMLDivElement>(null);
  const drag = useRef<DragState | null>(null);
  const draft = useRef<ClipDraft | null>(null);
  const animationFrame = useRef<number | null>(null);
  const highlightedLane = useRef<HTMLElement | null>(null);
  const cutPreview = useRef<HTMLDivElement>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);

  useEffect(() => {
    resetVisual();
    hideCutPreview();
  }, [clip.durationTicks, clip.startTick, pixelsPerTick, tool, trackId]);

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

  function cutTickAt(clientX: number, lane: HTMLElement, snapEnabled: boolean) {
    return snap((clientX - lane.getBoundingClientRect().left) / pixelsPerTick, snapEnabled);
  }

  function hideCutPreview() {
    if (cutPreview.current) cutPreview.current.style.opacity = "0";
  }

  function renderCutPreview(event: PointerEvent<HTMLDivElement>) {
    const marker = cutPreview.current;
    const lane = event.currentTarget.closest<HTMLElement>("[data-timeline-track-id]");
    if (!marker || !lane) return;
    const splitTick = cutTickAt(event.clientX, lane, !event.altKey);
    if (splitTick <= clip.startTick || splitTick >= clip.startTick + clip.durationTicks) {
      hideCutPreview();
      return;
    }
    marker.style.left = `${(splitTick - clip.startTick) * pixelsPerTick}px`;
    marker.style.opacity = "1";
  }

  function draftAt(active: DragState, clientX: number, snapEnabled: boolean): ClipDraft {
    const deltaTicks = (clientX - active.originClientX) / pixelsPerTick;
    const originalEnd = active.startTick + active.durationTicks;

    if (active.mode === "move") {
      return {
        startTick: Math.max(0, snap(active.startTick + deltaTicks, snapEnabled)),
        durationTicks: active.durationTicks,
        sourceOffsetTicks: active.sourceOffsetTicks,
        sourceDurationTicks: active.sourceDurationTicks,
      };
    }

    if (active.mode === "trim-end") {
      const minimumDuration =
        resizeMode === "stretch" ? Math.max(1, Math.ceil(active.sourceDurationTicks / 4)) : 1;
      const maximumDuration =
        resizeMode === "stretch"
          ? active.sourceDurationTicks * 4
          : sourceDurationTicks === null
            ? Number.POSITIVE_INFINITY
            : Math.floor(
                ((sourceDurationTicks - active.sourceOffsetTicks) * active.durationTicks) /
                  active.sourceDurationTicks,
              );
      const maximumEnd = active.startTick + maximumDuration;
      const endTick = Math.min(
        maximumEnd,
        Math.max(active.startTick + minimumDuration, snap(originalEnd + deltaTicks, snapEnabled)),
      );
      return {
        startTick: active.startTick,
        durationTicks: endTick - active.startTick,
        sourceOffsetTicks: active.sourceOffsetTicks,
        sourceDurationTicks:
          resizeMode === "stretch"
            ? active.sourceDurationTicks
            : Math.max(
                1,
                Math.round(
                  (active.sourceDurationTicks * (endTick - active.startTick)) /
                    active.durationTicks,
                ),
              ),
      };
    }

    const minimumStart =
      resizeMode === "stretch" || !clip.sourcePath
        ? resizeMode === "stretch"
          ? Math.max(0, originalEnd - active.sourceDurationTicks * 4)
          : 0
        : Math.max(
            0,
            active.startTick -
              Math.floor(
                (active.sourceOffsetTicks * active.durationTicks) / active.sourceDurationTicks,
              ),
          );
    const nextStart = Math.min(
      originalEnd -
        (resizeMode === "stretch" ? Math.max(1, Math.ceil(active.sourceDurationTicks / 4)) : 1),
      Math.max(minimumStart, snap(active.startTick + deltaTicks, snapEnabled)),
    );
    const nextDuration = originalEnd - nextStart;
    const nextSourceDuration =
      resizeMode === "stretch"
        ? active.sourceDurationTicks
        : Math.max(
            1,
            Math.round((active.sourceDurationTicks * nextDuration) / active.durationTicks),
          );
    return {
      startTick: nextStart,
      durationTicks: nextDuration,
      sourceOffsetTicks: !clip.sourcePath
        ? 0
        : resizeMode === "stretch"
          ? active.sourceOffsetTicks
          : active.sourceOffsetTicks + active.sourceDurationTicks - nextSourceDuration,
      sourceDurationTicks: nextSourceDuration,
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
    if (tool === "cut") {
      event.preventDefault();
      event.stopPropagation();
      const splitTick = cutTickAt(event.clientX, lane, !event.altKey);
      if (splitTick <= clip.startTick || splitTick >= clip.startTick + clip.durationTicks) return;
      hideCutPreview();
      onSelect(clip.id);
      void onSplit(clip.id, splitTick);
      return;
    }
    const edge = (event.target as HTMLElement).closest<HTMLElement>("[data-edge]")?.dataset.edge;
    const mode: DragMode = edge === "start" ? "trim-start" : edge === "end" ? "trim-end" : "move";
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    onSelect(clip.id);
    const initial = {
      startTick: clip.startTick,
      durationTicks: clip.durationTicks,
      sourceOffsetTicks: clip.sourceOffsetTicks,
      sourceDurationTicks: clip.sourceDurationTicks,
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
    if (tool === "cut") {
      renderCutPreview(event);
      return;
    }
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
        next.sourceDurationTicks !== active.sourceDurationTicks ||
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
        next.sourceDurationTicks !== clip.sourceDurationTicks ||
        active.targetTrackId !== trackId)
    ) {
      void onCommit({
        id: clip.id,
        trackId: active.targetTrackId,
        name: clip.name,
        resizeMode,
        ...next,
      }).then((success) => {
        if (!success) resetVisual();
      });
    } else {
      resetVisual();
    }
  }

  function cancelDrag(event: PointerEvent<HTMLDivElement>) {
    hideCutPreview();
    if (!drag.current) return;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    drag.current = null;
    draft.current = null;
    setDropTarget(null);
    resetVisual();
  }

  function setProperties(
    update: Partial<Pick<SetTimelineClipPropertiesParams, "stretchMode">> = {},
    makeUnique = false,
  ) {
    return onSetProperties({
      id: clip.id,
      stretchMode: update.stretchMode ?? clip.stretchMode,
      gainDb: clip.gainDb,
      pan: clip.pan,
      pitchSemitones: clip.pitchSemitones,
      tempoPercent: null,
      makeUnique,
    });
  }

  return (
    <>
      <ContextMenu onOpenChange={(open) => open && onSelect(clip.id)}>
        <ContextMenuTrigger asChild>
          <div
            ref={element}
            role="button"
            tabIndex={0}
            aria-label={`${clip.name}, starts at tick ${clip.startTick}`}
            aria-pressed={selected}
            className={cn(
              "group absolute inset-y-0 min-w-4 touch-none overflow-hidden border shadow-sm outline-none data-[dragging]:cursor-grabbing",
              "focus-visible:z-20 focus-visible:ring-2 focus-visible:ring-ring/70",
              tool === "cut" ? "cursor-crosshair" : "cursor-grab",
              selected
                ? "border-primary-foreground/30 bg-primary text-primary-foreground"
                : "border-border bg-secondary text-secondary-foreground hover:border-foreground/30",
            )}
            style={{
              left: clip.startTick * pixelsPerTick,
              width: Math.max(16, clip.durationTicks * pixelsPerTick),
            }}
            onPointerDown={handlePointerDown}
            onPointerMove={handlePointerMove}
            onPointerLeave={hideCutPreview}
            onPointerUp={finishDrag}
            onPointerCancel={cancelDrag}
            onFocus={() => onSelect(clip.id)}
          >
            {clip.sourcePath && sourceDurationTicks ? (
              <ClipWaveform
                peaks={clip.waveform}
                sourceOffsetTicks={clip.sourceOffsetTicks}
                durationTicks={clip.sourceDurationTicks}
                sourceDurationTicks={sourceDurationTicks}
              />
            ) : null}
            {tool === "cut" ? (
              <div
                ref={cutPreview}
                aria-hidden="true"
                className="pointer-events-none absolute inset-y-0 z-30 w-0.5 -translate-x-1/2 bg-destructive opacity-0 shadow-sm"
              />
            ) : null}
            {tool === "select" ? (
              <div
                data-edge="start"
                className="absolute inset-y-0 left-0 z-20 w-2 cursor-ew-resize border-l-2 border-transparent group-hover:border-current/40"
              />
            ) : null}
            <div className="absolute inset-x-0 top-0 z-10 flex h-6 min-w-0 items-center gap-1 border-b border-current/15 bg-current/10 pl-2">
              <span className="pointer-events-none min-w-0 flex-1 truncate text-[11px] font-medium">
                {clip.name}
              </span>
              <Button
                size="icon-xs"
                variant="ghost"
                className="size-5 hover:bg-background/20 hover:text-current"
                aria-label={`Open ${clip.name} menu`}
                onPointerDown={(event) => event.stopPropagation()}
                onClick={(event) => {
                  event.stopPropagation();
                  const bounds = event.currentTarget.getBoundingClientRect();
                  element.current?.dispatchEvent(
                    new MouseEvent("contextmenu", {
                      bubbles: true,
                      clientX: bounds.right,
                      clientY: bounds.bottom,
                    }),
                  );
                }}
              >
                <Ellipsis />
              </Button>
            </div>
            <span className="pointer-events-none absolute right-2 bottom-1 left-2 truncate font-mono text-[9px] uppercase tracking-[0.14em] opacity-50">
              {clip.sourcePath
                ? `${clip.stretchMode} / ${Math.round(clip.tempoPercent)}%`
                : "Empty region"}
            </span>
            {tool === "select" ? (
              <div
                data-edge="end"
                className="absolute inset-y-0 right-0 z-20 w-2 cursor-ew-resize border-r-2 border-transparent group-hover:border-current/40"
              />
            ) : null}
          </div>
        </ContextMenuTrigger>
        <ContextMenuContent className="w-56">
          <ContextMenuLabel className="truncate text-xs text-muted-foreground">
            {clip.name}
          </ContextMenuLabel>
          {clip.sourcePath ? (
            <>
              <ContextMenuLabel className="pb-0 text-[10px] uppercase tracking-wider text-muted-foreground">
                Time stretching
              </ContextMenuLabel>
              <ContextMenuRadioGroup
                value={clip.stretchMode}
                onValueChange={(stretchMode) =>
                  void setProperties({
                    stretchMode: stretchMode as SetTimelineClipPropertiesParams["stretchMode"],
                  })
                }
              >
                <ContextMenuRadioItem value="resample">Resample</ContextMenuRadioItem>
                <ContextMenuRadioItem value="stretch">Stretch</ContextMenuRadioItem>
              </ContextMenuRadioGroup>
              <ContextMenuSeparator />
              <ContextMenuItem
                onSelect={() => window.requestAnimationFrame(() => setSettingsOpen(true))}
              >
                More...
              </ContextMenuItem>
              <ContextMenuItem
                disabled={clip.isUnique}
                onSelect={() => void setProperties({}, true)}
              >
                {clip.isUnique ? "Unique settings" : "Make unique"}
              </ContextMenuItem>
              <ContextMenuSeparator />
            </>
          ) : null}
          <ContextMenuItem variant="destructive" onSelect={() => onDelete(clip.id)}>
            Delete clip
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      {clip.sourcePath ? (
        <AudioClipSettingsDialog
          clip={clip}
          open={settingsOpen}
          onOpenChange={setSettingsOpen}
          onSave={onSetProperties}
        />
      ) : null}
    </>
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
