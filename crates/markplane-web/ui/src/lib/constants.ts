import type { TaskStatus, EpicStatus, Priority } from "./types";

export const STATUS_CONFIG: Record<TaskStatus, { label: string; icon: string }> = {
  draft: { label: "Draft", icon: "✎" },
  backlog: { label: "Backlog", icon: "○" },
  planned: { label: "Planned", icon: "◉" },
  "in-progress": { label: "In Progress", icon: "◐" },
  done: { label: "Done", icon: "✓" },
  cancelled: { label: "Cancelled", icon: "—" },
};

export const EPIC_STATUS_CONFIG: Record<EpicStatus, { label: string; icon: string }> = {
  planned: { label: "Planned", icon: "◉" },
  active: { label: "Active", icon: "▶" },
  done: { label: "Done", icon: "✓" },
};

export const PRIORITY_CONFIG: Record<Priority, { label: string; icon: string }> = {
  critical: { label: "Critical", icon: "●" },
  high: { label: "High", icon: "●" },
  medium: { label: "Medium", icon: "◐" },
  low: { label: "Low", icon: "○" },
  someday: { label: "Someday", icon: "◌" },
};

export const PLAN_STATUS_CONFIG: Record<string, { label: string; icon: string }> = {
  draft: { label: "Draft", icon: "✎" },
  approved: { label: "Approved", icon: "✓" },
  "in-progress": { label: "In Progress", icon: "◐" },
  done: { label: "Done", icon: "✓" },
};

export const NOTE_STATUS_CONFIG: Record<string, { label: string; icon: string }> = {
  draft: { label: "Draft", icon: "✎" },
  active: { label: "Active", icon: "▶" },
  archived: { label: "Archived", icon: "—" },
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
  { href: "/graph", label: "Dependencies", icon: "GitBranch" },
  { href: "/search", label: "Search", icon: "Search" },
] as const;
