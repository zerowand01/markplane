"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ReactFlow,
  ReactFlowProvider,
  Background,
  Controls,
  ControlButton,
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

import { PREFIX_CONFIG, categoryOf } from "@/lib/constants";
import { useConfig } from "@/lib/hooks/use-config";
import type { TaskWorkflow } from "@/lib/types";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { GraphData, GraphNode } from "@/lib/types";
import { ChevronDown, LoaderCircle } from "lucide-react";

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

// --- Toggle system ---
// Dependencies (blocks/depends_on) are always shown — they're the point of the graph.
// Toggles control entity type visibility: epics, plans, notes.

const LAYERS = [
  { id: "epics", label: "Epics", color: "var(--entity-epic)", relations: ["epic"] },
  { id: "plans", label: "Plans", color: "var(--entity-plan)", relations: ["implements"] },
  { id: "notes", label: "Notes", color: "var(--entity-note)", relations: [] },
  { id: "related", label: "Related", color: "var(--entity-note)", relations: ["related"] },
] as const;

const DEFAULT_LAYERS = new Set(["epics"]);

// Maps relation types to layer IDs (including dependencies for edge counting)
const RELATION_TO_LAYER: Record<string, string> = {
  blocks: "dependencies",
  depends_on: "dependencies",
  epic: "epics",
  implements: "plans",
  related: "related",
};

// Maps entity ID prefixes to their controlling layer
const PREFIX_TO_LAYER: Record<string, string> = {
  EPIC: "epics",
  PLAN: "plans",
  NOTE: "notes",
};

const DEPENDENCY_RELATIONS = new Set(["blocks", "depends_on"]);

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

function statusToColor(status: string, workflow?: TaskWorkflow): string {
  // For task statuses, derive category fallback from workflow
  if (workflow) {
    const category = categoryOf(workflow, status);
    if (category) {
      return `var(--status-${status}, var(--status-category-${category}))`;
    }
  }
  // For non-task statuses (epic, plan, note) or when workflow unavailable,
  // use direct CSS variable with muted-foreground fallback
  return `var(--status-${status}, var(--muted-foreground))`;
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
  priorities: Set<string>;
  epics: Set<string>;
  tags: Set<string>;
}

const DEFAULT_FILTERS: Filters = {
  showCompleted: false,
  priorities: new Set(),
  epics: new Set(),
  tags: new Set(),
};

/** Check if a graph node is in a closed state (completed/cancelled for tasks, done for others). */
function isNodeClosed(node: GraphNode, workflow?: TaskWorkflow): boolean {
  if (node.type === "task" && workflow) {
    const cat = categoryOf(workflow, node.status);
    return cat === "completed" || cat === "cancelled";
  }
  // Epic, plan, note — fixed statuses
  return node.status === "done" || node.status === "cancelled" || node.status === "archived";
}

