import { useEffect, useState } from "react";
import {
  Background,
  BackgroundVariant,
  BaseEdge,
  ReactFlow,
  applyEdgeChanges,
  applyNodeChanges,
  type Connection,
  type Edge,
  type EdgeChange,
  type EdgeProps,
  type NodeChange,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import { MasterOutputNode, type MasterOutputNodeType } from "./master-output-node";
import { TrackChannelNode, type TrackChannelNodeType } from "./track-channel-node";
import { useAppStore } from "@/stores/app-store";
import type { WorkspaceSnapshot } from "@shared/ipc";

type MixNode = TrackChannelNodeType | MasterOutputNodeType;
type MixConnection = WorkspaceSnapshot["timeline"]["mixGraph"]["connections"][number];
type MixEdge = Edge<{ connection: MixConnection }, "noodle">;

const nodeTypes = {
  masterOutput: MasterOutputNode,
  trackChannel: TrackChannelNode,
};

const edgeTypes = {
  noodle: NoodleEdge,
};

export function MixGraphEditor() {
  const workspace = useAppStore((state) => state.workspace);
  const connectMixPorts = useAppStore((state) => state.connectMixPorts);
  const disconnectMixPorts = useAppStore((state) => state.disconnectMixPorts);
  const setNodePosition = useAppStore((state) => state.setMixNodePosition);
  const [nodes, setNodes] = useState<MixNode[]>([]);
  const [edges, setEdges] = useState<MixEdge[]>([]);

  useEffect(() => {
    if (!workspace) return;
    const timeline = workspace.timeline;
    setNodes(
      timeline.mixGraph.nodes.flatMap<MixNode>((mixNode) => {
        if (mixNode.kind === "trackChannel") {
          const track = timeline.tracks.find((item) => item.id === mixNode.trackId);
          return track
            ? [
                {
                  id: mixNode.id,
                  type: "trackChannel",
                  position: { x: mixNode.x, y: mixNode.y },
                  data: { track, mixNode },
                  deletable: false,
                  zIndex: 1,
                },
              ]
            : [];
        }
        return [
          {
            id: mixNode.id,
            type: "masterOutput",
            position: { x: mixNode.x, y: mixNode.y },
            data: { timeline, mixNode },
            deletable: false,
            zIndex: 1,
          },
        ];
      }),
    );
    setEdges(
      timeline.mixGraph.connections.map((connection) => ({
        id: connectionId(connection),
        type: "noodle",
        source: connection.sourceNodeId,
        sourceHandle: connection.sourcePortId,
        target: connection.targetNodeId,
        targetHandle: connection.targetPortId,
        data: { connection },
        style: {
          stroke: "color-mix(in oklch, var(--foreground) 55%, transparent)",
          strokeWidth: 1.5,
        },
      })),
    );
  }, [workspace]);

  if (!workspace) return null;
  const graph = workspace.timeline.mixGraph;

  function handleConnect(connection: Connection) {
    if (
      !connection.source ||
      !connection.sourceHandle ||
      !connection.target ||
      !connection.targetHandle
    ) {
      return;
    }
    void connectMixPorts({
      sourceNodeId: connection.source,
      sourcePortId: connection.sourceHandle,
      targetNodeId: connection.target,
      targetPortId: connection.targetHandle,
    });
  }

  function handleEdgesChange(changes: EdgeChange<MixEdge>[]) {
    for (const change of changes) {
      if (change.type !== "remove") continue;
      const connection = edges.find((edge) => edge.id === change.id)?.data?.connection;
      if (connection) void disconnectMixPorts(connection);
    }
    setEdges((current) => applyEdgeChanges(changes, current));
  }

  function isValidConnection(connection: Connection | MixEdge) {
    const source = graph.nodes.find((node) => node.id === connection.source);
    const target = graph.nodes.find((node) => node.id === connection.target);
    const sourcePort = source?.ports.find((port) => port.id === connection.sourceHandle);
    const targetPort = target?.ports.find((port) => port.id === connection.targetHandle);
    if (
      !sourcePort ||
      !targetPort ||
      sourcePort.direction !== "output" ||
      targetPort.direction !== "input" ||
      sourcePort.signalType !== targetPort.signalType
    ) {
      return false;
    }
    if (
      (!sourcePort.allowsMultipleConnections &&
        graph.connections.some(
          (item) =>
            item.sourceNodeId === connection.source &&
            item.sourcePortId === connection.sourceHandle,
        )) ||
      (!targetPort.allowsMultipleConnections &&
        graph.connections.some(
          (item) =>
            item.targetNodeId === connection.target &&
            item.targetPortId === connection.targetHandle,
        ))
    ) {
      return false;
    }
    return !graph.connections.some(
      (item) =>
        item.sourceNodeId === connection.source &&
        item.sourcePortId === connection.sourceHandle &&
        item.targetNodeId === connection.target &&
        item.targetPortId === connection.targetHandle,
    );
  }

  return (
    <div className="h-full min-h-0 bg-background">
      <ReactFlow<MixNode, MixEdge>
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
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
        isValidConnection={isValidConnection}
        connectionLineStyle={{ stroke: "var(--foreground)", strokeWidth: 1.5 }}
        proOptions={{ hideAttribution: false }}
      >
        <Background variant={BackgroundVariant.Dots} gap={24} size={1} color="var(--border)" />
      </ReactFlow>
    </div>
  );
}

function NoodleEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  markerEnd,
  markerStart,
  style,
  interactionWidth,
}: EdgeProps<MixEdge>) {
  const direction = targetX >= sourceX ? 1 : -1;
  const controlOffset = Math.max(96, Math.abs(targetX - sourceX) * 0.48);
  const path = `M ${sourceX} ${sourceY} C ${sourceX + controlOffset * direction} ${sourceY}, ${targetX - controlOffset * direction} ${targetY}, ${targetX} ${targetY}`;
  return (
    <BaseEdge
      id={id}
      path={path}
      markerEnd={markerEnd}
      markerStart={markerStart}
      style={style}
      interactionWidth={interactionWidth}
    />
  );
}

function connectionId(connection: MixConnection) {
  return `${connection.sourceNodeId}:${connection.sourcePortId}->${connection.targetNodeId}:${connection.targetPortId}`;
}
