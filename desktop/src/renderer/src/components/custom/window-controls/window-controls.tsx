import { Maximize, Minus, X } from "lucide-react";

import { Button } from "@/components/ui/button";

export function WindowControls() {
  return (
    <div className="no-drag flex h-full items-stretch">
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className="h-full w-11 rounded-none text-muted-foreground hover:bg-accent hover:text-foreground"
        aria-label="Minimize window"
        onClick={() => void window.kickHatSnare.minimizeWindow()}
      >
        <Minus />
      </Button>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className="h-full w-11 rounded-none text-muted-foreground hover:bg-accent hover:text-foreground"
        aria-label="Expand window"
        onClick={() => void window.kickHatSnare.toggleMaximizeWindow()}
      >
        <Maximize />
      </Button>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className="h-full w-11 rounded-none text-muted-foreground hover:bg-destructive hover:text-white"
        aria-label="Close window"
        onClick={() => void window.kickHatSnare.closeWindow()}
      >
        <X />
      </Button>
    </div>
  );
}
