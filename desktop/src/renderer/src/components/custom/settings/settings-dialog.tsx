import { useState } from "react";
import { SlidersHorizontal } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { OverlayScrollArea } from "@/components/ui/overlay-scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { useSettingsStore } from "@/stores/settings-store";
import type { SettingsSnapshot } from "@shared/ipc";

type Setting = SettingsSnapshot["categories"][number]["settings"][number];

export function SettingsDialog() {
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const isOpen = useSettingsStore((state) => state.isOpen);
  const snapshot = useSettingsStore((state) => state.snapshot);
  const isLoading = useSettingsStore((state) => state.isLoading);
  const pendingSettingId = useSettingsStore((state) => state.pendingSettingId);
  const error = useSettingsStore((state) => state.error);
  const setOpen = useSettingsStore((state) => state.setOpen);
  const update = useSettingsStore((state) => state.update);
  const categories = snapshot?.categories ?? [];
  const selectedCategory =
    categories.find((category) => category.id === selectedCategoryId) ?? categories[0];

  return (
    <Dialog open={isOpen} onOpenChange={setOpen}>
      <DialogContent className="h-[min(36rem,calc(100vh-2rem))] grid-rows-[auto_minmax(0,1fr)] gap-0 overflow-hidden p-0 sm:max-w-3xl">
        <DialogHeader className="border-b px-6 py-5 pr-12">
          <DialogTitle className="flex items-center gap-2">
            <SlidersHorizontal className="size-4 text-muted-foreground" />
            Settings
          </DialogTitle>
          <DialogDescription>
            Application preferences supplied and validated by the core.
          </DialogDescription>
        </DialogHeader>

        {isLoading && !snapshot ? (
          <SettingsLoading />
        ) : categories.length === 0 ? (
          <div className="grid min-h-0 flex-1 place-items-center p-8 text-center">
            <div>
              <p className={error ? "text-sm font-medium text-destructive" : "text-sm font-medium"}>
                {error ? "Settings unavailable" : "No settings available"}
              </p>
              <p className="mt-1 text-xs text-muted-foreground">
                {error ?? "The core did not register any configurable options."}
              </p>
            </div>
          </div>
        ) : (
          <div className="grid min-h-0 flex-1 grid-rows-[auto_minmax(0,1fr)] sm:grid-cols-[12rem_minmax(0,1fr)] sm:grid-rows-1">
            <nav
              aria-label="Settings categories"
              className="min-w-0 border-b bg-muted/20 sm:border-r sm:border-b-0"
            >
              <OverlayScrollArea
                className="sm:h-full"
                overflowY="hidden"
                viewportClassName="flex gap-1 p-2 sm:block sm:p-3"
              >
                {categories.map((category) => (
                  <Button
                    key={category.id}
                    type="button"
                    variant={selectedCategory?.id === category.id ? "secondary" : "ghost"}
                    className="h-9 justify-start px-3 text-xs sm:mb-1 sm:w-full"
                    onClick={() => setSelectedCategoryId(category.id)}
                  >
                    {category.label}
                  </Button>
                ))}
              </OverlayScrollArea>
            </nav>

            {selectedCategory ? (
              <section className="min-h-0">
                <OverlayScrollArea
                  className="h-full"
                  overflowX="hidden"
                  viewportClassName="px-5 py-5 sm:px-7 sm:py-6"
                >
                  <div className="mb-6">
                    <h2 className="mb-1 text-base font-semibold">{selectedCategory.label}</h2>
                    <p className="text-xs leading-relaxed text-muted-foreground">
                      {selectedCategory.description}
                    </p>
                  </div>

                  <div className="divide-y rounded-lg border bg-card">
                    {selectedCategory.settings.map((setting) => (
                      <SettingField
                        key={setting.id}
                        setting={setting}
                        disabled={pendingSettingId === setting.id}
                        onChange={(value) => void update({ id: setting.id, value })}
                      />
                    ))}
                  </div>
                  {error ? (
                    <p className="mt-3 text-xs text-destructive" role="alert">
                      {error}
                    </p>
                  ) : null}
                </OverlayScrollArea>
              </section>
            ) : null}
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}

function SettingField({
  setting,
  disabled,
  onChange,
}: {
  setting: Setting;
  disabled: boolean;
  onChange(value: { kind: "integer"; value: number }): void;
}) {
  switch (setting.kind) {
    case "integerSelect":
      return (
        <div className="grid gap-4 p-4 sm:grid-cols-[minmax(0,1fr)_10rem] sm:items-center sm:p-5">
          <div>
            <label className="text-sm font-medium" htmlFor={`setting-${setting.id}`}>
              {setting.label}
            </label>
            <p className="mt-1 max-w-md text-xs leading-relaxed text-muted-foreground">
              {setting.description}
            </p>
            <p className="mt-2 font-mono text-[10px] uppercase tracking-wider text-muted-foreground/70">
              Default {setting.defaultValue} {setting.unit}
            </p>
          </div>
          <Select
            value={String(setting.value)}
            disabled={disabled}
            onValueChange={(value) => onChange({ kind: "integer", value: Number(value) })}
          >
            <SelectTrigger
              id={`setting-${setting.id}`}
              className="w-full"
              aria-label={setting.label}
            >
              <SelectValue />
            </SelectTrigger>
            <SelectContent position="popper">
              {setting.options.map((option) => (
                <SelectItem key={option.value} value={String(option.value)}>
                  {option.label} {setting.unit}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      );
  }
}

function SettingsLoading() {
  return (
    <div className="grid min-h-0 flex-1 sm:grid-cols-[12rem_minmax(0,1fr)]">
      <div className="space-y-2 border-r bg-muted/20 p-3">
        <Skeleton className="h-9 w-full" />
        <Skeleton className="h-9 w-4/5" />
      </div>
      <div className="p-7">
        <Skeleton className="h-5 w-28" />
        <Skeleton className="mt-3 h-3 w-72 max-w-full" />
        <Skeleton className="mt-8 h-32 w-full rounded-lg" />
      </div>
    </div>
  );
}