function computeAllowedNodes(
  graphData: GraphData,
  filters: Filters,
  epicMembers: Record<string, Set<string>>,
  workflow?: TaskWorkflow,
): Set<string> {
  const allowed = new Set<string>();
  for (const n of graphData.nodes) {
    if (!filters.showCompleted && isNodeClosed(n, workflow)) continue;
    if (filters.priorities.size > 0 && n.priority && !filters.priorities.has(n.priority)) continue;
    if (filters.tags.size > 0 && (!n.tags || !n.tags.some((t) => filters.tags.has(t)))) continue;
    if (filters.epics.size > 0) {
      let inEpic = false;
      for (const epicId of filters.epics) {
        const members = epicMembers[epicId];
        if (n.id === epicId || (members && members.has(n.id))) { inEpic = true; break; }
      }
      if (!inEpic) continue;
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
  workflow?: TaskWorkflow,
) {
  const prefix = node.id.split("-")[0];
  const entityColor = PREFIX_CONFIG[prefix]?.color ?? "var(--entity-task)";

  return {
    id: node.id,
    label: node.title,
    entityType: node.type,
    status: node.status,
    entityColor,
    statusColor: statusToColor(node.status, workflow),
    isDone: isNodeClosed(node, workflow) ? "true" : "",
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
  workflow?: TaskWorkflow,
): Promise<{ nodes: Node[]; edges: Edge[] }> {
  const useCompound = activeLayers.has("epics");

  // Entity type gate: a node is type-allowed if its prefix has no controlling layer or that layer is active
  const isTypeAllowed = (id: string) => {
    const prefix = id.split("-")[0];
    const layer = PREFIX_TO_LAYER[prefix];
    return !layer || activeLayers.has(layer);
  };

  // Dependencies always visible; entity layer edges visible when their layer is active
  const visibilityEdges = graphData.edges.filter((e) => {
    if (!allowedNodes.has(e.source) || !allowedNodes.has(e.target)) return false;
    if (!isTypeAllowed(e.source) || !isTypeAllowed(e.target)) return false;
    if (DEPENDENCY_RELATIONS.has(e.relation)) return true;
    const layer = RELATION_TO_LAYER[e.relation];
    return layer ? activeLayers.has(layer) : false;
  });

  // Edges for rendering (suppress epic edges when compound grouping is active)
  const renderEdges = useCompound
    ? visibilityEdges.filter((e) => e.relation !== "epic")
    : visibilityEdges;

  // Tasks always visible (if they pass filters); other types visible via edges
  const edgeConnected = new Set<string>();
  for (const e of visibilityEdges) {
    edgeConnected.add(e.source);
    edgeConnected.add(e.target);
  }

  const filteredNodes = graphData.nodes.filter((n) => {
    if (!allowedNodes.has(n.id)) return false;
    if (!isTypeAllowed(n.id)) return false;
    // Tasks are always shown; other entity types need an edge to appear
    const prefix = n.id.split("-")[0];
    return !PREFIX_TO_LAYER[prefix] || edgeConnected.has(n.id);
  });
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
          data: buildNodeData(childNode, sourcePosition, targetPosition, workflow),
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
        data: buildNodeData(node, sourcePosition, targetPosition, workflow),
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

// --- Shared toolbar styles ---

const TOOLBAR_BTN_ACTIVE = "bg-card border border-border text-foreground shadow-sm";
const TOOLBAR_BTN_INACTIVE = "text-muted-foreground/50 hover:text-muted-foreground";

function toolbarBtnClass(active: boolean) {
  return `flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium transition-colors cursor-pointer ${active ? TOOLBAR_BTN_ACTIVE : TOOLBAR_BTN_INACTIVE}`;
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

const DIRECTION_LABELS: Record<LayoutDirection, string> = { TB: "Top-down", LR: "Left-right" };

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
      <DropdownMenu>
        <DropdownMenuTrigger className={toolbarBtnClass(true)}>
          {DIRECTION_LABELS[direction]}
          <ChevronDown className="w-3 h-3 opacity-50" />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          {(Object.entries(DIRECTION_LABELS) as [LayoutDirection, string][]).map(([value, label]) => (
            <DropdownMenuItem
              key={value}
              onClick={() => onChange(value)}
              className={`text-xs ${direction === value ? "font-semibold" : ""}`}
            >
              {label}
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
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
          className={toolbarBtnClass(showCompleted)}
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
            className={toolbarBtnClass(isActive)}
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

function toggleInSet(set: Set<string>, value: string): Set<string> {
  const next = new Set(set);
  if (next.has(value)) next.delete(value);
  else next.add(value);
  return next;
}

function MultiSelectFilter({
  label,
  selected,
  options,
  onChange,
}: {
  label: string;
  selected: Set<string>;
  options: string[];
  onChange: (next: Set<string>) => void;
}) {
  const count = selected.size;
  return (
    <DropdownMenu>
      <DropdownMenuTrigger className={toolbarBtnClass(count > 0)}>
        {label}
        {count > 0 && <span className="text-[10px] text-muted-foreground">({count})</span>}
        <ChevronDown className="w-3 h-3 opacity-50" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="min-w-[140px]">
        {options.map((opt) => (
          <DropdownMenuCheckboxItem
            key={opt}
            checked={selected.has(opt)}
            onCheckedChange={() => onChange(toggleInSet(selected, opt))}
            onSelect={(e) => e.preventDefault()}
            className="text-xs capitalize"
          >
            {opt}
          </DropdownMenuCheckboxItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

const PRIORITY_OPTIONS = ["critical", "high", "medium", "low", "someday"];

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
  const hasFilters = filters.priorities.size > 0 || filters.epics.size > 0 || filters.tags.size > 0;

  return (
    <div className="flex items-center gap-1.5">
      <MultiSelectFilter
        label="Priority"
        selected={filters.priorities}
        options={PRIORITY_OPTIONS}
        onChange={(priorities) => onChange({ priorities })}
      />

      {epicOptions.length > 0 && (
        <MultiSelectFilter
          label="Epic"
          selected={filters.epics}
          options={epicOptions}
          onChange={(epics) => onChange({ epics })}
        />
      )}

      {tagOptions.length > 0 && (
        <MultiSelectFilter
          label="Tag"
          selected={filters.tags}
          options={tagOptions}
          onChange={(tags) => onChange({ tags })}
        />
      )}

      {hasFilters && (
        <button
          onClick={() => onChange({ priorities: new Set(), epics: new Set(), tags: new Set() })}
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
  // Dependencies are always shown when edges exist
  const depRelations = (edgeCounts["dependencies"] ?? 0) > 0 ? ["blocks", "depends_on"] : [];

  // Entity layer relations shown when their layer is active
  const layerRelations = LAYERS
    .filter((l) => activeLayers.has(l.id))
    .flatMap((l) => [...l.relations])
    .filter((rel) => {
      if (useCompound && rel === "epic") return false;
      const layer = RELATION_TO_LAYER[rel];
      return (edgeCounts[layer] ?? 0) > 0;
    });

  const visibleRelations = [...depRelations, ...layerRelations];

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
  onResetLayout,
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
  onResetLayout: () => void;
  focusId?: string;
  layoutTrigger: number;
  activeLayers: Set<string>;
  edgeCounts: Record<string, number>;
  useCompound: boolean;
}) {
  const { fitView, setCenter } = useReactFlow();
  const nodesRef = useRef(nodes);
  nodesRef.current = nodes;

  useEffect(() => {
    if (layoutTrigger === 0) return;

    // Small delay to let React Flow render the new positions
    const timer = setTimeout(() => {
      if (focusId) {
        const focusNode = nodesRef.current.find((n) => n.id === focusId);
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [layoutTrigger, focusId]);

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
        <Controls showInteractive={false} position="bottom-left">
          <ControlButton onClick={onResetLayout} title="Reset layout">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <rect x="3" y="3" width="7" height="7" />
              <rect x="14" y="3" width="7" height="7" />
              <rect x="3" y="14" width="7" height="7" />
              <rect x="14" y="14" width="7" height="7" />
            </svg>
          </ControlButton>
        </Controls>
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
  const { data: graphConfig } = useConfig();
  const graphWorkflow = graphConfig?.workflows.task;

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
    () => computeAllowedNodes(graphData, filters, epicMembers, graphWorkflow),
    [graphData, filters, epicMembers, graphWorkflow],
  );

  const edgeCounts = useMemo(
    () => countEdgesByLayer(graphData.edges, allowedNodes),
    [graphData, allowedNodes],
  );

  const hasCompletedItems = useMemo(
    () => graphData.nodes.some((n) => isNodeClosed(n, graphWorkflow)),
    [graphData, graphWorkflow],
  );

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  // Track layout version and previous positions for incremental layout
  const layoutVersionRef = useRef(0);
  const prevPositionsRef = useRef<Map<string, { x: number; y: number }>>(new Map());
  const [layoutKey, setLayoutKey] = useState(0);

  const resetLayout = useCallback(() => {
    prevPositionsRef.current = new Map();
    setLayoutKey((prev) => prev + 1);
  }, []);

  useEffect(() => {
    const version = ++layoutVersionRef.current;
    setIsLayouting(true);

    buildLayout(graphData, activeLayers, allowedNodes, direction, prevPositionsRef.current, graphWorkflow)
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
  }, [graphData, activeLayers, allowedNodes, direction, layoutKey, setNodes, setEdges, graphWorkflow]);

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
            onResetLayout={resetLayout}
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
