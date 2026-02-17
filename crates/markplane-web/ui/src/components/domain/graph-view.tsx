"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ReactFlow,
  ReactFlowProvider,
  Background,
  Controls,
  Panel,
  Handle,
  Position,
  useNodesState,
  useEdgesState,
  useReactFlow,
  type Node,
  type Edge,
} from "@xyflow/react";
import ELK from "elkjs/lib/elk.bundled.js";
import "@xyflow/react/dist/style.css";

import { PREFIX_CONFIG } from "@/lib/constants";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { GraphData, GraphNode } from "@/lib/types";
import { LoaderCircle } from "lucide-react";

// --- Constants ---

const NODE_WIDTH = 180;
const NODE_HEIGHT = 80;

const PRIORITY_WEIGHT: Record<string, number> = {
  critical: 10,
  high: 8,
  medium: 5,
  low: 2,
  someday: 1,
};

// --- Layout types ---

type LayoutDirection = "TB" | "LR";

const elk = new ELK();

// --- Layer system ---

const LAYERS = [
  { id: "dependencies", label: "Dependencies", color: "var(--status-in-progress)", relations: ["blocks", "depends_on"] },
  { id: "epics", label: "Epics", color: "var(--entity-epic)", relations: ["epic"] },
  { id: "plans", label: "Plans", color: "var(--entity-plan)", relations: ["implements"] },
  { id: "related", label: "Related", color: "var(--entity-note)", relations: ["related"] },
] as const;

const DEFAULT_LAYERS = new Set(["dependencies", "epics"]);

const RELATION_TO_LAYER: Record<string, string> = {};
for (const layer of LAYERS) {
  for (const rel of layer.relations) {
    RELATION_TO_LAYER[rel] = layer.id;
  }
}

const EDGE_STYLE: Record<string, { stroke: string; strokeWidth: number; strokeDasharray?: string }> = {
  blocks: { stroke: "var(--status-blocked)", strokeWidth: 2 },
  depends_on: { stroke: "var(--status-in-progress)", strokeWidth: 1.5 },
  epic: { stroke: "var(--entity-epic)", strokeWidth: 1, strokeDasharray: "5 3" },
  implements: { stroke: "var(--entity-plan)", strokeWidth: 1, strokeDasharray: "3 3" },
  related: { stroke: "var(--entity-note)", strokeWidth: 1, strokeDasharray: "2 3" },
};

const RELATION_LABELS: Record<string, string> = {
  blocks: "Blocks",
  depends_on: "Depends on",
  epic: "Epic membership",
  implements: "Implements",
  related: "Related",
};

// --- Helpers ---

function statusToColor(status: string): string {
  switch (status) {
    case "in-progress":
    case "active":
      return "var(--status-in-progress)";
    case "done":
      return "var(--status-done)";
    case "planned":
      return "var(--status-planned)";
    case "backlog":
      return "var(--status-backlog)";
    case "cancelled":
      return "var(--status-cancelled)";
    default:
      return "var(--status-draft)";
  }
}

function countEdgesByLayer(
  edges: GraphData["edges"],
  allowedNodes: Set<string>,
): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const e of edges) {
    if (!allowedNodes.has(e.source) || !allowedNodes.has(e.target)) continue;
    const layer = RELATION_TO_LAYER[e.relation];
    if (layer) counts[layer] = (counts[layer] ?? 0) + 1;
  }
  return counts;
}

function buildEpicMembers(edges: GraphData["edges"]): Record<string, Set<string>> {
  const map: Record<string, Set<string>> = {};
  for (const e of edges) {
    if (e.relation === "epic") {
      if (!map[e.source]) map[e.source] = new Set();
      map[e.source].add(e.target);
    }
  }
  return map;
}

// --- Filters ---

interface Filters {
  showCompleted: boolean;
  epic: string;
  priority: string;
  tag: string;
}

const DEFAULT_FILTERS: Filters = {
  showCompleted: false,
  epic: "all",
  priority: "all",
  tag: "all",
};

function computeAllowedNodes(
  graphData: GraphData,
  filters: Filters,
  epicMembers: Record<string, Set<string>>,
): Set<string> {
  const allowed = new Set<string>();
  for (const n of graphData.nodes) {
    if (!filters.showCompleted && (n.status === "done" || n.status === "cancelled")) continue;
    if (filters.priority !== "all" && n.priority && n.priority !== filters.priority) continue;
    if (filters.tag !== "all" && (!n.tags || !n.tags.includes(filters.tag))) continue;
    if (filters.epic !== "all") {
      const members = epicMembers[filters.epic];
      if (n.id !== filters.epic && (!members || !members.has(n.id))) continue;
    }
    allowed.add(n.id);
  }
  return allowed;
}

