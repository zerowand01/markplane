import type { LucideIcon } from "lucide-react";
import {
  Circle,
  CircleDashed,
  CircleDot,
  LoaderCircle,
  CircleCheck,
  CircleX,
  CirclePlay,
  ThumbsUp,
  Archive,
} from "lucide-react";
import type { EpicStatus, Priority, StatusCategory, TaskWorkflow } from "./types";

/** Fixed config per status category — icon and color for system-level rendering. */
export const CATEGORY_CONFIG: Record<StatusCategory, { label: string; icon: LucideIcon }> = {
  draft: { label: "Draft", icon: CircleDashed },
  backlog: { label: "Backlog", icon: Circle },
  planned: { label: "Planned", icon: CircleDot },
  active: { label: "Active", icon: LoaderCircle },
  completed: { label: "Completed", icon: CircleCheck },
  cancelled: { label: "Cancelled", icon: CircleX },
};

/** Default status config for known statuses (backward compat). */
export const STATUS_CONFIG: Record<string, { label: string; icon: LucideIcon }> = {
  draft: { label: "Draft", icon: CircleDashed },
  backlog: { label: "Backlog", icon: Circle },
  planned: { label: "Planned", icon: CircleDot },
  "in-progress": { label: "In Progress", icon: LoaderCircle },
  done: { label: "Done", icon: CircleCheck },
  cancelled: { label: "Cancelled", icon: CircleX },
};

/** Build a status→config mapping from a TaskWorkflow.
 *  Known statuses get their explicit config; custom statuses inherit from their category. */
export function buildStatusConfig(workflow: TaskWorkflow): Record<string, { label: string; icon: LucideIcon; category: StatusCategory }> {
  const result: Record<string, { label: string; icon: LucideIcon; category: StatusCategory }> = {};
  for (const [category, statuses] of Object.entries(workflow) as [StatusCategory, string[]][]) {
    const catConfig = CATEGORY_CONFIG[category];
    for (const status of statuses) {
      const known = STATUS_CONFIG[status];
      result[status] = {
        label: known?.label ?? status.split("-").map(w => w.length > 0 ? w[0].toUpperCase() + w.slice(1) : "").join(" "),
        icon: known?.icon ?? catConfig.icon,
        category,
      };
    }
  }
  return result;
}

/** Get the category for a status given a workflow. */
export function categoryOf(workflow: TaskWorkflow, status: string): StatusCategory | undefined {
  for (const [category, statuses] of Object.entries(workflow) as [StatusCategory, string[]][]) {
    if (statuses.includes(status)) return category;
  }
  return undefined;
}

/** Get all status strings from a workflow in category order. */
export function allStatuses(workflow: TaskWorkflow): string[] {
  const order: StatusCategory[] = ["draft", "backlog", "planned", "active", "completed", "cancelled"];
  return order.flatMap(cat => workflow[cat] ?? []);
}

export const EPIC_STATUS_CONFIG: Record<EpicStatus, { label: string; icon: LucideIcon }> = {
  now: { label: "Now", icon: CirclePlay },
  next: { label: "Next", icon: CircleDot },
  later: { label: "Later", icon: Circle },
  done: { label: "Done", icon: CircleCheck },
};

export const PRIORITY_CONFIG: Record<Priority, { label: string; icon: string }> = {
  critical: { label: "Critical", icon: "●" },
  high: { label: "High", icon: "●" },
  medium: { label: "Medium", icon: "◐" },
  low: { label: "Low", icon: "○" },
  someday: { label: "Someday", icon: "◌" },
};

export const PLAN_STATUS_CONFIG: Record<string, { label: string; icon: LucideIcon }> = {
  draft: { label: "Draft", icon: CircleDashed },
  approved: { label: "Approved", icon: ThumbsUp },
  "in-progress": { label: "In Progress", icon: LoaderCircle },
  done: { label: "Done", icon: CircleCheck },
};

export const NOTE_STATUS_CONFIG: Record<string, { label: string; icon: LucideIcon }> = {
  draft: { label: "Draft", icon: CircleDashed },
  active: { label: "Active", icon: CirclePlay },
  archived: { label: "Archived", icon: Archive },
};

export const NOTE_TYPE_CONFIG: Record<string, { label: string }> = {
  research: { label: "Research" },
  analysis: { label: "Analysis" },
  idea: { label: "Idea" },
  decision: { label: "Decision" },
  meeting: { label: "Meeting" },
};

export const PREFIX_CONFIG: Record<string, { route: string; apiPath: string; color: string }> = {
  TASK: { route: "/backlog", apiPath: "tasks", color: "var(--entity-task)" },
  EPIC: { route: "/roadmap", apiPath: "epics", color: "var(--entity-epic)" },
  PLAN: { route: "/plans", apiPath: "plans", color: "var(--entity-plan)" },
  NOTE: { route: "/notes", apiPath: "notes", color: "var(--entity-note)" },
};

export const NAV_ITEMS = [
  { href: "/dashboard", label: "Dashboard", icon: "LayoutDashboard" },
  { href: "/backlog", label: "Backlog", icon: "CheckSquare" },
  { href: "/plans", label: "Plans", icon: "FileText" },
  { href: "/notes", label: "Notes", icon: "Lightbulb" },
  { href: "/roadmap", label: "Roadmap", icon: "Map" },
  { href: "/graph", label: "Graph", icon: "GitBranch" },
  { href: "/docs", label: "Docs", icon: "BookOpen" },
] as const;
