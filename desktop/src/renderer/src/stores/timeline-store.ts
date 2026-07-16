import { create } from "zustand";

export type TimelineTool = "select" | "cut";
export type TimelineResizeMode = "trim" | "stretch";

interface TimelineState {
  tool: TimelineTool;
  resizeMode: TimelineResizeMode;
  setTool(tool: TimelineTool): void;
  setResizeMode(resizeMode: TimelineResizeMode): void;
}

export const useTimelineStore = create<TimelineState>((set) => ({
  tool: "select",
  resizeMode: "trim",
  setTool: (tool) => set({ tool }),
  setResizeMode: (resizeMode) => set({ resizeMode }),
}));
