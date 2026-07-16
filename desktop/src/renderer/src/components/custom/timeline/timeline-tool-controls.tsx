import { MousePointer2, Scissors } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useTimelineStore, type TimelineTool } from "@/stores/timeline-store";

const tools: Array<{ id: TimelineTool; label: string; icon: typeof MousePointer2 }> = [
  { id: "select", label: "Select tool", icon: MousePointer2 },
  { id: "cut", label: "Cut tool", icon: Scissors },
];

export function TimelineToolControls() {
  const tool = useTimelineStore((state) => state.tool);
  const setTool = useTimelineStore((state) => state.setTool);

  return (
    <div
      className="flex items-center gap-1 border-r border-border px-2"
      role="toolbar"
      aria-label="Timeline tools"
    >
      {tools.map((item) => {
        const Icon = item.icon;
        return (
          <Button
            key={item.id}
            size="icon-sm"
            variant={tool === item.id ? "secondary" : "ghost"}
            aria-label={item.label}
            aria-pressed={tool === item.id}
            title={item.label}
            onClick={() => setTool(item.id)}
          >
            <Icon />
          </Button>
        );
      })}
    </div>
  );
}
