import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  InputGroupText,
} from "@/components/ui/input-group";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
    <div className="flex min-w-0 items-center gap-3 px-3">
      <InputGroup className="h-8 w-28">
        <InputGroupInput
          key={timeline.bpm}
          className="h-8 font-mono text-xs"
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
        <InputGroupAddon align="inline-end" className="pr-2">
          <InputGroupText className="text-xs">BPM</InputGroupText>
        </InputGroupAddon>
      </InputGroup>

      <div
        className="flex items-center gap-1.5 font-mono text-xs text-muted-foreground"
        role="group"
        aria-label="Time signature"
      >
        <Select
          value={String(timeline.timeSignatureNumerator)}
          onValueChange={(value) =>
            void setTimelineSettings(settings({ timeSignatureNumerator: Number(value) }))
          }
        >
          <SelectTrigger
            size="sm"
            className="h-8 w-14 px-2 font-mono text-xs text-foreground"
            aria-label="Time signature numerator"
          >
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {Array.from({ length: 12 }, (_, index) => index + 1).map((value) => (
              <SelectItem key={value} value={String(value)}>
                {value}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <span>/</span>
        <Select
          value={String(timeline.timeSignatureDenominator)}
          onValueChange={(value) =>
            void setTimelineSettings(settings({ timeSignatureDenominator: Number(value) }))
          }
        >
          <SelectTrigger
            size="sm"
            className="h-8 w-14 px-2 font-mono text-xs text-foreground"
            aria-label="Time signature denominator"
          >
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {[1, 2, 4, 8, 16, 32].map((value) => (
              <SelectItem key={value} value={String(value)}>
                {value}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}
