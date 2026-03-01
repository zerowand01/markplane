export type TaskStatus = string;
export type StatusCategory = "draft" | "backlog" | "planned" | "active" | "completed" | "cancelled";
export type EpicStatus = "now" | "next" | "later" | "done";
export type PlanStatus = "draft" | "approved" | "in-progress" | "done";
export type NoteStatus = "draft" | "active" | "archived";
export type Priority = "critical" | "high" | "medium" | "low" | "someday";
export type TaskType = string;
export type Effort = "xs" | "small" | "medium" | "large" | "xl";
export type NoteType = string;

export type TaskWorkflow = Record<StatusCategory, string[]>;

export interface ProjectInfo {
  name: string;
  description: string;
}

export interface ContextConfig {
  token_budget: number;
  recent_days: number;
  auto_generate: boolean;
}

export interface ProjectConfig {
  project: ProjectInfo;
  context: ContextConfig;
  documentation_paths: string[];
  task_types: string[];
  note_types: string[];
  workflows: {
    task: TaskWorkflow;
  };
}

/** Matches the backend's UpdateConfigRequest — partial at every level. */
export interface UpdateConfigRequest {
  project?: Partial<ProjectInfo>;
  context?: Partial<ContextConfig>;
  documentation_paths?: string[];
  task_types?: string[];
  note_types?: string[];
  workflows?: { task?: TaskWorkflow };
}

export interface Task {
  id: string;
  title: string;
  status: TaskStatus;
  priority: Priority;
  type: TaskType;
  effort: Effort;
  epic: string | null;
  plan: string | null;
  depends_on: string[];
  blocks: string[];
  related: string[];
  assignee: string | null;
  tags: string[];
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
  related: string[];
  tags: string[];
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
  related: string[];
  created: string;
  updated: string;
  body: string;
}

export interface Note {
  id: string;
  title: string;
  status: NoteStatus;
  type: NoteType;
  related: string[];
  tags: string[];
  created: string;
  updated: string;
  body: string;
}

export interface ProjectSummary {
  name: string;
  description: string;
  counts: {
    total: number;
    active: number;
    planned: number;
    backlog: number;
    draft: number;
    completed: number;
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

export interface DocMeta {
  slug: string;
  title: string;
}

export interface DocContent {
  slug: string;
  title: string;
  content: string;
}

export type WsEvent =
  | { type: "file_changed"; entity: "task" | "epic" | "plan" | "note"; id: string; action: "created" | "modified" | "deleted" }
  | { type: "config_changed" }
  | { type: "sync_complete" }
  | { type: "doc_changed"; slug: string }
  | { type: "connected"; version: string };
