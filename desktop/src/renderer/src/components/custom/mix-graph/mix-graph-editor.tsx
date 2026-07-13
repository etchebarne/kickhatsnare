import { useEffect, useState } from "react";
import {
  Background,
  BackgroundVariant,
  ReactFlow,
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  type Connection,
  type Edge,
  type EdgeChange,
  type NodeChange,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import { MasterOutputNode, type MasterOutputNodeType } from "./master-output-node";
import { TrackChannelNode, type TrackChannelNodeType } from "./track-channel-node";
import { useAppStore } from "@/stores/app-store";

type MixNode = TrackChannelNodeType | MasterOutputNodeType;

const nodeTypes = {
  masterOutput: MasterOutputNode,
  trackChannel: TrackChannelNode,
};

export function MixGraphEditor() {
  const workspace = useAppStore((state) => state.workspace);
  const saveTrack = useAppStore((state) => state.saveTimelineTrack);
  const setNodePosition = useAppStore((state) => state.setMixNodePosition);
  const [nodes, setNodes] = useStateNodes();
  const [edges, setEdges] = useStateEdges();

  useEffect(() => {
    if (!workspace) return;
    const timeline = workspace.timeline;
    setNodes([
      ...timeline.tracks.map<MixNode>((track) => ({
        id: track.id,
        type: "trackChannel",
        position: { x: track.nodeX, y: track.nodeY },
        data: { track },
        deletable: false,
      })),
      {
        id: "master",
        type: "masterOutput",
        position: { x: timeline.masterNodeX, y: timeline.masterNodeY },
        data: { timeline },
        deletable: false,
      },
    ]);
    setEdges(
      timeline.tracks
        .filter((track) => track.isConnected)
        .map((track) => ({
          id: `route:${track.id}`,
          source: track.id,
          sourceHandle: "audio-out",
          target: "master",
          targetHandle: "audio-in",
          type: "smoothstep",
          animated: false,
          style: { stroke: "var(--foreground)", strokeWidth: 1.5 },
        })),
    );
  }, [setEdges, setNodes, workspace]);

  if (!workspace) return null;

  function setRoute(trackId: string, isConnected: boolean) {
    const track = workspace!.timeline.tracks.find((item) => item.id === trackId);
    if (!track) return;
    void saveTrack({
      id: track.id,
      name: track.name,
      isMuted: track.isMuted,
      isSoloed: track.isSoloed,
      gainDb: track.gainDb,
      pan: track.pan,
      isConnected,
    });
  }

  function handleConnect(connection: Connection) {
    if (connection.target !== "master" || connection.source === "master") return;
    setEdges((current) => addEdge({ ...connection, id: `route:${connection.source}` }, current));
    setRoute(connection.source, true);
  }

  function handleEdgesChange(changes: EdgeChange<Edge>[]) {
    for (const change of changes) {
      if (change.type === "remove" && change.id.startsWith("route:")) {
        setRoute(change.id.slice("route:".length), false);
      }
    }
    setEdges((current) => applyEdgeChanges(changes, current));
  }

  return (
    <div className="h-full min-h-0 bg-background">
      <ReactFlow<MixNode, Edge>
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        fitView
        minZoom={0.2}
        maxZoom={2}
        deleteKeyCode={["Backspace", "Delete"]}
        onNodesChange={(changes: NodeChange<MixNode>[]) =>
          setNodes((current) => applyNodeChanges(changes, current))
        }
        onNodeDragStop={(_event, node) =>
          void setNodePosition({ nodeId: node.id, x: node.position.x, y: node.position.y })
        }
        onEdgesChange={handleEdgesChange}
        onConnect={handleConnect}
        isValidConnection={(connection) =>
          connection.target === "master" && connection.source !== "master"
        }
        proOptions={{ hideAttribution: false }}
      >
        <Background variant={BackgroundVariant.Dots} gap={24} size={1} color="var(--border)" />
      </ReactFlow>
    </div>
  );
}

function useStateNodes() {
  const [nodes, setNodes] = useState<MixNode[]>([]);
  return [nodes, setNodes] as const;
}

function useStateEdges() {
  const [edges, setEdges] = useState<Edge[]>([]);
  return [edges, setEdges] as const;
}