// --- Layout ---

function buildNodeData(
  node: GraphNode,
  sourcePosition: Position,
  targetPosition: Position,
) {
  const prefix = node.id.split("-")[0];
  const entityColor = PREFIX_CONFIG[prefix]?.color ?? "var(--entity-task)";
  const isDone = node.status === "done" || node.status === "cancelled";

  return {
    id: node.id,
    label: node.title,
    entityType: node.type,
    status: node.status,
    entityColor,
    statusColor: statusToColor(node.status),
    isDone: isDone ? "true" : "",
    sourcePosition,
    targetPosition,
  };
}

async function buildLayout(
  graphData: GraphData,
  activeLayers: Set<string>,
  allowedNodes: Set<string>,
  direction: LayoutDirection,
  prevPositions: Map<string, { x: number; y: number }>,
): Promise<{ nodes: Node[]; edges: Edge[] }> {
  const useCompound = activeLayers.has("epics");

  // Edges for visibility (all active layers, including epic)
  const visibilityEdges = graphData.edges.filter(
    (e) =>
      activeLayers.has(RELATION_TO_LAYER[e.relation]) &&
      allowedNodes.has(e.source) &&
      allowedNodes.has(e.target),
  );

  // Edges for rendering (suppress epic edges when compound grouping is active)
  const renderEdges = useCompound
    ? visibilityEdges.filter((e) => e.relation !== "epic")
    : visibilityEdges;

  // Visible IDs from ALL visibility edges
  const visibleIds = new Set<string>();
  for (const e of visibilityEdges) {
    visibleIds.add(e.source);
    visibleIds.add(e.target);
  }

  const filteredNodes = graphData.nodes.filter((n) => visibleIds.has(n.id));
  if (filteredNodes.length === 0) return { nodes: [], edges: [] };

  // Node lookup
  const nodeMap = new Map<string, GraphNode>();
  for (const n of filteredNodes) nodeMap.set(n.id, n);

  // Build layout options
  const elkDirection = direction === "TB" ? "DOWN" : "RIGHT";
  const hasPrev = prevPositions.size > 0;

  const layoutOptions: Record<string, string> = {
    "elk.algorithm": "layered",
    "elk.direction": elkDirection,
    "elk.spacing.nodeNode": "60",
    "elk.layered.spacing.nodeNodeBetweenLayers": "80",
    "elk.layered.crossingMinimization.strategy": "LAYER_SWEEP",
    "elk.edgeRouting": "ORTHOGONAL",
  };
  // Incremental: semi-interactive crossing minimization preserves ordering from previous positions
  if (hasPrev) {
    layoutOptions["elk.layered.crossingMinimization.semiInteractive"] = "true";
  }

  // Determine handle positions
  const sourcePosition = direction === "LR" ? Position.Right : Position.Bottom;
  const targetPosition = direction === "LR" ? Position.Left : Position.Top;

  // --- Build ELK graph ---
  type ElkChild = {
    id: string;
    width?: number;
    height?: number;
    x?: number;
    y?: number;
    layoutOptions?: Record<string, string>;
    children?: ElkChild[];
    edges?: { id: string; sources: string[]; targets: string[] }[];
  };

  let elkChildren: ElkChild[];
  let elkRootEdges: { id: string; sources: string[]; targets: string[] }[];
  // Track which nodes are children of a compound group (for React Flow parentId)
  const parentMap = new Map<string, string>();

  if (useCompound) {
    // Build epic membership: epicId -> set of member task IDs (visible only)
    const epicChildIds = new Map<string, string[]>();
    for (const e of visibilityEdges) {
      if (e.relation !== "epic") continue;
      // e.source = epic, e.target = member task
      if (!epicChildIds.has(e.source)) epicChildIds.set(e.source, []);
      epicChildIds.get(e.source)!.push(e.target);
    }

    // Only create compound nodes for epics with visible children
    const compoundEpics = new Set<string>();
    for (const [epicId, children] of epicChildIds) {
      if (children.length > 0 && nodeMap.has(epicId)) {
        compoundEpics.add(epicId);
        for (const childId of children) parentMap.set(childId, epicId);
      }
    }

    // Build children array
    elkChildren = [];

    // Compound epic containers
    for (const epicId of compoundEpics) {
      const memberIds = epicChildIds.get(epicId)!;
      const innerChildren: ElkChild[] = memberIds.map((id) => {
        const prev = prevPositions.get(id);
        return {
          id,
          width: NODE_WIDTH,
          height: NODE_HEIGHT,
          ...(prev ? { x: prev.x, y: prev.y } : {}),
          // Priority-based ordering for layered
          ...(nodeMap.get(id)?.priority
            ? { layoutOptions: { "elk.priority": String(PRIORITY_WEIGHT[nodeMap.get(id)!.priority!] ?? 5) } }
            : {}),
        };
      });

      // Collect edges internal to this compound node
      const innerEdges: { id: string; sources: string[]; targets: string[] }[] = [];
      const memberSet = new Set(memberIds);
      for (let i = 0; i < renderEdges.length; i++) {
        const e = renderEdges[i];
        if (memberSet.has(e.source) && memberSet.has(e.target)) {
          innerEdges.push({ id: `e-${i}`, sources: [e.source], targets: [e.target] });
        }
      }

      elkChildren.push({
        id: epicId,
        layoutOptions: {
          "elk.padding": "[top=36,left=12,right=12,bottom=12]",
          "elk.algorithm": "layered",
          "elk.direction": elkDirection,
          "elk.spacing.nodeNode": "40",
          "elk.layered.spacing.nodeNodeBetweenLayers": "60",
          "elk.edgeRouting": "ORTHOGONAL",
        },
        children: innerChildren,
        edges: innerEdges,
      });
    }

    // Non-compound nodes (tasks without epic, epics without children, plans, notes)
    for (const node of filteredNodes) {
      if (compoundEpics.has(node.id)) continue; // already a compound container
      if (parentMap.has(node.id)) continue; // child of a compound
      const prev = prevPositions.get(node.id);
      elkChildren.push({
        id: node.id,
        width: NODE_WIDTH,
        height: NODE_HEIGHT,
        ...(prev ? { x: prev.x, y: prev.y } : {}),
        ...(node.priority
          ? { layoutOptions: { "elk.priority": String(PRIORITY_WEIGHT[node.priority] ?? 5) } }
          : {}),
      });
    }

    // Root-level edges: any edge where source and target are NOT both in the same compound
    const usedInner = new Set<number>();
    for (const epicId of compoundEpics) {
      const memberSet = new Set(epicChildIds.get(epicId)!);
      for (let i = 0; i < renderEdges.length; i++) {
        if (memberSet.has(renderEdges[i].source) && memberSet.has(renderEdges[i].target)) {
          usedInner.add(i);
        }
      }
    }
    elkRootEdges = [];
    for (let i = 0; i < renderEdges.length; i++) {
      if (usedInner.has(i)) continue;
      elkRootEdges.push({ id: `e-${i}`, sources: [renderEdges[i].source], targets: [renderEdges[i].target] });
    }
  } else {
    // Flat layout (no compound grouping)
    elkChildren = filteredNodes.map((node) => {
      const prev = prevPositions.get(node.id);
      return {
        id: node.id,
        width: NODE_WIDTH,
        height: NODE_HEIGHT,
        ...(prev ? { x: prev.x, y: prev.y } : {}),
        // Priority-based ordering for layered
        ...(node.priority
          ? { layoutOptions: { "elk.priority": String(PRIORITY_WEIGHT[node.priority] ?? 5) } }
          : {}),
      };
    });

    elkRootEdges = renderEdges.map((edge, i) => ({
      id: `e-${i}`,
      sources: [edge.source],
      targets: [edge.target],
    }));
  }

  const elkGraph = {
    id: "root",
    layoutOptions,
    children: elkChildren,
    edges: elkRootEdges,
  };

  const elkResult = await elk.layout(elkGraph);

  // --- Extract React Flow nodes ---
  const nodes: Node[] = [];

  for (const elkChild of elkResult.children ?? []) {
    if (elkChild.children && elkChild.children.length > 0) {
      // Compound epic container
      const epicNode = nodeMap.get(elkChild.id);
      nodes.push({
        id: elkChild.id,
        position: { x: elkChild.x ?? 0, y: elkChild.y ?? 0 },
        data: {
          id: elkChild.id,
          label: epicNode?.title ?? elkChild.id,
          entityColor: PREFIX_CONFIG["EPIC"]?.color ?? "var(--entity-epic)",
          sourcePosition,
          targetPosition,
        },
        type: "epicGroup",
        style: { width: elkChild.width, height: elkChild.height },
      });

      // Children inside the compound
      for (const innerChild of elkChild.children) {
        const childNode = nodeMap.get(innerChild.id);
        if (!childNode) continue;
        nodes.push({
          id: innerChild.id,
          position: { x: innerChild.x ?? 0, y: innerChild.y ?? 0 },
          data: buildNodeData(childNode, sourcePosition, targetPosition),
          type: "itemNode",
          parentId: elkChild.id,
          extent: "parent" as const,
          style: { width: NODE_WIDTH, height: NODE_HEIGHT },
        });
      }
    } else {
      // Regular flat node
      const node = nodeMap.get(elkChild.id);
      if (!node) continue;
      nodes.push({
        id: elkChild.id,
        position: { x: elkChild.x ?? 0, y: elkChild.y ?? 0 },
        data: buildNodeData(node, sourcePosition, targetPosition),
        type: "itemNode",
        style: { width: NODE_WIDTH, height: NODE_HEIGHT },
      });
    }
  }

  // --- Extract React Flow edges ---
  const edges: Edge[] = renderEdges.map((edge, i) => {
    const style = EDGE_STYLE[edge.relation] ?? { stroke: "var(--border)", strokeWidth: 1 };
    return {
      id: `e-${i}`,
      source: edge.source,
      target: edge.target,
      type: "smoothstep",
      animated: edge.relation === "blocks",
      style,
    };
  });

  return { nodes, edges };
}

