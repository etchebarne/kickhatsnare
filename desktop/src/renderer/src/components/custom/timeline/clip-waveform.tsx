import { useEffect, useRef, useState } from "react";

import {
  loadWaveformPage,
  WAVEFORM_BASE_FRAMES_PER_PEAK,
  WAVEFORM_PAGE_PEAKS,
  type WaveformPage,
} from "@/lib/waveform-cache";

interface ClipWaveformProps {
  sourcePath: string;
  sourceKey: string;
  sampleRate: number;
  sourceDurationSeconds: number;
  sourceOffsetTicks: number;
  sourceWindowTicks: number;
  sourceDurationTicks: number;
  clipWidth: number;
  visibleLeft: number;
  visibleWidth: number;
}

interface WaveformData {
  framesPerPeak: number;
  startPeak: number;
  minimums: Float32Array;
  maximums: Float32Array;
}

interface WaveformLayers {
  current: WaveformData;
  previous: WaveformData | null;
  transitionStart: number;
}

const LOD_HYSTERESIS = 1.15;
const LOD_TRANSITION_MS = 100;
const VIEWPORT_OVERSCAN = 0.5;

export function ClipWaveform({
  sourcePath,
  sourceKey,
  sampleRate,
  sourceDurationSeconds,
  sourceOffsetTicks,
  sourceWindowTicks,
  sourceDurationTicks,
  clipWidth,
  visibleLeft,
  visibleWidth,
}: ClipWaveformProps) {
  const totalFrames = Math.max(1, Math.round(sampleRate * sourceDurationSeconds));
  const canvas = useRef<HTMLCanvasElement>(null);
  const activeLod = useRef<number | undefined>(undefined);
  const hasDrawn = useRef(false);
  const mounted = useRef(true);
  const requestedRange = useRef<string | undefined>(undefined);
  const [layers, setLayers] = useState<WaveformLayers>();
  const [isReady, setIsReady] = useState(false);
  const sourceStartFrame = (totalFrames * sourceOffsetTicks) / sourceDurationTicks;
  const sourceWindowFrames = (totalFrames * sourceWindowTicks) / sourceDurationTicks;
  const visibleStartFrame = sourceStartFrame + (visibleLeft / clipWidth) * sourceWindowFrames;
  const visibleEndFrame =
    sourceStartFrame + ((visibleLeft + visibleWidth) / clipWidth) * sourceWindowFrames;

  useEffect(() => {
    const ratio = Math.min(window.devicePixelRatio, 2);
    const framesPerDevicePixel = sourceWindowFrames / Math.max(1, clipWidth * ratio);
    const framesPerPeak = chooseLod(framesPerDevicePixel, activeLod.current);
    activeLod.current = framesPerPeak;

    const overscanFrames = ((visibleEndFrame - visibleStartFrame) * VIEWPORT_OVERSCAN) / 2;
    const requestStart = Math.max(sourceStartFrame, visibleStartFrame - overscanFrames);
    const requestEnd = Math.min(
      sourceStartFrame + sourceWindowFrames,
      visibleEndFrame + overscanFrames,
    );
    const firstPeak = Math.max(0, Math.floor(requestStart / framesPerPeak));
    const lastPeak = Math.max(firstPeak, Math.ceil(requestEnd / framesPerPeak));
    const firstPage = Math.floor(firstPeak / WAVEFORM_PAGE_PEAKS);
    const lastPage = Math.floor(lastPeak / WAVEFORM_PAGE_PEAKS);
    const rangeKey = `${sourceKey}:${framesPerPeak}:${firstPage}:${lastPage}`;
    if (requestedRange.current === rangeKey) return;
    requestedRange.current = rangeKey;
    const requests: Promise<WaveformPage>[] = [];
    for (let page = firstPage; page <= lastPage; page += 1) {
      requests.push(
        loadWaveformPage(sourceKey, {
          sourcePath,
          framesPerPeak,
          startPeak: page * WAVEFORM_PAGE_PEAKS,
          peakCount: WAVEFORM_PAGE_PEAKS,
        }),
      );
    }

    void Promise.all(requests)
      .then((pages) => {
        if (!mounted.current || requestedRange.current !== rangeKey || pages.length === 0) return;
        const data = combinePages(pages);
        setLayers((current) => ({
          current: data,
          previous:
            current && current.current.framesPerPeak !== data.framesPerPeak
              ? current.current
              : null,
          transitionStart: performance.now(),
        }));
      })
      .catch(() => {
        // The application store reports backend errors; retain the last complete LOD here.
        if (requestedRange.current === rangeKey) requestedRange.current = undefined;
      });
  }, [
    clipWidth,
    sourceKey,
    sourcePath,
    sourceStartFrame,
    sourceWindowFrames,
    visibleEndFrame,
    visibleStartFrame,
  ]);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  useEffect(() => {
    const element = canvas.current;
    if (!element || !layers) return;
    const activeLayers = layers;
    let animationFrame: number | undefined;

    function draw(now = performance.now()) {
      if (!element) return;
      const bounds = element.getBoundingClientRect();
      const ratio = Math.min(window.devicePixelRatio, 2);
      const width = Math.max(1, Math.round(bounds.width * ratio));
      const height = Math.max(1, Math.round(bounds.height * ratio));
      if (element.width !== width) element.width = width;
      if (element.height !== height) element.height = height;
      const context = element.getContext("2d");
      if (!context) return;
      context.clearRect(0, 0, width, height);
      context.fillStyle = getComputedStyle(element).color;

      const progress = activeLayers.previous
        ? Math.min(1, (now - activeLayers.transitionStart) / LOD_TRANSITION_MS)
        : 1;
      if (activeLayers.previous && progress < 1) {
        drawWaveform(
          context,
          activeLayers.previous,
          visibleStartFrame,
          visibleEndFrame,
          width,
          height,
          1 - progress,
        );
      }
      drawWaveform(
        context,
        activeLayers.current,
        visibleStartFrame,
        visibleEndFrame,
        width,
        height,
        progress,
      );
      if (!hasDrawn.current) {
        hasDrawn.current = true;
        setIsReady(true);
      }
      if (progress < 1) animationFrame = window.requestAnimationFrame(draw);
    }

    const observer = new ResizeObserver(() => draw());
    observer.observe(element);
    draw();
    return () => {
      observer.disconnect();
      if (animationFrame !== undefined) window.cancelAnimationFrame(animationFrame);
    };
  }, [layers, visibleEndFrame, visibleStartFrame]);

  return (
    <div
      className="pointer-events-none absolute top-6 overflow-hidden"
      style={{ left: visibleLeft, width: visibleWidth, height: "calc(100% - 1.5rem)" }}
    >
      {!isReady ? (
        <div className="absolute inset-0 flex items-center justify-center overflow-hidden bg-background/10">
          <div
            className="absolute inset-0 animate-pulse opacity-20"
            style={{
              backgroundImage:
                "repeating-linear-gradient(90deg, currentColor 0, currentColor 1px, transparent 1px, transparent 8px)",
            }}
          />
          <span className="relative bg-background/55 px-1.5 py-0.5 font-mono text-[8px] uppercase tracking-[0.14em] opacity-70">
            Analyzing waveform
          </span>
        </div>
      ) : null}
      <canvas
        ref={canvas}
        aria-hidden="true"
        className={`absolute inset-0 size-full transition-opacity duration-100 ${isReady ? "opacity-100" : "opacity-0"}`}
      />
    </div>
  );
}

