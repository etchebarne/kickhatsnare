import { create } from "zustand";

export type TimelineTool = "select" | "cut";

interface TimelineState {
  tool: TimelineTool;
  setTool(tool: TimelineTool): void;
}

export const useTimelineStore = create<TimelineState>((set) => ({
  tool: "select",
  setTool: (tool) => set({ tool }),
}));