// --- Components ---

function ItemNode({ data }: { data: Record<string, string | Position> }) {
  const sourcePos = (data.sourcePosition as Position) ?? Position.Bottom;
  const targetPos = (data.targetPosition as Position) ?? Position.Top;

  return (
    <div
      className="bg-card border rounded-lg p-3 shadow-sm"
      style={{
        borderTopColor: data.entityColor as string,
        borderTopWidth: 3,
        opacity: data.isDone ? 0.4 : 1,
      }}
    >
      <Handle type="target" position={targetPos} />
      <div className="flex items-center gap-1.5 mb-1">
        <span className="font-mono text-[10px]" style={{ color: data.entityColor as string }}>
          {data.id as string}
        </span>
        <span
          className="w-2 h-2 rounded-full"
          style={{ backgroundColor: data.statusColor as string }}
        />
      </div>
      <p className="text-xs font-medium line-clamp-2 leading-tight">
        {data.label as string}
      </p>
      <Handle type="source" position={sourcePos} />
    </div>
  );
}

function EpicGroupNode({ data }: { data: Record<string, string | Position> }) {
  const sourcePos = (data.sourcePosition as Position) ?? Position.Bottom;
  const targetPos = (data.targetPosition as Position) ?? Position.Top;

  return (
    <div
      className="rounded-xl border border-dashed"
      style={{
        borderColor: data.entityColor as string,
        backgroundColor: "color-mix(in oklch, var(--card) 60%, transparent)",
        width: "100%",
        height: "100%",
      }}
    >
      {/* Invisible handles for cross-group edges */}
      <Handle type="target" position={targetPos} className="!opacity-0" />
      <div className="px-3 py-2 flex items-center gap-1.5 overflow-hidden">
        <span className="font-mono text-[10px] font-semibold shrink-0 whitespace-nowrap" style={{ color: data.entityColor as string }}>
          {data.id as string}
        </span>
        <span className="text-[11px] font-medium text-muted-foreground truncate">
          {data.label as string}
        </span>
      </div>
      <Handle type="source" position={sourcePos} className="!opacity-0" />
    </div>
  );
}

