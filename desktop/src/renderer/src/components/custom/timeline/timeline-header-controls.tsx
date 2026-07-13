import { Magnet } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useAppStore } from "@/stores/app-store";
import type { SetTimelineSettingsParams } from "@shared/ipc";

export function TimelineHeaderControls() {
  const workspace = useAppStore((state) => state.workspace);
  const setTimelineSettings = useAppStore((state) => state.setTimelineSettings);

  if (!workspace) return null;
  const { timeline } = workspace;

  function settings(overrides: Partial<SetTimelineSettingsParams> = {}) {
    return {
      bpm: timeline.bpm,
      timeSignatureNumerator: timeline.timeSignatureNumerator,
      timeSignatureDenominator: timeline.timeSignatureDenominator,
      gridDivision: timeline.gridDivision,
      isSnapEnabled: timeline.isSnapEnabled,
      ...overrides,
    } satisfies SetTimelineSettingsParams;
  }

  return (
    <div className="flex min-w-0 items-center gap-4 px-3">
      <label className="flex items-center gap-2">
        <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-muted-foreground">
          Tempo
        </span>
        <Input
          key={timeline.bpm}
          className="h-7 w-18 font-mono text-xs"
          type="number"
          min={20}
          max={400}
          step={0.1}
          defaultValue={timeline.bpm}
          aria-label="Project tempo"
          onKeyDown={(event) => {
            if (event.key === "Enter") event.currentTarget.blur();
          }}
          onBlur={(event) => {
            const bpm = event.currentTarget.valueAsNumber;
            if (Number.isFinite(bpm) && bpm >= 20 && bpm <= 400 && bpm !== timeline.bpm) {
              void setTimelineSettings(settings({ bpm }));
            } else {
              event.currentTarget.value = String(timeline.bpm);
            }
          }}
        />
        <span className="hidden text-xs text-muted-foreground lg:inline">BPM</span>
      </label>

      <label className="flex items-center gap-1.5 font-mono text-[10px] uppercase tracking-[0.18em] text-muted-foreground">
        Meter
        <select
          className="h-7 rounded-md border border-input bg-background px-1.5 font-mono text-xs text-foreground outline-none focus:border-ring"
          value={timeline.timeSignatureNumerator}
          aria-label="Time signature numerator"
          onChange={(event) =>
            void setTimelineSettings(
              settings({ timeSignatureNumerator: Number(event.currentTarget.value) }),
            )
          }
        >
          {Array.from({ length: 12 }, (_, index) => index + 1).map((value) => (
            <option key={value} value={value}>
              {value}
            </option>
          ))}
        </select>
        <span>/</span>
        <select
          className="h-7 rounded-md border border-input bg-background px-1.5 font-mono text-xs text-foreground outline-none focus:border-ring"
          value={timeline.timeSignatureDenominator}
          aria-label="Time signature denominator"
          onChange={(event) =>
            void setTimelineSettings(
              settings({ timeSignatureDenominator: Number(event.currentTarget.value) }),
            )
          }
        >
          {[1, 2, 4, 8, 16, 32].map((value) => (
            <option key={value} value={value}>
              {value}
            </option>
          ))}
        </select>
      </label>

      <Button
        size="xs"
        variant={timeline.isSnapEnabled ? "secondary" : "ghost"}
        aria-pressed={timeline.isSnapEnabled}
        onClick={() =>
          void setTimelineSettings(settings({ isSnapEnabled: !timeline.isSnapEnabled }))
        }
      >
        <Magnet /> Snap
      </Button>
    </div>
  );
}