function chooseLod(framesPerDevicePixel: number, previous?: number) {
  if (!previous) return quantizedLod(framesPerDevicePixel);
  let level = previous;
  const upperBoundary = Math.SQRT2 * LOD_HYSTERESIS;
  const lowerBoundary = Math.SQRT1_2 / LOD_HYSTERESIS;
  while (framesPerDevicePixel > level * upperBoundary && level <= 2 ** 30) level *= 2;
  while (framesPerDevicePixel < level * lowerBoundary && level > WAVEFORM_BASE_FRAMES_PER_PEAK) {
    level /= 2;
  }
  return level;
}

function quantizedLod(framesPerDevicePixel: number) {
  const level = Math.round(
    Math.log2(Math.max(1, framesPerDevicePixel) / WAVEFORM_BASE_FRAMES_PER_PEAK),
  );
  return WAVEFORM_BASE_FRAMES_PER_PEAK * 2 ** Math.max(0, level);
}

function combinePages(pages: WaveformPage[]): WaveformData {
  const ordered = pages.toSorted((left, right) => left.startPeak - right.startPeak);
  const length = ordered.reduce((total, page) => total + page.minimums.length, 0);
  const minimums = new Float32Array(length);
  const maximums = new Float32Array(length);
  let offset = 0;
  for (const page of ordered) {
    minimums.set(page.minimums, offset);
    maximums.set(page.maximums, offset);
    offset += page.minimums.length;
  }
  return {
    framesPerPeak: ordered[0]!.framesPerPeak,
    startPeak: ordered[0]!.startPeak,
    minimums,
    maximums,
  };
}

function drawWaveform(
  context: CanvasRenderingContext2D,
  data: WaveformData,
  visibleStartFrame: number,
  visibleEndFrame: number,
  width: number,
  height: number,
  opacity: number,
) {
  context.globalAlpha = opacity * 0.55;
  const center = height / 2;
  const amplitude = height * 0.44;
  const framesPerPixel = (visibleEndFrame - visibleStartFrame) / width;
  for (let x = 0; x < width; x += 1) {
    const firstPeak = Math.floor((visibleStartFrame + x * framesPerPixel) / data.framesPerPeak);
    const lastPeak = Math.floor(
      (visibleStartFrame + (x + 1) * framesPerPixel) / data.framesPerPeak,
    );
    let minimum = 1;
    let maximum = -1;
    for (let peak = firstPeak; peak <= lastPeak; peak += 1) {
      const index = peak - data.startPeak;
      const nextMinimum = data.minimums[index];
      const nextMaximum = data.maximums[index];
      if (nextMinimum !== undefined) minimum = Math.min(minimum, nextMinimum);
      if (nextMaximum !== undefined) maximum = Math.max(maximum, nextMaximum);
    }
    if (maximum < minimum) continue;
    const top = center - maximum * amplitude;
    const bottom = center - minimum * amplitude;
    context.fillRect(x, top, 1, Math.max(1, bottom - top));
  }
  context.globalAlpha = 1;
}
