import { useEffect, useRef, useState, type CSSProperties, type MouseEvent } from "react";
import { Plus, Trash2 } from "lucide-react";

import { TimelineClip } from "./timeline-clip";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/app-store";
import type { SaveTimelineClipParams, WorkspaceSnapshot } from "@shared/ipc";

type Timeline = WorkspaceSnapshot["timeline"];
type TimelineTrack = Timeline["tracks"][number];

const TRACK_HEADER_WIDTH = 208;
const RULER_HEIGHT = 32;
const DEFAULT_BARS = 32;
const MIN_HORIZONTAL_ZOOM = 24;
const MAX_HORIZONTAL_ZOOM = 768;
const MIN_TRACK_HEIGHT = 44;
const MAX_TRACK_HEIGHT = 160;

export function TimelineEditor() {
  const workspace = useAppStore((state) => state.workspace);
  const saveTimelineTrack = useAppStore((state) => state.saveTimelineTrack);
  const deleteTimelineTrack = useAppStore((state) => state.deleteTimelineTrack);
  const saveTimelineClip = useAppStore((state) => state.saveTimelineClip);
  const deleteTimelineClip = useAppStore((state) => state.deleteTimelineClip);
  const scrollContainer = useRef<HTMLDivElement>(null);
  const trackHeight = useRef(80);
  const horizontalZoom = useRef(96);
  const [pixelsPerQuarter, setPixelsPerQuarter] = useState(96);
  const [playheadTick, setPlayheadTick] = useState(0);
  const [selectedClipId, setSelectedClipId] = useState<string | null>(null);
  const timelineForZoom = workspace?.timeline;

  useEffect(() => {
    const container = scrollContainer.current;
    if (!container || !timelineForZoom) return;
    const activeContainer = container;
    const activeTimeline = timelineForZoom;
    let horizontalAnimationFrame: number | undefined;
    let verticalAnimationFrame: number | undefined;
    let pendingAnchor: { tick: number; pointerX: number } | undefined;

    function handleWheel(event: WheelEvent) {
      if (!event.ctrlKey && !event.altKey) return;
      event.preventDefault();

      if (event.altKey && !event.ctrlKey) {
        const bounds = activeContainer.getBoundingClientRect();
        const pointerY = event.clientY - bounds.top;
        const anchorTrack = Math.max(
          0,
          (activeContainer.scrollTop + pointerY - RULER_HEIGHT) / trackHeight.current,
        );
        const nextHeight = Math.max(
          MIN_TRACK_HEIGHT,
          Math.min(MAX_TRACK_HEIGHT, trackHeight.current - event.deltaY * 0.12),
        );
        if (nextHeight === trackHeight.current) return;
        trackHeight.current = nextHeight;
        activeContainer.style.setProperty("--timeline-track-height", `${nextHeight}px`);
        if (verticalAnimationFrame !== undefined) {
          window.cancelAnimationFrame(verticalAnimationFrame);
        }
        verticalAnimationFrame = window.requestAnimationFrame(() => {
          verticalAnimationFrame = undefined;
          activeContainer.scrollTop = Math.max(
            0,
            anchorTrack * nextHeight + RULER_HEIGHT - pointerY,
          );
        });
        return;
      }

      const bounds = activeContainer.getBoundingClientRect();
      const pointerX = event.clientX - bounds.left;
      const currentPixelsPerTick = pixelsPerQuarter / activeTimeline.ticksPerQuarter;
      const anchorTick = Math.max(
        0,
        (activeContainer.scrollLeft + pointerX - TRACK_HEADER_WIDTH) / currentPixelsPerTick,
      );
      const nextZoom = Math.max(
        MIN_HORIZONTAL_ZOOM,
        Math.min(MAX_HORIZONTAL_ZOOM, horizontalZoom.current * Math.exp(-event.deltaY * 0.002)),
      );
      if (nextZoom === horizontalZoom.current) return;
      horizontalZoom.current = nextZoom;
      pendingAnchor = { tick: anchorTick, pointerX };
      if (horizontalAnimationFrame !== undefined) return;
      horizontalAnimationFrame = window.requestAnimationFrame(() => {
        horizontalAnimationFrame = undefined;
        const zoom = horizontalZoom.current;
        const anchor = pendingAnchor;
        setPixelsPerQuarter(zoom);
        if (!anchor) return;
        activeContainer.scrollLeft = Math.max(
          0,
          anchor.tick * (zoom / activeTimeline.ticksPerQuarter) +
            TRACK_HEADER_WIDTH -
            anchor.pointerX,
        );
      });
    }

    activeContainer.addEventListener("wheel", handleWheel, { passive: false });
    return () => {
      activeContainer.removeEventListener("wheel", handleWheel);
      if (horizontalAnimationFrame !== undefined) {
        window.cancelAnimationFrame(horizontalAnimationFrame);
      }
      if (verticalAnimationFrame !== undefined) {
        window.cancelAnimationFrame(verticalAnimationFrame);
      }
    };
  }, [pixelsPerQuarter, timelineForZoom]);

  if (!workspace) return null;
  const { timeline } = workspace;
  const ticksPerBeat = (timeline.ticksPerQuarter * 4) / timeline.timeSignatureDenominator;
  const ticksPerBar = ticksPerBeat * timeline.timeSignatureNumerator;
  const pixelsPerTick = pixelsPerQuarter / timeline.ticksPerQuarter;
  const automaticGrid = gridForZoom(timeline.ticksPerQuarter, pixelsPerQuarter);
  const gridTicks = automaticGrid.ticks;
  const maxClipEnd = timeline.tracks.reduce(
    (maximum, track) =>
      Math.max(maximum, ...track.clips.map((clip) => clip.startTick + clip.durationTicks)),
    0,
  );
  const totalBars = Math.max(DEFAULT_BARS, Math.ceil(maxClipEnd / ticksPerBar) + 4);
  const totalTicks = totalBars * ticksPerBar;
  const timelineWidth = totalTicks * pixelsPerTick;
  const playheadLeft = playheadTick * pixelsPerTick;
  const laneStyle = timelineGridStyle(gridTicks, ticksPerBeat, ticksPerBar, pixelsPerTick);

  function snapTick(tick: number) {
    const bounded = Math.max(0, Math.min(Math.round(tick), totalTicks));
    return timeline.isSnapEnabled ? Math.round(bounded / gridTicks) * gridTicks : bounded;
  }

  function positionPlayhead(event: MouseEvent<HTMLElement>) {
    const bounds = event.currentTarget.getBoundingClientRect();
    setPlayheadTick(snapTick((event.clientX - bounds.left) / pixelsPerTick));
  }

  function addTrack() {
    void saveTimelineTrack({
      id: null,
      name: `Track ${timeline.tracks.length + 1}`,
      isMuted: false,
      isSoloed: false,
    });
  }

  function saveTrack(track: TimelineTrack, update: Partial<TimelineTrack>) {
    void saveTimelineTrack({
      id: track.id,
      name: update.name ?? track.name,
      isMuted: update.isMuted ?? track.isMuted,
      isSoloed: update.isSoloed ?? track.isSoloed,
    });
  }

  function removeTrack(track: TimelineTrack) {
    if (
      track.clips.length > 0 &&
      !window.confirm(`Delete ${track.name} and its ${track.clips.length} clip(s)?`)
    ) {
      return;
    }
    void deleteTimelineTrack(track.id);
  }

  function addClip(track: TimelineTrack, startTick = playheadTick) {
    void saveTimelineClip({
      id: null,
      trackId: track.id,
      name: `Clip ${track.clips.length + 1}`,
      startTick: snapTick(startTick),
      durationTicks: ticksPerBar,
      sourceOffsetTicks: 0,
    });
  }

  function commitClip(params: SaveTimelineClipParams) {
    return saveTimelineClip(params);
  }

  function removeClip(id: string) {
    setSelectedClipId((selected) => (selected === id ? null : selected));
    void deleteTimelineClip(id);
  }

  const rulerBars = Array.from({ length: totalBars }, (_, index) => ({
    number: index + 1,
    left: index * ticksPerBar * pixelsPerTick,
  }));

  return (
    <section className="flex h-full min-h-0 min-w-0 flex-col bg-background">
      <div
        ref={scrollContainer}
        className="min-h-0 flex-1 overflow-auto"
        style={{ "--timeline-track-height": "80px" } as CSSProperties}
      >
        <div className="relative min-h-full" style={{ width: TRACK_HEADER_WIDTH + timelineWidth }}>
          <div
            className="sticky top-0 z-30 grid h-8 border-b border-border bg-card"
            style={{ gridTemplateColumns: `${TRACK_HEADER_WIDTH}px ${timelineWidth}px` }}
          >
            <div className="sticky left-0 z-40 flex items-center justify-between border-r border-border bg-card px-3">
              <span className="font-mono text-[10px] uppercase tracking-[0.2em] text-muted-foreground">
                Arrangement
              </span>
              <Button size="icon-xs" variant="ghost" aria-label="Add track" onClick={addTrack}>
                <Plus />
              </Button>
            </div>
            <div
              className="relative cursor-crosshair overflow-hidden"
              style={laneStyle}
              onMouseDown={positionPlayhead}
            >
              {rulerBars.map((bar) => (
                <span
                  key={bar.number}
                  className="pointer-events-none absolute top-1 font-mono text-[10px] text-muted-foreground"
                  style={{ left: bar.left + 5 }}
                >
                  {bar.number}
                </span>
              ))}
              <div
                className="pointer-events-none absolute inset-y-0 z-20 w-px bg-foreground"
                style={{ left: playheadLeft }}
              >
                <div className="absolute -left-1 top-0 size-2 rotate-45 bg-foreground" />
              </div>
            </div>
          </div>

          {timeline.tracks.map((track) => (
            <div
              key={track.id}
              className="grid border-b border-border"
              style={{
                gridTemplateColumns: `${TRACK_HEADER_WIDTH}px ${timelineWidth}px`,
                height: "var(--timeline-track-height)",
              }}
            >
              <div className="sticky left-0 z-20 flex min-w-0 items-center gap-2 border-r border-border bg-card px-2">
                <div className="flex shrink-0 gap-1">
                  <Button
                    size="icon-xs"
                    variant={track.isMuted ? "secondary" : "ghost"}
                    aria-label={`Mute ${track.name}`}
                    aria-pressed={track.isMuted}
                    onClick={() => saveTrack(track, { isMuted: !track.isMuted })}
                  >
                    M
                  </Button>
                  <Button
                    size="icon-xs"
                    variant={track.isSoloed ? "secondary" : "ghost"}
                    aria-label={`Solo ${track.name}`}
                    aria-pressed={track.isSoloed}
                    onClick={() => saveTrack(track, { isSoloed: !track.isSoloed })}
                  >
                    S
                  </Button>
                </div>
                <Input
                  key={`${track.id}-${track.name}`}
                  className="h-7 min-w-0 border-transparent px-1 text-xs shadow-none focus-visible:border-ring"
                  defaultValue={track.name}
                  aria-label="Track name"
                  onKeyDown={(event) => {
                    if (event.key === "Enter") event.currentTarget.blur();
                  }}
                  onBlur={(event) => {
                    const name = event.currentTarget.value.trim();
                    if (name && name !== track.name) saveTrack(track, { name });
                    else event.currentTarget.value = track.name;
                  }}
                />
                <div className="flex shrink-0">
                  <Button
                    size="icon-xs"
                    variant="ghost"
                    aria-label={`Add clip to ${track.name}`}
                    onClick={() => addClip(track)}
                  >
                    <Plus />
                  </Button>
                  <Button
                    size="icon-xs"
                    variant="ghost"
                    aria-label={`Delete ${track.name}`}
                    disabled={timeline.tracks.length === 1}
                    onClick={() => removeTrack(track)}
                  >
                    <Trash2 />
                  </Button>
                </div>
              </div>
              <div
                data-timeline-track-id={track.id}
                className={cn(
                  "relative cursor-crosshair overflow-visible data-[clip-drop-target]:bg-accent/60",
                  (track.isMuted ||
                    (timeline.tracks.some((item) => item.isSoloed) && !track.isSoloed)) &&
                    "opacity-45",
                )}
                style={laneStyle}
                onMouseDown={(event) => {
                  if (event.target === event.currentTarget) positionPlayhead(event);
                }}
                onDoubleClick={(event) => {
                  if (event.target !== event.currentTarget) return;
                  const bounds = event.currentTarget.getBoundingClientRect();
                  addClip(track, (event.clientX - bounds.left) / pixelsPerTick);
                }}
              >
                {track.clips.map((clip) => (
                  <TimelineClip
                    key={clip.id}
                    clip={clip}
                    trackId={track.id}
                    pixelsPerTick={pixelsPerTick}
                    gridTicks={gridTicks}
                    snapEnabled={timeline.isSnapEnabled}
                    selected={clip.id === selectedClipId}
                    onSelect={setSelectedClipId}
                    onCommit={commitClip}
                    onDelete={removeClip}
                  />
                ))}
                <div
                  className="pointer-events-none absolute inset-y-0 z-10 w-px bg-foreground/70"
                  style={{ left: playheadLeft }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>

      <footer className="flex h-7 shrink-0 items-center gap-4 border-t border-border bg-card px-3 font-mono text-[10px] uppercase tracking-[0.14em] text-muted-foreground">
        <span>{timeline.tracks.length} tracks</span>
        <span>{timeline.tracks.reduce((count, track) => count + track.clips.length, 0)} clips</span>
        <span>{automaticGrid.label} grid</span>
        <span>{formatPosition(playheadTick, ticksPerBeat, ticksPerBar)}</span>
        <span className="ml-auto hidden md:inline">
          Ctrl+scroll horizontal / Alt+scroll vertical
        </span>
      </footer>
    </section>
  );
}

function gridForZoom(ticksPerQuarter: number, pixelsPerQuarter: number) {
  const divisions = [
    { ticks: ticksPerQuarter / 64, label: "1/256" },
    { ticks: ticksPerQuarter / 32, label: "1/128" },
    { ticks: ticksPerQuarter / 16, label: "1/64" },
    { ticks: ticksPerQuarter / 8, label: "1/32" },
    { ticks: ticksPerQuarter / 4, label: "1/16" },
    { ticks: ticksPerQuarter / 2, label: "1/8" },
    { ticks: ticksPerQuarter, label: "1/4" },
    { ticks: ticksPerQuarter * 2, label: "1/2" },
    { ticks: ticksPerQuarter * 4, label: "1/1" },
  ];
  const pixelsPerTick = pixelsPerQuarter / ticksPerQuarter;
  for (const division of divisions) {
    if (division.ticks * pixelsPerTick >= 10) return division;
  }
  return divisions[divisions.length - 1]!;
}

function timelineGridStyle(
  gridTicks: number,
  ticksPerBeat: number,
  ticksPerBar: number,
  pixelsPerTick: number,
): CSSProperties {
  return {
    backgroundImage: [
      "linear-gradient(to right, color-mix(in oklch, var(--foreground) 8%, transparent) 1px, transparent 1px)",
      "linear-gradient(to right, color-mix(in oklch, var(--foreground) 14%, transparent) 1px, transparent 1px)",
      "linear-gradient(to right, color-mix(in oklch, var(--foreground) 28%, transparent) 1px, transparent 1px)",
    ].join(","),
    backgroundSize: [gridTicks, ticksPerBeat, ticksPerBar]
      .map((ticks) => `${ticks * pixelsPerTick}px 100%`)
      .join(","),
  };
}

function formatPosition(tick: number, ticksPerBeat: number, ticksPerBar: number) {
  const bar = Math.floor(tick / ticksPerBar) + 1;
  const withinBar = tick % ticksPerBar;
  const beat = Math.floor(withinBar / ticksPerBeat) + 1;
  const beatTick = Math.round(withinBar % ticksPerBeat);
  return `${bar}.${beat}.${String(beatTick).padStart(3, "0")}`;
}
