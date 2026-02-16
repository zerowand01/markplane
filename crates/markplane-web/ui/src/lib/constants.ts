import type { LucideIcon } from "lucide-react";
import {
  Circle,
  CircleDashed,
  CircleDot,
  LoaderCircle,
  CircleCheck,
  CircleX,
  CirclePlay,
  BadgeCheck,
  Archive,
} from "lucide-react";
import type { TaskStatus, EpicStatus, Priority } from "./types";

export const STATUS_CONFIG: Record<TaskStatus, { label: string; icon: LucideIcon }> = {
  draft: { label: "Draft", icon: CircleDashed },
  backlog: { label: "Backlog", icon: Circle },
  planned: { label: "Planned", icon: CircleDot },
  "in-progress": { label: "In Progress", icon: LoaderCircle },
  done: { label: "Done", icon: CircleCheck },
  cancelled: { label: "Cancelled", icon: CircleX },
};

export const EPIC_STATUS_CONFIG: Record<EpicStatus, { label: string; icon: LucideIcon }> = {
  planned: { label: "Planned", icon: CircleDot },
  active: { label: "Active", icon: CirclePlay },
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
  approved: { label: "Approved", icon: BadgeCheck },
  "in-progress": { label: "In Progress", icon: LoaderCircle },
  done: { label: "Done", icon: CircleCheck },
};

export const NOTE_STATUS_CONFIG: Record<string, { label: string; icon: LucideIcon }> = {
  draft: { label: "Draft", icon: CircleDashed },
  active: { label: "Active", icon: CirclePlay },
  archived: { label: "Archived", icon: Archive },
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
] as const;