const nodeTypes = { itemNode: ItemNode, epicGroup: EpicGroupNode };

function DirectionSelector({
  direction,
  onChange,
  isLayouting,
}: {
  direction: LayoutDirection;
  onChange: (d: LayoutDirection) => void;
  isLayouting: boolean;
}) {
  return (
    <div className="flex items-center gap-1.5">
      {isLayouting && (
        <LoaderCircle className="w-3.5 h-3.5 animate-spin text-muted-foreground" />
      )}
      <Select value={direction} onValueChange={(v) => onChange(v as LayoutDirection)}>
        <SelectTrigger className="w-[110px] h-7 text-xs">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="TB">Top-down</SelectItem>
          <SelectItem value="LR">Left-right</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}

function LayoutLoadingOverlay({ isLayouting }: { isLayouting: boolean }) {
  if (!isLayouting) return null;

  return (
    <div className="absolute inset-0 z-10 flex items-center justify-center pointer-events-none">
      <div className="flex items-center gap-2 bg-card/90 backdrop-blur-sm border border-border rounded-lg px-3 py-2 shadow-sm">
        <LoaderCircle className="w-4 h-4 animate-spin text-muted-foreground" />
        <span className="text-xs text-muted-foreground">Computing layout…</span>
      </div>
    </div>
  );
}

function LayerToggles({
  activeLayers,
  onToggle,
  edgeCounts,
  showCompleted,
  onToggleCompleted,
  hasCompletedItems,
}: {
  activeLayers: Set<string>;
  onToggle: (id: string) => void;
  edgeCounts: Record<string, number>;
  showCompleted: boolean;
  onToggleCompleted: () => void;
  hasCompletedItems: boolean;
}) {
  return (
    <div className="flex gap-1.5">
      {hasCompletedItems && (
        <button
          onClick={onToggleCompleted}
          className={`px-2.5 py-1 rounded-md text-xs font-medium transition-colors cursor-pointer ${
            showCompleted
              ? "bg-card border border-border text-foreground shadow-sm"
              : "text-muted-foreground/50 hover:text-muted-foreground"
          }`}
        >
          {showCompleted ? "Hide completed" : "Show completed"}
        </button>
      )}
      {LAYERS.filter((layer) => (edgeCounts[layer.id] ?? 0) > 0).map((layer) => {
        const isActive = activeLayers.has(layer.id);
        const count = edgeCounts[layer.id] ?? 0;
        return (
          <button
            key={layer.id}
            onClick={() => onToggle(layer.id)}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium transition-colors cursor-pointer ${
              isActive
                ? "bg-card border border-border text-foreground shadow-sm"
                : "text-muted-foreground/50 hover:text-muted-foreground"
            }`}
          >
            <span
              className="w-2 h-2 rounded-full shrink-0"
              style={{
                backgroundColor: layer.color,
                opacity: isActive ? 1 : 0.3,
              }}
            />
            {layer.label}
            {count > 0 && (
              <span className="text-[10px] text-muted-foreground">{count}</span>
            )}
          </button>
        );
      })}
    </div>
  );
}

function GraphFilters({
  filters,
  onChange,
  epicOptions,
  tagOptions,
}: {
  filters: Filters;
  onChange: (f: Partial<Filters>) => void;
  epicOptions: string[];
  tagOptions: string[];
}) {
  const hasFilters = filters.epic !== "all" || filters.priority !== "all" || filters.tag !== "all";

  return (
    <div className="flex items-center gap-1.5">
      <Select value={filters.priority} onValueChange={(v) => onChange({ priority: v })}>
        <SelectTrigger className="w-[120px] h-7 text-xs">
          <SelectValue placeholder="Priority" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All priorities</SelectItem>
          <SelectItem value="critical">Critical</SelectItem>
          <SelectItem value="high">High</SelectItem>
          <SelectItem value="medium">Medium</SelectItem>
          <SelectItem value="low">Low</SelectItem>
          <SelectItem value="someday">Someday</SelectItem>
        </SelectContent>
      </Select>

      {epicOptions.length > 0 && (
        <Select value={filters.epic} onValueChange={(v) => onChange({ epic: v })}>
          <SelectTrigger className="w-[120px] h-7 text-xs">
            <SelectValue placeholder="Epic" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All epics</SelectItem>
            {epicOptions.map((e) => (
              <SelectItem key={e} value={e}>
                {e}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )}

      {tagOptions.length > 0 && (
        <Select value={filters.tag} onValueChange={(v) => onChange({ tag: v })}>
          <SelectTrigger className="w-[120px] h-7 text-xs">
            <SelectValue placeholder="Tag" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All tags</SelectItem>
            {tagOptions.map((t) => (
              <SelectItem key={t} value={t}>
                {t}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )}

      {hasFilters && (
        <button
          onClick={() => onChange(DEFAULT_FILTERS)}
          className="px-2 py-1 rounded-md text-[10px] text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
        >
          Clear
        </button>
      )}
    </div>
  );
}

function Legend({
  activeLayers,
  edgeCounts,
  useCompound,
}: {
  activeLayers: Set<string>;
  edgeCounts: Record<string, number>;
  useCompound: boolean;
}) {
  const visibleRelations = LAYERS
    .filter((l) => activeLayers.has(l.id))
    .flatMap((l) => [...l.relations])
    .filter((rel) => {
      // When compound grouping is active, epic edges are visual (no line in legend)
      if (useCompound && rel === "epic") return false;
      const layer = RELATION_TO_LAYER[rel];
      return (edgeCounts[layer] ?? 0) > 0;
    });

  if (visibleRelations.length === 0 && !(useCompound && activeLayers.has("epics"))) return null;

  return (
    <div className="bg-card/90 backdrop-blur-sm border border-border rounded-lg px-3 py-2 flex flex-col gap-1.5">
      {visibleRelations.map((rel) => {
        const style = EDGE_STYLE[rel];
        return (
          <div key={rel} className="flex items-center gap-2">
            <svg width="24" height="10" className="shrink-0">
              <line
                x1="0"
                y1="5"
                x2="24"
                y2="5"
                stroke={style.stroke}
                strokeWidth={style.strokeWidth}
                strokeDasharray={style.strokeDasharray}
              />
              {rel === "blocks" && (
                <line
                  x1="18"
                  y1="2"
                  x2="24"
                  y2="5"
                  stroke={style.stroke}
                  strokeWidth={style.strokeWidth}
                />
              )}
            </svg>
            <span className="text-[10px] text-muted-foreground">
              {RELATION_LABELS[rel]}
              {rel === "blocks" && " (animated)"}
            </span>
          </div>
        );
      })}
      {useCompound && activeLayers.has("epics") && (
        <div className="flex items-center gap-2">
          <span
            className="w-5 h-3 rounded border border-dashed shrink-0"
            style={{ borderColor: "var(--entity-epic)" }}
          />
          <span className="text-[10px] text-muted-foreground">Epic grouping</span>
        </div>
      )}
    </div>
  );
}

// --- Animated node transitions ---

const TRANSITION_CSS = `
.react-flow__node {
  transition: transform 300ms ease;
}
.react-flow__node.dragging {
  transition: none;
}
`;

// --- Inner canvas (needs useReactFlow) ---

function GraphCanvas({
  nodes,
  edges,
  onNodesChange,
  onEdgesChange,
  onNodeClick,
  focusId,
  layoutTrigger,
  activeLayers,
  edgeCounts,
  useCompound,
}: {
  nodes: Node[];
  edges: Edge[];
  onNodesChange: ReturnType<typeof useNodesState>[2];
  onEdgesChange: ReturnType<typeof useEdgesState>[2];
  onNodeClick: (event: React.MouseEvent, node: Node) => void;
  focusId?: string;
  layoutTrigger: number;
  activeLayers: Set<string>;
  edgeCounts: Record<string, number>;
  useCompound: boolean;
}) {
  const { fitView, setCenter } = useReactFlow();

  useEffect(() => {
    if (layoutTrigger === 0) return;

    // Small delay to let React Flow render the new positions
    const timer = setTimeout(() => {
      if (focusId) {
        const focusNode = nodes.find((n) => n.id === focusId);
        if (focusNode) {
          setCenter(
            focusNode.position.x + NODE_WIDTH / 2,
            focusNode.position.y + NODE_HEIGHT / 2,
            { zoom: 1.2, duration: 300 },
          );
          return;
        }
      }
      fitView({ duration: 300, padding: 0.1 });
    }, 50);

    return () => clearTimeout(timer);
  }, [layoutTrigger, focusId, nodes, fitView, setCenter]);

  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: TRANSITION_CSS }} />
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeClick={onNodeClick}
        nodeTypes={nodeTypes}
        fitView
        minZoom={0.3}
        maxZoom={2}
        colorMode="dark"
        proOptions={{ hideAttribution: true }}
      >
        <Panel position="bottom-right">
          <Legend
            activeLayers={activeLayers}
            edgeCounts={edgeCounts}
            useCompound={useCompound}
          />
        </Panel>
        <Background gap={20} size={1} />
        <Controls showInteractive={false} position="bottom-left" />
      </ReactFlow>
    </>
  );
}

// --- Main ---

export default function GraphView({
  graphData,
  focusId,
  onNodeClick: onNodeClickProp,
}: {
  graphData: GraphData;
  focusId?: string;
  onNodeClick?: (nodeId: string) => void;
}) {
  const [activeLayers, setActiveLayers] = useState<Set<string>>(DEFAULT_LAYERS);
  const [filters, setFilters] = useState<Filters>(DEFAULT_FILTERS);
  const [direction, setDirection] = useState<LayoutDirection>("TB");
  const [isLayouting, setIsLayouting] = useState(false);
  const [layoutTrigger, setLayoutTrigger] = useState(0);

  const useCompound = activeLayers.has("epics");

  const updateFilters = useCallback((partial: Partial<Filters>) => {
    setFilters((prev) => ({ ...prev, ...partial }));
  }, []);

  const toggleLayer = useCallback((id: string) => {
    setActiveLayers((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const epicMembers = useMemo(
    () => buildEpicMembers(graphData.edges),
    [graphData],
  );

  const epicOptions = useMemo(
    () => graphData.nodes.filter((n) => n.type === "epic").map((n) => n.id).sort(),
    [graphData],
  );

  const tagOptions = useMemo(() => {
    const tags = new Set<string>();
    for (const n of graphData.nodes) {
      if (n.tags) for (const t of n.tags) tags.add(t);
    }
    return [...tags].sort();
  }, [graphData]);

  const allowedNodes = useMemo(
    () => computeAllowedNodes(graphData, filters, epicMembers),
    [graphData, filters, epicMembers],
  );

  const edgeCounts = useMemo(
    () => countEdgesByLayer(graphData.edges, allowedNodes),
    [graphData, allowedNodes],
  );

  const hasCompletedItems = useMemo(
    () => graphData.nodes.some((n) => n.status === "done" || n.status === "cancelled"),
    [graphData],
  );

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  // Track layout version and previous positions for incremental layout
  const layoutVersionRef = useRef(0);
  const prevPositionsRef = useRef<Map<string, { x: number; y: number }>>(new Map());

  useEffect(() => {
    const version = ++layoutVersionRef.current;
    setIsLayouting(true);

    buildLayout(graphData, activeLayers, allowedNodes, direction, prevPositionsRef.current)
      .then((result) => {
        if (version !== layoutVersionRef.current) return;

        // Store positions for next incremental layout
        const posMap = new Map<string, { x: number; y: number }>();
        for (const node of result.nodes) {
          posMap.set(node.id, node.position);
        }
        prevPositionsRef.current = posMap;

        setNodes(result.nodes);
        setEdges(result.edges);
        setLayoutTrigger((prev) => prev + 1);
      })
      .finally(() => {
        if (version !== layoutVersionRef.current) return;
        setIsLayouting(false);
      });
  }, [graphData, activeLayers, allowedNodes, direction, setNodes, setEdges]);

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    if (onNodeClickProp) {
      onNodeClickProp(node.id);
    }
  }, [onNodeClickProp]);

  return (
    <div className="h-screen w-full overflow-hidden bg-background flex flex-col">
      <div className="flex items-center justify-between px-3 py-2 border-b border-border bg-background shrink-0">
        <LayerToggles
          activeLayers={activeLayers}
          onToggle={toggleLayer}
          edgeCounts={edgeCounts}
          showCompleted={filters.showCompleted}
          onToggleCompleted={() => updateFilters({ showCompleted: !filters.showCompleted })}
          hasCompletedItems={hasCompletedItems}
        />
        <div className="flex items-center gap-1.5">
          <DirectionSelector direction={direction} onChange={setDirection} isLayouting={isLayouting} />
          <div className="w-px h-5 bg-border mx-1" />
          <GraphFilters
            filters={filters}
            onChange={updateFilters}
            epicOptions={epicOptions}
            tagOptions={tagOptions}
          />
        </div>
      </div>
      <div className="flex-1 relative">
        <LayoutLoadingOverlay isLayouting={isLayouting} />
        <ReactFlowProvider>
          <GraphCanvas
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onNodeClick={onNodeClick}
            focusId={focusId}
            layoutTrigger={layoutTrigger}
            activeLayers={activeLayers}
            edgeCounts={edgeCounts}
            useCompound={useCompound}
          />
        </ReactFlowProvider>
      </div>
    </div>
  );
}
