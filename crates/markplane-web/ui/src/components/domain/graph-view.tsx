"use client";

import { useCallback, useMemo } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
} from "@xyflow/react";
import dagre from "@dagrejs/dagre";
import "@xyflow/react/dist/style.css";

import { PREFIX_CONFIG } from "@/lib/constants";
import type { GraphData } from "@/lib/types";

const NODE_WIDTH = 180;
const NODE_HEIGHT = 80;

function getLayoutedElements(graphData: GraphData) {
  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir: "TB", nodesep: 60, ranksep: 80 });

  graphData.nodes.forEach((node) => {
    g.setNode(node.id, { width: NODE_WIDTH, height: NODE_HEIGHT });
  });

  graphData.edges.forEach((edge) => {
    g.setEdge(edge.source, edge.target);
  });

  dagre.layout(g);

  const nodes: Node[] = graphData.nodes.map((node) => {
    const pos = g.node(node.id);
    const prefix = node.id.split("-")[0];
    const entityColor = PREFIX_CONFIG[prefix]?.color ?? "var(--entity-task)";

    let statusColor = "var(--status-draft)";
    if (node.status === "in-progress" || node.status === "active") {
      statusColor = "var(--status-in-progress)";
    } else if (node.status === "done") {
      statusColor = "var(--status-done)";
    } else if (node.status === "planned") {
      statusColor = "var(--status-planned)";
    } else if (node.status === "backlog") {
      statusColor = "var(--status-backlog)";
    }

    return {
      id: node.id,
      position: {
        x: pos.x - NODE_WIDTH / 2,
        y: pos.y - NODE_HEIGHT / 2,
      },
      data: {
        label: node.title,
        entityType: node.type,
        status: node.status,
        entityColor,
        statusColor,
      },
      type: "itemNode",
      style: {
        width: NODE_WIDTH,
        height: NODE_HEIGHT,
      },
    };
  });

  const edges: Edge[] = graphData.edges.map((edge, i) => ({
    id: `e-${i}`,
    source: edge.source,
    target: edge.target,
    type: "default",
    animated: edge.relation === "blocks",
    style: {
      stroke:
        edge.relation === "blocks"
          ? "var(--status-blocked)"
          : edge.relation === "depends_on"
            ? "var(--status-in-progress)"
            : "var(--border)",
      strokeWidth: 1.5,
    },
    label: edge.relation === "blocks" ? "blocks" : undefined,
    labelStyle: { fontSize: 10, fill: "var(--muted-foreground)" },
  }));

  return { nodes, edges };
}

function ItemNode({ data }: { data: Record<string, string> }) {
  return (
    <div
      className="bg-card border rounded-lg p-3 shadow-sm"
      style={{ borderTopColor: data.entityColor, borderTopWidth: 3 }}
    >
      <div className="flex items-center gap-1.5 mb-1">
        <span className="font-mono text-[10px]" style={{ color: data.entityColor }}>
          {data.entityType?.toUpperCase()}
        </span>
        <span
          className="w-2 h-2 rounded-full"
          style={{ backgroundColor: data.statusColor }}
        />
      </div>
      <p className="text-xs font-medium line-clamp-2 leading-tight">
        {data.label}
      </p>
    </div>
  );
}

const nodeTypes = { itemNode: ItemNode };

export default function GraphView({ graphData, focusId }: { graphData: GraphData; focusId?: string }) {
  const { nodes: layoutedNodes, edges: layoutedEdges } = useMemo(
    () => getLayoutedElements(graphData),
    [graphData]
  );

  const [nodes, , onNodesChange] = useNodesState(layoutedNodes);
  const [edges, , onEdgesChange] = useEdgesState(layoutedEdges);

  const defaultViewport = useMemo(() => {
    if (focusId) {
      const focusNode = layoutedNodes.find((n) => n.id === focusId);
      if (focusNode) {
        return {
          x: -focusNode.position.x + 400,
          y: -focusNode.position.y + 200,
          zoom: 1.2,
        };
      }
    }
    return { x: 100, y: 50, zoom: 0.8 };
  }, [focusId, layoutedNodes]);

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    const prefix = node.id.split("-")[0];
    const config = PREFIX_CONFIG[prefix];
    if (config) {
      window.location.href = `${config.route}?${node.data.entityType}=${node.id}`;
    }
  }, []);

  return (
    <div className="h-screen w-full overflow-hidden bg-background">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeClick={onNodeClick}
        nodeTypes={nodeTypes}
        defaultViewport={defaultViewport}
        fitView={!focusId}
        minZoom={0.3}
        maxZoom={2}
        proOptions={{ hideAttribution: true }}
      >
        <Background gap={20} size={1} />
        <Controls showInteractive={false} />
        <MiniMap
          nodeStrokeWidth={3}
          pannable
          zoomable
          className="!bg-card !border-border"
        />
      </ReactFlow>
    </div>
  );
}
