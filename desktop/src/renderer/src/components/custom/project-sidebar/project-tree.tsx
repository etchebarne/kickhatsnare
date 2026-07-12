import { FileTree, useFileTree } from "@pierre/trees/react";

const placeholderPaths = [
  "Audio/Drums/Hi-Hat Closed.wav",
  "Audio/Drums/Kick.wav",
  "Audio/Drums/Snare.wav",
  "Audio/Synths/Bass.wav",
  "Audio/Synths/Lead.wav",
  "MIDI/Drums.mid",
  "MIDI/Synths.mid",
  "Presets/Default.json",
];

const searchHeaderStyles = `
  [data-file-tree-search-container] {
    box-sizing: border-box;
    flex: 0 0 44px;
    align-items: center;
    margin-bottom: 0;
    padding-inline: 4px;
    border-bottom: 1px solid var(--trees-border-color);
  }

  [data-file-tree-search-input] {
    margin-block: 0;
  }

  [data-file-tree-virtualized-scroll='true'] {
    padding-top: 4px;
  }
`;

export function ProjectTree() {
  const { model } = useFileTree({
    density: "compact",
    fileTreeSearchMode: "hide-non-matches",
    icons: { colored: false, set: "standard" },
    initialExpansion: 2,
    paths: placeholderPaths,
    search: true,
    unsafeCSS: searchHeaderStyles,
  });

  return <FileTree className="project-tree" model={model} aria-label="Project files" />;
}
