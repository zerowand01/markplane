"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ReactFlow,
  ReactFlowProvider,
  Background,
  Controls,
  MiniMap,
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
import type { GraphData } from "@/lib/types";
import { LoaderCircle } from "lucide-react";

// --- Constants ---

const NODE_WIDTH = 180;
const NODE_HEIGHT = 80;

// --- Layout types ---

type LayoutAlgorithm = "layered" | "force";
type LayoutDirection = "TB" | "LR";

interface LayoutConfig {
  algorithm: LayoutAlgorithm;
  direction: LayoutDirection;
}

const DEFAULT_LAYOUT: LayoutConfig = { algorithm: "layered", direction: "TB" };

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

// --- Critical path ---

function findCriticalPath(allEdges: GraphData["edges"], doneNodes: Set<string>): Set<string> {
  const seen = new Set<string>();
  const depEdges: Array<{ source: string; target: string }> = [];

  for (const e of allEdges) {
    if (e.relation !== "blocks" && e.relation !== "depends_on") continue;
    if (doneNodes.has(e.source) || doneNodes.has(e.target)) continue;
    const key = `${e.source}->${e.target}`;
    if (seen.has(key)) continue;
    seen.add(key);
    depEdges.push({ source: e.source, target: e.target });
  }

  if (depEdges.length === 0) return new Set();

  const adj: Record<string, string[]> = {};
  const allNodes = new Set<string>();
  const inDegree: Record<string, number> = {};

  for (const e of depEdges) {
    allNodes.add(e.source);
    allNodes.add(e.target);
    if (!adj[e.source]) adj[e.source] = [];
    adj[e.source].push(e.target);
    inDegree[e.target] = (inDegree[e.target] ?? 0) + 1;
  }
  for (const node of allNodes) {
    if (!(node in inDegree)) inDegree[node] = 0;
  }

  const queue: string[] = [];
  const dist: Record<string, number> = {};
  const prev: Record<string, string | null> = {};

  for (const node of allNodes) {
    dist[node] = 0;
    prev[node] = null;
    if (inDegree[node] === 0) queue.push(node);
  }

  while (queue.length > 0) {
    const node = queue.shift()!;
    for (const next of adj[node] ?? []) {
      if (dist[node] + 1 > dist[next]) {
        dist[next] = dist[node] + 1;
        prev[next] = node;
      }
      inDegree[next]--;
      if (inDegree[next] === 0) queue.push(next);
    }
  }

  let maxNode = "";
  let maxDist = 0;
  for (const [node, d] of Object.entries(dist)) {
    if (d > maxDist) {
      maxDist = d;
      maxNode = node;
    }
  }

  if (maxDist === 0) return new Set();

  const path = new Set<string>();
  let current: string | null = maxNode;
  while (current) {
    path.add(current);
    current = prev[current];
  }

  return path;
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

async function buildLayout(
  graphData: GraphData,
  activeLayers: Set<string>,
  allowedNodes: Set<string>,
  criticalPath: Set<string>,
  layoutConfig: LayoutConfig,
): Promise<{ nodes: Node[]; edges: Edge[] }> {
  // Filter edges: active layer + both endpoints allowed
  const filteredEdges = graphData.edges.filter(
    (e) =>
      activeLayers.has(RELATION_TO_LAYER[e.relation]) &&
      allowedNodes.has(e.source) &&
      allowedNodes.has(e.target),
  );

  const visibleIds = new Set<string>();
  for (const e of filteredEdges) {
    visibleIds.add(e.source);
    visibleIds.add(e.target);
  }

  const filteredNodes = graphData.nodes.filter((n) => visibleIds.has(n.id));
  if (filteredNodes.length === 0) return { nodes: [], edges: [] };

  // Build ELK layout options based on config
  const layoutOptions: Record<string, string> =
    layoutConfig.algorithm === "layered"
      ? {
          "elk.algorithm": "layered",
          "elk.direction": layoutConfig.direction === "TB" ? "DOWN" : "RIGHT",
          "elk.spacing.nodeNode": "60",
          "elk.layered.spacing.nodeNodeBetweenLayers": "80",
          "elk.layered.crossingMinimization.strategy": "LAYER_SWEEP",
        }
      : {
          "elk.algorithm": "stress",
          "elk.stress.desiredEdgeLength": "150",
          "elk.spacing.nodeNode": "60",
        };

  const elkGraph = {
    id: "root",
    layoutOptions,
    children: filteredNodes.map((node) => ({
      id: node.id,
      width: NODE_WIDTH,
      height: NODE_HEIGHT,
    })),
    edges: filteredEdges.map((edge, i) => ({
      id: `e-${i}`,
      sources: [edge.source],
      targets: [edge.target],
    })),
  };

  const elkResult = await elk.layout(elkGraph);

  const showCritical = activeLayers.has("dependencies");

  // Determine handle positions based on direction
  const sourcePosition = layoutConfig.algorithm === "layered" && layoutConfig.direction === "LR"
    ? Position.Right
    : Position.Bottom;
  const targetPosition = layoutConfig.algorithm === "layered" && layoutConfig.direction === "LR"
    ? Position.Left
    : Position.Top;

  const nodes: Node[] = filteredNodes.map((node) => {
    const elkNode = elkResult.children?.find((n) => n.id === node.id);
    const prefix = node.id.split("-")[0];
    const entityColor = PREFIX_CONFIG[prefix]?.color ?? "var(--entity-task)";
    const isDone = node.status === "done" || node.status === "cancelled";
    const isCritical = showCritical && criticalPath.has(node.id);

    return {
      id: node.id,
      position: { x: elkNode?.x ?? 0, y: elkNode?.y ?? 0 },
      data: {
        id: node.id,
        label: node.title,
        entityType: node.type,
        status: node.status,
        entityColor,
        statusColor: statusToColor(node.status),
        isDone: isDone ? "true" : "",
        isCritical: isCritical ? "true" : "",
        sourcePosition,
        targetPosition,
      },
      type: "itemNode",
      style: { width: NODE_WIDTH, height: NODE_HEIGHT },
    };
  });

  const edges: Edge[] = filteredEdges.map((edge, i) => {
    const style = EDGE_STYLE[edge.relation] ?? { stroke: "var(--border)", strokeWidth: 1 };
    return {
      id: `e-${i}`,
      source: edge.source,
      target: edge.target,
      type: "default",
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
        boxShadow: data.isCritical
          ? "0 0 0 1.5px var(--priority-high), 0 0 12px color-mix(in oklch, var(--priority-high) 30%, transparent)"
          : undefined,
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
        {data.isCritical && (
          <span className="ml-auto text-[9px] font-medium" style={{ color: "var(--priority-high)" }}>
            critical path
          </span>
        )}
      </div>
      <p className="text-xs font-medium line-clamp-2 leading-tight">
        {data.label as string}
      </p>
      <Handle type="source" position={sourcePos} />
    </div>
  );
}

const nodeTypes = { itemNode: ItemNode };

function LayoutSelector({
  layout,
  onChange,
  isLayouting,
}: {
  layout: LayoutConfig;
  onChange: (l: Partial<LayoutConfig>) => void;
  isLayouting: boolean;
}) {
  return (
    <div className="flex items-center gap-1.5">
      {isLayouting && (
        <LoaderCircle className="w-3.5 h-3.5 animate-spin text-muted-foreground" />
      )}
      <Select
        value={layout.algorithm}
        onValueChange={(v) => onChange({ algorithm: v as LayoutAlgorithm })}
      >
        <SelectTrigger className="w-[130px] h-7 text-xs">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="layered">Hierarchical</SelectItem>
          <SelectItem value="force">Force</SelectItem>
        </SelectContent>
      </Select>
      {layout.algorithm === "layered" && (
        <Select
          value={layout.direction}
          onValueChange={(v) => onChange({ direction: v as LayoutDirection })}
        >
          <SelectTrigger className="w-[110px] h-7 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="TB">Top-down</SelectItem>
            <SelectItem value="LR">Left-right</SelectItem>
          </SelectContent>
        </Select>
      )}
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
  criticalPathSize,
}: {
  activeLayers: Set<string>;
  edgeCounts: Record<string, number>;
  criticalPathSize: number;
}) {
  const visibleRelations = LAYERS
    .filter((l) => activeLayers.has(l.id))
    .flatMap((l) => [...l.relations])
    .filter((rel) => {
      const layer = RELATION_TO_LAYER[rel];
      return (edgeCounts[layer] ?? 0) > 0;
    });

  if (visibleRelations.length === 0) return null;

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
      {activeLayers.has("dependencies") && criticalPathSize > 0 && (
        <div className="flex items-center gap-2 pt-0.5 border-t border-border/50">
          <span
            className="w-3 h-3 rounded shrink-0 border"
            style={{
              borderColor: "var(--priority-high)",
              boxShadow: "0 0 6px color-mix(in oklch, var(--priority-high) 40%, transparent)",
            }}
          />
          <span className="text-[10px] text-muted-foreground">
            Critical path ({criticalPathSize})
          </span>
        </div>
      )}
    </div>
  );
}

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
  criticalPathSize,
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
  criticalPathSize: number;
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
      <Panel position="top-right">
        <Legend
          activeLayers={activeLayers}
          edgeCounts={edgeCounts}
          criticalPathSize={criticalPathSize}
        />
      </Panel>
      <Background gap={20} size={1} />
      <Controls showInteractive={false} position="bottom-right" />
      <MiniMap nodeStrokeWidth={3} pannable zoomable position="bottom-left" />
    </ReactFlow>
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
  const [layout, setLayout] = useState<LayoutConfig>(DEFAULT_LAYOUT);
  const [isLayouting, setIsLayouting] = useState(false);
  const [layoutTrigger, setLayoutTrigger] = useState(0);

  const updateFilters = useCallback((partial: Partial<Filters>) => {
    setFilters((prev) => ({ ...prev, ...partial }));
  }, []);

  const updateLayout = useCallback((partial: Partial<LayoutConfig>) => {
    setLayout((prev) => ({ ...prev, ...partial }));
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

  const doneNodes = useMemo(() => {
    const set = new Set<string>();
    for (const n of graphData.nodes) {
      if (n.status === "done" || n.status === "cancelled") set.add(n.id);
    }
    return set;
  }, [graphData]);

  const criticalPath = useMemo(
    () => findCriticalPath(graphData.edges, doneNodes),
    [graphData, doneNodes],
  );

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  // Ref to track the latest layout version and discard stale results
  const layoutVersionRef = useRef(0);

  useEffect(() => {
    const version = ++layoutVersionRef.current;
    setIsLayouting(true);

    buildLayout(graphData, activeLayers, allowedNodes, criticalPath, layout)
      .then((result) => {
        // Discard if a newer layout was triggered
        if (version !== layoutVersionRef.current) return;
        setNodes(result.nodes);
        setEdges(result.edges);
        setLayoutTrigger((prev) => prev + 1);
      })
      .finally(() => {
        if (version !== layoutVersionRef.current) return;
        setIsLayouting(false);
      });
  }, [graphData, activeLayers, allowedNodes, criticalPath, layout, setNodes, setEdges]);

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    if (onNodeClickProp) {
      onNodeClickProp(node.id);
    }
  }, [onNodeClickProp]);

  return (
    <div className="h-screen w-full overflow-hidden bg-background flex flex-col">
      <div className="flex items-center justify-between px-3 py-2 border-b border-border bg-background shrink-0">
        <div className="flex items-center gap-1.5">
          <LayoutSelector layout={layout} onChange={updateLayout} isLayouting={isLayouting} />
          <div className="w-px h-5 bg-border mx-1" />
          <LayerToggles
            activeLayers={activeLayers}
            onToggle={toggleLayer}
            edgeCounts={edgeCounts}
            showCompleted={filters.showCompleted}
            onToggleCompleted={() => updateFilters({ showCompleted: !filters.showCompleted })}
            hasCompletedItems={doneNodes.size > 0}
          />
        </div>
        <GraphFilters
          filters={filters}
          onChange={updateFilters}
          epicOptions={epicOptions}
          tagOptions={tagOptions}
        />
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
            criticalPathSize={criticalPath.size}
          />
        </ReactFlowProvider>
      </div>
    </div>
  );
}
