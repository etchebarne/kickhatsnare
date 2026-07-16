import { useEffect, useState, type ComponentProps, type ReactNode } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { SetTimelineClipPropertiesParams, WorkspaceSnapshot } from "@shared/ipc";

type TimelineClipData = WorkspaceSnapshot["timeline"]["tracks"][number]["clips"][number];

interface AudioClipSettingsDialogProps {
  clip: TimelineClipData;
  open: boolean;
  onOpenChange(open: boolean): void;
  onSave(params: SetTimelineClipPropertiesParams): Promise<boolean>;
}

export function AudioClipSettingsDialog({
  clip,
  open,
  onOpenChange,
  onSave,
}: AudioClipSettingsDialogProps) {
  const [stretchMode, setStretchMode] = useState(clip.stretchMode);
  const [gainDb, setGainDb] = useState(numberDraft(clip.gainDb));
  const [pan, setPan] = useState(numberDraft(clip.pan));
  const [pitchSemitones, setPitchSemitones] = useState(numberDraft(clip.pitchSemitones));
  const [tempoPercent, setTempoPercent] = useState(numberDraft(clip.tempoPercent));
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (!open) return;
    setStretchMode(clip.stretchMode);
    setGainDb(numberDraft(clip.gainDb));
    setPan(numberDraft(clip.pan));
    setPitchSemitones(numberDraft(clip.pitchSemitones));
    setTempoPercent(numberDraft(clip.tempoPercent));
  }, [clip, open]);

  const parsedGainDb = Number(gainDb);
  const parsedPan = Number(pan);
  const parsedPitchSemitones = Number(pitchSemitones);
  const parsedTempoPercent = Number(tempoPercent);
  const valuesAreValid =
    gainDb.trim() !== "" &&
    pan.trim() !== "" &&
    pitchSemitones.trim() !== "" &&
    tempoPercent.trim() !== "" &&
    Number.isFinite(parsedGainDb) &&
    parsedGainDb >= -60 &&
    parsedGainDb <= 12 &&
    Number.isFinite(parsedPan) &&
    parsedPan >= -1 &&
    parsedPan <= 1 &&
    Number.isFinite(parsedPitchSemitones) &&
    parsedPitchSemitones >= -24 &&
    parsedPitchSemitones <= 24 &&
    Number.isFinite(parsedTempoPercent) &&
    parsedTempoPercent >= 25 &&
    parsedTempoPercent <= 400;
  const resamplePitch = 12 * Math.log2(parsedTempoPercent / 100);

  async function save(event: { preventDefault(): void }, makeUnique: boolean) {
    event.preventDefault();
    if (!valuesAreValid || isSaving) return;
    setIsSaving(true);
    const saved = await onSave({
      id: clip.id,
      stretchMode,
      gainDb: parsedGainDb,
      pan: parsedPan,
      pitchSemitones: parsedPitchSemitones,
      tempoPercent: parsedTempoPercent,
      makeUnique,
    });
    setIsSaving(false);
    if (saved) onOpenChange(false);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="gap-0 overflow-hidden p-0 sm:max-w-xl">
        <DialogHeader className="border-b border-border px-5 py-4">
          <DialogTitle className="truncate pr-8">{clip.name}</DialogTitle>
          <DialogDescription>
            Clip-local playback and timing. The source file is never modified.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={(event) => void save(event, false)}>
          <div className="grid gap-5 px-5 py-5 sm:grid-cols-2">
            <Field label="Time stretching" hint="Resample couples pitch to tempo.">
              <Select
                value={stretchMode}
                onValueChange={(value) => setStretchMode(value as typeof stretchMode)}
              >
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="resample">Resample</SelectItem>
                  <SelectItem value="stretch">Stretch</SelectItem>
                </SelectContent>
              </Select>
            </Field>
            <NumberField
              label="Volume"
              suffix="dB"
              value={gainDb}
              min={-60}
              max={12}
              step={0.1}
              onChange={setGainDb}
            />
            <NumberField
              label="Pan"
              suffix="L / R"
              value={pan}
              min={-1}
              max={1}
              step={0.01}
              onChange={setPan}
            />
            <NumberField
              label="Pitch"
              suffix="semitones"
              value={stretchMode === "resample" ? numberDraft(resamplePitch) : pitchSemitones}
              min={-24}
              max={24}
              step={0.01}
              disabled={stretchMode === "resample"}
              hint={
                stretchMode === "resample"
                  ? "Derived from tempo in Resample mode."
                  : "Independent of clip tempo."
              }
              onChange={setPitchSemitones}
            />
            <NumberField
              label="Tempo"
              suffix="%"
              value={tempoPercent}
              min={25}
              max={400}
              step={0.1}
              hint="Updates the clip length without changing its source window."
              onChange={setTempoPercent}
            />
          </div>
          <div className="flex items-center justify-between border-t border-border bg-muted/30 px-5 py-4">
            <div className="text-xs text-muted-foreground">
              {clip.isUnique ? "Unique clip settings" : "Settings shared with source instances"}
            </div>
            <DialogFooter className="flex-row">
              {!clip.isUnique ? (
                <Button
                  type="button"
                  variant="outline"
                  disabled={!valuesAreValid || isSaving}
                  onClick={(event) => void save(event, true)}
                >
                  Make unique
                </Button>
              ) : null}
              <Button type="submit" disabled={!valuesAreValid || isSaving}>
                Apply
              </Button>
            </DialogFooter>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}

function Field({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <label className="grid content-start gap-2 text-sm font-medium">
      <span>{label}</span>
      {children}
      {hint ? (
        <span className="text-xs leading-relaxed font-normal text-muted-foreground">{hint}</span>
      ) : null}
    </label>
  );
}

function NumberField({
  label,
  suffix,
  value,
  hint,
  onChange,
  ...inputProps
}: {
  label: string;
  suffix: string;
  value: string;
  hint?: string;
  onChange(value: string): void;
} & Pick<ComponentProps<"input">, "disabled" | "max" | "min" | "step">) {
  return (
    <Field label={label} hint={hint}>
      <div className="relative">
        <Input
          {...inputProps}
          type="number"
          value={value}
          className="pr-20 font-mono"
          onChange={(event) => onChange(event.currentTarget.value)}
        />
        <span className="pointer-events-none absolute inset-y-0 right-3 flex items-center text-[10px] uppercase tracking-wider text-muted-foreground">
          {suffix}
        </span>
      </div>
    </Field>
  );
}

function numberDraft(value: number) {
  return Number.isFinite(value) ? String(Number(value.toFixed(2))) : "";
}
