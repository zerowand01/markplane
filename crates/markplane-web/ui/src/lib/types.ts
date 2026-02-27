export type TaskStatus = "draft" | "backlog" | "planned" | "in-progress" | "done" | "cancelled";
export type EpicStatus = "now" | "next" | "later" | "done";
export type PlanStatus = "draft" | "approved" | "in-progress" | "done";
export type NoteStatus = "draft" | "active" | "archived";
export type Priority = "critical" | "high" | "medium" | "low" | "someday";
export type TaskType = string;
export type Effort = "xs" | "small" | "medium" | "large" | "xl";
export type NoteType = string;

export interface ProjectConfig {
  task_types: string[];
  note_types: string[];
}

export interface Task {
  id: string;
  title: string;
  status: TaskStatus;
  priority: Priority;
  type: TaskType;
  effort: Effort;
  tags: string[];
  epic: string | null;
  plan: string | null;
  depends_on: string[];
  blocks: string[];
  related: string[];
  assignee: string | null;
  position: string | null;
  created: string;
  updated: string;
  body: string;
}

export interface Epic {
  id: string;
  title: string;
  status: EpicStatus;
  priority: Priority;
  started: string | null;
  target: string | null;
  tags: string[];
  related: string[];
  created: string;
  updated: string;
  body: string;
  task_count: number;
  done_count: number;
  progress: number;
  status_breakdown: Record<string, number>;
}

export interface Plan {
  id: string;
  title: string;
  status: PlanStatus;
  implements: string[];
  epic: string | null;
  related: string[];
  created: string;
  updated: string;
  body: string;
}

export interface Note {
  id: string;
  title: string;
  type: NoteType;
  status: NoteStatus;
  tags: string[];
  related: string[];
  created: string;
  updated: string;
  body: string;
}

export interface ProjectSummary {
  name: string;
  description: string;
  counts: {
    total: number;
    in_progress: number;
    planned: number;
    backlog: number;
    draft: number;
    done: number;
    blocked: number;
  };
  now_epics: Epic[];
  in_progress_tasks: Task[];
  blocked_tasks: Task[];
  recent_completions: Task[];
  next_up_tasks: Task[];
  context_summary: string | null;
  context_last_synced: string | null;
}

export interface CreateTaskRequest {
  title: string;
  type?: TaskType;
  priority?: Priority;
  effort?: Effort;
  epic?: string;
  tags?: string[];
}

export interface CreateEpicRequest {
  title: string;
  priority?: Priority;
}

export interface CreatePlanRequest {
  title: string;
  task_id?: string;
}

export interface CreateNoteRequest {
  title: string;
  type?: NoteType;
  tags?: string[];
}

export interface UpdateTaskRequest {
  title?: string;
  status?: TaskStatus;
  priority?: Priority;
  effort?: Effort;
  type?: TaskType;
  tags?: string[];
  epic?: string;
  plan?: string;
  assignee?: string;
  position?: string;
  depends_on?: string[];
  blocks?: string[];
  related?: string[];
  body?: string;
}

export interface UpdateEpicRequest {
  title?: string;
  status?: EpicStatus;
  priority?: Priority;
  tags?: string[];
  started?: string;
  target?: string;
  related?: string[];
  body?: string;
}

export interface UpdatePlanRequest {
  title?: string;
  status?: PlanStatus;
  epic?: string;
  related?: string[];
  body?: string;
}

export interface UpdateNoteRequest {
  title?: string;
  status?: NoteStatus;
  type?: NoteType;
  tags?: string[];
  related?: string[];
  body?: string;
}

export interface ApiResponse<T> {
  data: T;
}

export interface ApiListResponse<T> {
  data: T[];
  meta: { total: number };
}

export interface GraphNode {
  id: string;
  type: "task" | "epic" | "plan" | "note";
  title: string;
  status: string;
  priority?: string;
  tags?: string[];
}

export interface GraphEdge {
  source: string;
  target: string;
  relation: "blocks" | "depends_on" | "implements" | "epic" | "related";
}

export interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface SearchResult {
  id: string;
  entity_type: "task" | "epic" | "plan" | "note";
  title: string;
  status: string;
  priority?: string;
  snippet: string;
  score: number;
  archived: boolean;
}

export type WsEvent =
  | { type: "file_changed"; entity: "task" | "epic" | "plan" | "note"; id: string; action: "created" | "modified" | "deleted" }
  | { type: "config_changed" }
  | { type: "sync_complete" }
  | { type: "connected"; version: string };
