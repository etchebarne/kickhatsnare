import { useEffect, useRef } from "react";

interface ClipWaveformProps {
  peaks: number[];
  sourceOffsetTicks: number;
  durationTicks: number;
  sourceDurationTicks: number;
}

export function ClipWaveform({
  peaks,
  sourceOffsetTicks,
  durationTicks,
  sourceDurationTicks,
}: ClipWaveformProps) {
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const element = canvas.current;
    if (!element || peaks.length === 0) return;
    function draw() {
      if (!element) return;
      const bounds = element.getBoundingClientRect();
      const ratio = Math.min(window.devicePixelRatio, 2);
      element.width = Math.max(1, Math.round(bounds.width * ratio));
      element.height = Math.max(1, Math.round(bounds.height * ratio));
      const context = element.getContext("2d");
      if (!context) return;
      context.clearRect(0, 0, element.width, element.height);
      context.fillStyle = "currentColor";
      context.globalAlpha = 0.45;
      const center = element.height / 2;
      const visibleStart = sourceOffsetTicks / sourceDurationTicks;
      const visibleLength = durationTicks / sourceDurationTicks;
      for (let x = 0; x < element.width; x += 1) {
        const sourcePosition = visibleStart + (x / element.width) * visibleLength;
        const peak = peaks[Math.min(peaks.length - 1, Math.floor(sourcePosition * peaks.length))];
        const height = Math.max(1, (peak ?? 0) * element.height * 0.82);
        context.fillRect(x, center - height / 2, 1, height);
      }
    }
    const observer = new ResizeObserver(draw);
    observer.observe(element);
    draw();
    return () => observer.disconnect();
  }, [durationTicks, peaks, sourceDurationTicks, sourceOffsetTicks]);

  return <canvas ref={canvas} className="pointer-events-none absolute inset-0 size-full" />;
}
