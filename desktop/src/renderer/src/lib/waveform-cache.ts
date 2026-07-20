import type { GetWaveformPeaksParams, WaveformPeaks } from "@shared/ipc";

export const WAVEFORM_BASE_FRAMES_PER_PEAK = 16;
export const WAVEFORM_PAGE_PEAKS = 2_048;

const MAX_CACHED_PAGES = 256;
const WAVEFORM_POLL_INTERVAL_MS = 200;

type ReadyWaveformPeaks = Extract<WaveformPeaks, { status: "ready" }>;

export interface WaveformPage extends Omit<ReadyWaveformPeaks, "minimums" | "maximums"> {
  minimums: Float32Array;
  maximums: Float32Array;
}

interface CacheEntry {
  promise: Promise<WaveformPage>;
  value?: WaveformPage;
}

const pages = new Map<string, CacheEntry>();

export function loadWaveformPage(
  sourceKey: string,
  params: GetWaveformPeaksParams,
): Promise<WaveformPage> {
  const key = `${sourceKey}:${params.framesPerPeak}:${params.startPeak}`;
  const cached = pages.get(key);
  if (cached) {
    pages.delete(key);
    pages.set(key, cached);
    return cached.promise;
  }

  const entry: CacheEntry = {
    promise: requestWaveformPage(params).catch((error: unknown) => {
      pages.delete(key);
      throw error;
    }),
  };
  entry.promise.then((value) => {
    entry.value = value;
    evictPages();
  });
  pages.set(key, entry);
  evictPages();
  return entry.promise;
}

async function requestWaveformPage(params: GetWaveformPeaksParams): Promise<WaveformPage> {
  for (;;) {
    const peaks = await window.kickHatSnare.getWaveformPeaks(params);
    if (peaks.status === "ready") {
      return {
        ...peaks,
        minimums: Float32Array.from(peaks.minimums),
        maximums: Float32Array.from(peaks.maximums),
      };
    }
    await new Promise((resolve) => window.setTimeout(resolve, WAVEFORM_POLL_INTERVAL_MS));
  }
}

function evictPages() {
  if (pages.size <= MAX_CACHED_PAGES) return;
  for (const [key, entry] of pages) {
    if (pages.size <= MAX_CACHED_PAGES) break;
    if (entry.value) pages.delete(key);
  }
}
