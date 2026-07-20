import { useAppStore } from "@/stores/app-store";

import { AppMenu } from "./app-menu";
import { WindowControls } from "./window-controls";

export function TitleBar() {
  const workspace = useAppStore((state) => state.workspace);
  const title = workspace ? `${workspace.isDirty ? "*" : ""}${workspace.name}` : "KickHatSnare";

  return (
    <header className="drag-region relative grid h-10 shrink-0 grid-cols-[1fr_auto_1fr] items-center border-b border-border bg-card select-none">
      <nav className="no-drag h-full justify-self-start" aria-label="Application menu">
        <AppMenu />
      </nav>
      <p className="pointer-events-none px-4 text-xs font-medium text-muted-foreground">{title}</p>
      <div className="h-full justify-self-end">
        <WindowControls />
      </div>
    </header>
  );
}
