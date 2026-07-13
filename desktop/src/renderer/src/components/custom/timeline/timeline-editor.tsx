import {
  useEffect,
  useRef,
  useState,
  type CSSProperties,
  type DragEvent,
  type MouseEvent,
} from "react";
import { Plus, Trash2 } from "lucide-react";

import { TimelineClip } from "./timeline-clip";
import { TimelinePlayhead } from "./timeline-playhead";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/app-store";
import { useTransportStore } from "@/stores/transport-store";
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
const AUDIO_DRAG_TYPE = "application/x-kickhatsnare-audio";
const AUDIO_EXTENSIONS = new Set([
  "aif",
  "aiff",
  "flac",
  "m4a",
  "mp3",
  "oga",
  "ogg",
  "opus",
  "wav",
  "wave",
]);

export function TimelineEditor() {
  const workspace = useAppStore((state) => state.workspace);
  const saveTimelineTrack = useAppStore((state) => state.saveTimelineTrack);
  const deleteTimelineTrack = useAppStore((state) => state.deleteTimelineTrack);
  const saveTimelineClip = useAppStore((state) => state.saveTimelineClip);
  const deleteTimelineClip = useAppStore((state) => state.deleteTimelineClip);
  const addAudioClip = useAppStore((state) => state.addAudioClip);
  const importAudioFiles = useAppStore((state) => state.importAudioFiles);
  const seek = useTransportStore((state) => state.seek);
  const scrollContainer = useRef<HTMLDivElement>(null);
  const trackHeight = useRef(80);
  const horizontalZoom = useRef(96);
  const [pixelsPerQuarter, setPixelsPerQuarter] = useState(96);
  const [viewportWidth, setViewportWidth] = useState(0);
  const [selectedClipId, setSelectedClipId] = useState<string | null>(null);
  const timelineForZoom = workspace?.timeline;
  const zoomMetrics = timelineForZoom ? timelineMetrics(timelineForZoom) : null;
  const minimumHorizontalZoom = zoomMetrics
    ? Math.min(
        MIN_HORIZONTAL_ZOOM,
        Math.max(
          0.5,
          (Math.max(200, viewportWidth - TRACK_HEADER_WIDTH - 16) * zoomMetrics.ticksPerQuarter) /
            zoomMetrics.totalTicks,
        ),
      )
    : MIN_HORIZONTAL_ZOOM;

  useEffect(() => {
    const container = scrollContainer.current;
    if (!container || !timelineForZoom) return;
    const observer = new ResizeObserver(() => setViewportWidth(container.clientWidth));
    observer.observe(container);
    setViewportWidth(container.clientWidth);
    return () => observer.disconnect();
  }, [timelineForZoom]);

  useEffect(() => {
    if (horizontalZoom.current >= minimumHorizontalZoom) return;
    horizontalZoom.current = minimumHorizontalZoom;
    setPixelsPerQuarter(minimumHorizontalZoom);
  }, [minimumHorizontalZoom]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (
        selectedClipId === null ||
        (event.key !== "Delete" && event.key !== "Backspace") ||
        isEditable(event.target)
      ) {
        return;
      }
      event.preventDefault();
      setSelectedClipId(null);
      void deleteTimelineClip(selectedClipId);
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [deleteTimelineClip, selectedClipId]);

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
        minimumHorizontalZoom,
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
  }, [minimumHorizontalZoom, pixelsPerQuarter, timelineForZoom]);

  if (!workspace) return null;
  const { timeline } = workspace;
  const { ticksPerBeat, ticksPerBar, totalBars, totalTicks } = timelineMetrics(timeline);
  const pixelsPerTick = pixelsPerQuarter / timeline.ticksPerQuarter;
  const automaticGrid = gridForZoom(
    timeline.ticksPerQuarter,
    ticksPerBeat,
    ticksPerBar,
    pixelsPerQuarter,
    totalTicks,
  );
  const gridTicks = automaticGrid.ticks;
  const timelineWidth = totalTicks * pixelsPerTick;
  const laneStyle = timelineGridStyle(automaticGrid, pixelsPerTick);

  function snapTick(tick: number) {
    const bounded = Math.max(0, Math.min(Math.round(tick), totalTicks));
    return timeline.isSnapEnabled ? Math.round(bounded / gridTicks) * gridTicks : bounded;
  }

  function positionPlayhead(event: MouseEvent<HTMLElement>) {
    const bounds = event.currentTarget.getBoundingClientRect();
    void seek(snapTick((event.clientX - bounds.left) / pixelsPerTick));
  }

  function addTrack() {
    void saveTimelineTrack({
      id: null,
      name: `Track ${timeline.tracks.length + 1}`,
      isMuted: false,
      isSoloed: false,
      gainDb: 0,
      pan: 0,
    });
  }

  function saveTrack(track: TimelineTrack, update: Partial<TimelineTrack>) {
    void saveTimelineTrack({
      id: track.id,
      name: update.name ?? track.name,
      isMuted: update.isMuted ?? track.isMuted,
      isSoloed: update.isSoloed ?? track.isSoloed,
      gainDb: update.gainDb ?? track.gainDb,
      pan: update.pan ?? track.pan,
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

  function addClip(
    track: TimelineTrack,
    startTick = useTransportStore.getState().transport.positionTick,
  ) {
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

  function handleAudioDragOver(event: DragEvent<HTMLDivElement>) {
    if (
      !event.dataTransfer.types.includes(AUDIO_DRAG_TYPE) &&
      !event.dataTransfer.types.includes("Files")
    ) {
      return;
    }
    event.preventDefault();
    event.dataTransfer.dropEffect = "copy";
  }

  function handleAudioDrop(event: DragEvent<HTMLDivElement>, track: TimelineTrack) {
    const sourcePath = event.dataTransfer.getData(AUDIO_DRAG_TYPE);
    const bounds = event.currentTarget.getBoundingClientRect();
    const startTick = snapTick((event.clientX - bounds.left) / pixelsPerTick);
    if (sourcePath) {
      event.preventDefault();
      void addAudioClip({ trackId: track.id, sourcePath, startTick });
      return;
    }

    const files = Array.from(event.dataTransfer.files).filter(isAudioFile);
    if (files.length === 0) return;
    event.preventDefault();
    void importAndPlaceAudio(files, track.id, startTick);
  }

  async function importAndPlaceAudio(files: File[], trackId: string, startTick: number) {
    let nextTick = startTick;
    for (const file of files) {
      const importedPaths = await importAudioFiles([file], "");
      const workspaceFiles = useAppStore.getState().workspace?.files ?? [];
      const sourcePath =
        importedPaths[0] ?? (workspaceFiles.includes(file.name) ? file.name : null);
      if (!sourcePath) continue;
      const previousClipIds = new Set(
        useAppStore
          .getState()
          .workspace?.timeline.tracks.find((item) => item.id === trackId)
          ?.clips.map((clip) => clip.id) ?? [],
      );
      const added = await addAudioClip({ trackId, sourcePath, startTick: nextTick });
      if (!added) continue;
      const track = useAppStore
        .getState()
        .workspace?.timeline.tracks.find((item) => item.id === trackId);
      const clip = track?.clips.find((item) => !previousClipIds.has(item.id));
      if (clip) nextTick += clip.durationTicks;
    }
  }

  const rulerStepBars = Math.max(
    1,
    Math.round(Math.max(ticksPerBar, automaticGrid.mediumTicks) / ticksPerBar),
  );
  const rulerBars = Array.from({ length: Math.ceil(totalBars / rulerStepBars) }, (_, index) => ({
    number: index * rulerStepBars + 1,
    left: index * rulerStepBars * ticksPerBar * pixelsPerTick,
  }));

  return (
    <section className="flex h-full min-h-0 min-w-0 flex-col bg-background">
      <div
        ref={scrollContainer}
        className="min-h-0 flex-1 overflow-auto"
        style={{ "--timeline-track-height": "80px" } as CSSProperties}
      >
        <div className="relative min-h-full" style={{ width: TRACK_HEADER_WIDTH + timelineWidth }}>
          <TimelinePlayhead leftOffset={TRACK_HEADER_WIDTH} pixelsPerTick={pixelsPerTick} />
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
                onDragOver={handleAudioDragOver}
                onDrop={(event) => handleAudioDrop(event, track)}
              >
                {track.clips.map((clip) => (
                  <TimelineClip
                    key={clip.id}
                    clip={clip}
                    trackId={track.id}
                    pixelsPerTick={pixelsPerTick}
                    gridTicks={gridTicks}
                    sourceDurationTicks={
                      clip.sourcePath
                        ? Math.max(
                            1,
                            Math.round(
                              (clip.sourceDurationSeconds *
                                timeline.bpm *
                                timeline.ticksPerQuarter) /
                                60,
                            ),
                          )
                        : null
                    }
                    snapEnabled={timeline.isSnapEnabled}
                    selected={clip.id === selectedClipId}
                    onSelect={setSelectedClipId}
                    onCommit={commitClip}
                    onDelete={removeClip}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function isAudioFile(file: File) {
  const extension = file.name.split(".").pop()?.toLowerCase();
  return extension !== undefined && AUDIO_EXTENSIONS.has(extension);
}

function isEditable(target: EventTarget | null) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || ["INPUT", "SELECT", "TEXTAREA"].includes(target.tagName))
  );
}

function timelineMetrics(timeline: Timeline) {
  const ticksPerBeat = (timeline.ticksPerQuarter * 4) / timeline.timeSignatureDenominator;
  const ticksPerBar = ticksPerBeat * timeline.timeSignatureNumerator;
  const maxClipEnd = timeline.tracks.reduce(
    (maximum, track) =>
      Math.max(maximum, ...track.clips.map((clip) => clip.startTick + clip.durationTicks)),
    0,
  );
  const totalBars = Math.max(DEFAULT_BARS, Math.ceil(maxClipEnd / ticksPerBar) + 4);
  return {
    ticksPerQuarter: timeline.ticksPerQuarter,
    ticksPerBeat,
    ticksPerBar,
    totalBars,
    totalTicks: totalBars * ticksPerBar,
  };
}

function gridForZoom(
  ticksPerQuarter: number,
  ticksPerBeat: number,
  ticksPerBar: number,
  pixelsPerQuarter: number,
  totalTicks: number,
) {
  const candidates = new Map<number, string>();
  const addCandidate = (ticks: number, label: string) => {
    if (ticks > 0 && ticks <= ticksPerBeat && !candidates.has(ticks)) {
      candidates.set(ticks, label);
    }
  };
  addCandidate(ticksPerQuarter / 64, "1/256");
  addCandidate(ticksPerQuarter / 32, "1/128");
  addCandidate(ticksPerQuarter / 16, "1/64");
  addCandidate(ticksPerQuarter / 8, "1/32");
  addCandidate(ticksPerQuarter / 4, "1/16");
  addCandidate(ticksPerQuarter / 2, "1/8");
  addCandidate(ticksPerQuarter, "1/4");
  if (!candidates.has(ticksPerBeat)) candidates.set(ticksPerBeat, "1 beat");

  const maximumGridTicks = Math.max(totalTicks * 64, ticksPerBar * 16);
  for (let bars = 1; ticksPerBar * bars <= maximumGridTicks; bars *= 2) {
    candidates.set(ticksPerBar * bars, bars === 1 ? "1 bar" : `${bars} bars`);
  }
  const divisions = Array.from(candidates, ([ticks, label]) => ({ ticks, label })).sort(
    (left, right) => left.ticks - right.ticks,
  );
  const pixelsPerTick = pixelsPerQuarter / ticksPerQuarter;
  const division =
    divisions.find((candidate) => candidate.ticks * pixelsPerTick >= 12) ??
    divisions[divisions.length - 1]!;

  function coarser(current: number) {
    const target = current * 4;
    if (current < ticksPerBar && ticksPerBar <= target) return ticksPerBar;
    return divisions.find((candidate) => candidate.ticks >= target)?.ticks ?? target;
  }

  const mediumTicks = coarser(division.ticks);
  return {
    ...division,
    mediumTicks,
    majorTicks: coarser(mediumTicks),
  };
}

function timelineGridStyle(
  grid: { ticks: number; mediumTicks: number; majorTicks: number },
  pixelsPerTick: number,
): CSSProperties {
  return {
    backgroundImage: [
      "linear-gradient(to right, color-mix(in oklch, var(--foreground) 8%, transparent) 1px, transparent 1px)",
      "linear-gradient(to right, color-mix(in oklch, var(--foreground) 14%, transparent) 1px, transparent 1px)",
      "linear-gradient(to right, color-mix(in oklch, var(--foreground) 28%, transparent) 1px, transparent 1px)",
    ].join(","),
    backgroundSize: [grid.ticks, grid.mediumTicks, grid.majorTicks]
      .map((ticks) => `${ticks * pixelsPerTick}px 100%`)
      .join(","),
  };
}
