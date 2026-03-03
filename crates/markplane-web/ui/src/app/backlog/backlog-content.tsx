"use client";

import { useState, useCallback, useMemo, useEffect } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import {
  DndContext,
  DragOverlay,
  closestCorners,
  rectIntersection,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import type { DragStartEvent, DragEndEvent, CollisionDetection } from "@dnd-kit/core";
import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
  defaultAnimateLayoutChanges,
} from "@dnd-kit/sortable";
import type { AnimateLayoutChanges } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useDroppable } from "@dnd-kit/core";
import { useTasks } from "@/lib/hooks/use-tasks";
import { useEpics } from "@/lib/hooks/use-epics";
import { useUpdateTask, useArchiveItem, useBatchArchive } from "@/lib/hooks/use-mutations";
import { TaskCard } from "@/components/domain/task-card";
import { TaskDetailSheet } from "@/components/domain/task-detail-sheet";
import { StatusBadge } from "@/components/domain/status-badge";
import { PriorityIndicator } from "@/components/domain/priority-indicator";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { PageTransition } from "@/components/domain/page-transition";
import { isInputFocused } from "@/lib/hooks/use-keyboard-nav";
import { ArrowUpRight, ArrowDownLeft, Archive, Plus } from "lucide-react";
import { CreateDialog } from "@/components/domain/create-dialog";
import { useConfig } from "@/lib/hooks/use-config";
import { buildStatusConfig, categoryOf } from "@/lib/constants";
import { generateKeyBetween } from "fractional-indexing";
import type { Task, TaskStatus, Priority, Effort, StatusCategory } from "@/lib/types";

// ── Constants ──────────────────────────────────────────────────────────────

type ViewMode = "board" | "backlog";

/** Categories that appear as kanban board columns. */
const BOARD_CATEGORIES: StatusCategory[] = ["planned", "active", "completed"];
/** Categories that appear in the backlog list view. */
const BACKLOG_CATEGORIES: StatusCategory[] = ["draft", "backlog"];

const PRIORITY_GROUPS: { priority: Priority; label: string }[] = [
  { priority: "critical", label: "Critical" },
  { priority: "high", label: "High" },
  { priority: "medium", label: "Medium" },
  { priority: "low", label: "Low" },
  { priority: "someday", label: "Someday" },
];

const EFFORT_RANK: Record<Effort, number> = {
  xs: 0,
  small: 1,
  medium: 2,
  large: 3,
  xl: 4,
};

type StatusFilter = "all" | string;

type SortKey = "manual" | "title" | "effort" | "epic" | "updated";

const SORT_OPTIONS: { key: SortKey; label: string }[] = [
  { key: "manual", label: "Manual" },
  { key: "updated", label: "Updated" },
  { key: "title", label: "Title" },
  { key: "effort", label: "Effort" },
  { key: "epic", label: "Epic" },
];

function makeKanbanCollisionDetection(columnIds: Set<string>): CollisionDetection {
  return (args) => {
    const columnOnly = {
      ...args,
      droppableContainers: args.droppableContainers.filter((c) =>
        columnIds.has(c.id as string)
      ),
    };
    const collisions = rectIntersection(columnOnly);
    if (collisions.length > 0) return collisions;
    return closestCorners(columnOnly);
  };
}

// ── Main Content ──────────────────────────────────────────────────────────

export function BacklogContent() {
  const searchParams = useSearchParams();
  const router = useRouter();

  // Normalize old URL params for backward compat
  const viewParam = searchParams.get("view");
  const normalizedView: ViewMode = viewParam === "backlog" ? "backlog" : "board";
  const [view, setView] = useState<ViewMode>(normalizedView);
  const [filterPriority, setFilterPriority] = useState<string>("all");
  const [filterStatus, setFilterStatus] = useState<StatusFilter>("all");
  const [filterEpic, setFilterEpic] = useState<string>("all");
  const [filterAssignee, setFilterAssignee] = useState<string>("all");
  const [filterTag, setFilterTag] = useState<string>("all");
  const [sortKey, setSortKey] = useState<SortKey>("manual");
  const [createOpen, setCreateOpen] = useState(false);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(
    searchParams.get("task")
  );

  // Sync selectedTaskId when URL changes (e.g. WikiLinkChip navigation)
  useEffect(() => {
    setSelectedTaskId(searchParams.get("task"));
  }, [searchParams]);

  const { data: tasks, isLoading, error, refetch } = useTasks();
  useEpics();
  const { data: projectConfig, isLoading: configLoading } = useConfig();
  const workflow = projectConfig?.workflows.task;

  // Derive kanban columns and status sets from workflow config
  const { kanbanColumns, boardStatuses, backlogStatuses, statusConfig } = useMemo(() => {
    if (!workflow) return { kanbanColumns: [], boardStatuses: new Set<string>(), backlogStatuses: new Set<string>(), statusConfig: {} as ReturnType<typeof buildStatusConfig> };

    const sc = buildStatusConfig(workflow);
    const cols: { status: string; label: string; wipLimit?: number }[] = [];
    const boardSet = new Set<string>();
    const backlogSet = new Set<string>();

    for (const cat of BOARD_CATEGORIES) {
      for (const status of workflow[cat] ?? []) {
        cols.push({
          status,
          label: sc[status]?.label ?? status,
          wipLimit: cat === "active" ? 5 : undefined,
        });
        boardSet.add(status);
      }
    }
    // Cancelled statuses also show on the board (in the done column as sub-section)
    for (const status of workflow.cancelled ?? []) {
      boardSet.add(status);
    }
    for (const cat of BACKLOG_CATEGORIES) {
      for (const status of workflow[cat] ?? []) {
        backlogSet.add(status);
      }
    }

    return { kanbanColumns: cols, boardStatuses: boardSet, backlogStatuses: backlogSet, statusConfig: sc };
  }, [workflow]);

  const filteredTasks = useMemo(() => {
    if (!tasks) return [];
    const viewStatuses = view === "board" ? boardStatuses : backlogStatuses;
    return tasks.filter((t) => {
      if (!viewStatuses.has(t.status)) return false;
      if (view === "backlog" && filterStatus !== "all" && t.status !== filterStatus)
        return false;
      if (filterPriority !== "all" && t.priority !== filterPriority)
        return false;
      if (filterEpic !== "all" && t.epic !== filterEpic) return false;
      if (filterAssignee !== "all" && t.assignee !== filterAssignee)
        return false;
      if (filterTag !== "all" && !t.tags.includes(filterTag)) return false;
      return true;
    });
  }, [tasks, view, filterPriority, filterStatus, filterEpic, filterAssignee, filterTag, boardStatuses, backlogStatuses]);

  const openTask = useCallback(
    (id: string) => {
      setSelectedTaskId(id);
      const params = new URLSearchParams(searchParams.toString());
      params.set("task", id);
      router.replace(`?${params.toString()}`, { scroll: false });
    },
    [router, searchParams]
  );

  const closeTask = useCallback(() => {
    setSelectedTaskId(null);
    const params = new URLSearchParams(searchParams.toString());
    params.delete("task");
    const qs = params.toString();
    router.replace(qs ? `?${qs}` : "/backlog/", { scroll: false });
  }, [router, searchParams]);

  const changeView = useCallback(
    (v: ViewMode) => {
      setView(v);
      const params = new URLSearchParams(searchParams.toString());
      if (v === "board") {
        params.delete("view");
      } else {
        params.set("view", v);
      }
      const qs = params.toString();
      router.replace(qs ? `?${qs}` : "/backlog/", { scroll: false });
    },
    [router, searchParams]
  );

  // "v" keyboard shortcut to toggle between Board and Backlog views
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key !== "v" || e.metaKey || e.ctrlKey || e.altKey || isInputFocused()) return;
      e.preventDefault();
      changeView(view === "board" ? "backlog" : "board");
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [view, changeView]);

  if (isLoading || configLoading || !workflow) return <BacklogSkeleton />;

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
        <p className="text-muted-foreground">Failed to load tasks.</p>
        <p className="text-sm text-destructive">{error.message}</p>
        <Button variant="outline" size="sm" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  const epicOptions = [
    ...new Set((tasks || []).map((t) => t.epic).filter(Boolean) as string[]),
  ];
  const assigneeOptions = [
    ...new Set(
      (tasks || []).map((t) => t.assignee).filter(Boolean) as string[]
    ),
  ];
  const tagOptions = [
    ...new Set((tasks || []).flatMap((t) => t.tags)),
  ].sort();

  return (
    <PageTransition>
    <div className="space-y-4">
      {/* Header — tabs, filters & sort on one line */}
      <div className="flex items-center gap-3 flex-wrap">
        {/* Tabs */}
        <div className="flex gap-4 border-b">
          {(["board", "backlog"] as const).map((v) => (
            <button
              key={v}
              className={`capitalize text-base pb-2 -mb-px transition-colors focus-visible:outline-none ${
                view === v
                  ? "text-primary border-b-2 border-primary font-semibold"
                  : "text-muted-foreground hover:text-foreground"
              }`}
              onClick={() => changeView(v)}
            >
              {v}
            </button>
          ))}
        </div>

        <div className="flex-1" />

        <Button
          variant="outline"
          className="text-xs gap-1 cursor-pointer"
          style={{
            color: "var(--entity-task)",
            borderColor: "var(--entity-task)",
            backgroundColor: "color-mix(in oklch, var(--entity-task) 8%, transparent)",
          }}
          onClick={() => setCreateOpen(true)}
        >
          <Plus className="size-3.5" /> New Task
        </Button>

        {/* Filters */}
        <div className="flex items-center gap-1.5">
          <span className="text-sm text-muted-foreground font-medium shrink-0">Filter</span>
          <Select value={filterPriority} onValueChange={setFilterPriority}>
            <SelectTrigger className="w-[130px] h-7 text-xs">
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

          {view === "backlog" && (
            <Select value={filterStatus} onValueChange={(v) => setFilterStatus(v as StatusFilter)}>
              <SelectTrigger className="w-[130px] h-7 text-xs">
                <SelectValue placeholder="Status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All statuses</SelectItem>
                {[...backlogStatuses].map((s) => (
                  <SelectItem key={s} value={s}>
                    {statusConfig[s]?.label ?? s} only
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}

          {epicOptions.length > 0 && (
            <Select value={filterEpic} onValueChange={setFilterEpic}>
              <SelectTrigger className="w-[130px] h-7 text-xs">
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
            <Select value={filterTag} onValueChange={setFilterTag}>
              <SelectTrigger className="w-[130px] h-7 text-xs">
                <SelectValue placeholder="Tag" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All tags</SelectItem>
                {tagOptions.map((tag) => (
                  <SelectItem key={tag} value={tag}>
                    #{tag}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}

          {assigneeOptions.length > 0 && (
            <Select value={filterAssignee} onValueChange={setFilterAssignee}>
              <SelectTrigger className="w-[130px] h-7 text-xs">
                <SelectValue placeholder="Assignee" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All assignees</SelectItem>
                {assigneeOptions.map((a) => (
                  <SelectItem key={a} value={a}>
                    @{a}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
        </div>

        {/* Sort (backlog tab only) */}
        {view === "backlog" && (
          <div className="flex items-center gap-1.5">
            <span className="text-sm text-muted-foreground font-medium shrink-0">Sort</span>
            <Select value={sortKey} onValueChange={(v) => setSortKey(v as SortKey)}>
              <SelectTrigger className="w-[100px] h-7 text-xs">
                <SelectValue placeholder="Sort" />
              </SelectTrigger>
              <SelectContent>
                {SORT_OPTIONS.map((opt) => (
                  <SelectItem key={opt.key} value={opt.key}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        {/* Clear filters */}
        {(filterPriority !== "all" ||
          filterStatus !== "all" ||
          filterEpic !== "all" ||
          filterAssignee !== "all" ||
          filterTag !== "all") && (
          <Button
            variant="ghost"
            size="sm"
            className="text-xs h-7"
            onClick={() => {
              setFilterPriority("all");
              setFilterStatus("all");
              setFilterEpic("all");
              setFilterAssignee("all");
              setFilterTag("all");
            }}
          >
            Clear
          </Button>
        )}
      </div>

      {/* Views */}
      {view === "board" && (
        <KanbanView tasks={filteredTasks} onTaskClick={openTask} kanbanColumns={kanbanColumns} workflow={workflow} />
      )}
      {view === "backlog" && (
        <BacklogListView tasks={filteredTasks} onTaskClick={openTask} sortKey={sortKey} />
      )}

      {/* Task detail sheet */}
      <TaskDetailSheet
        taskId={selectedTaskId}
        open={!!selectedTaskId}
        onOpenChange={(open) => {
          if (!open) closeTask();
        }}
      />

      <CreateDialog
        kind="task"
        open={createOpen}
        onOpenChange={setCreateOpen}
        onCreated={(id) => openTask(id)}
      />
    </div>
    </PageTransition>
  );
}

// ── Kanban View ────────────────────────────────────────────────────────────

function KanbanView({
  tasks,
  onTaskClick,
  kanbanColumns,
  workflow,
}: {
  tasks: Task[];
  onTaskClick: (id: string) => void;
  kanbanColumns: { status: string; label: string; wipLimit?: number }[];
  workflow: import("@/lib/types").TaskWorkflow;
}) {
  const updateTask = useUpdateTask();
  const archiveItem = useArchiveItem();
  const batchArchive = useBatchArchive();
  const [activeTask, setActiveTask] = useState<Task | null>(null);

  // Optimistic local state — updated synchronously on drop so the card
  // appears in the target column before the async mutation fires.
  const [pendingUpdate, setPendingUpdate] = useState<{ data: Task[]; snapshot: Task[] } | null>(null);
  const displayTasks = pendingUpdate?.snapshot === tasks ? pendingUpdate.data : tasks;

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } })
  );

  const kanbanColumnIds = useMemo(() => new Set(kanbanColumns.map(c => c.status)), [kanbanColumns]);
  const kanbanCollisionDetection = useMemo(() => makeKanbanCollisionDetection(kanbanColumnIds), [kanbanColumnIds]);

  const cancelledStatuses = useMemo(() => {
    return new Set(workflow.cancelled ?? []);
  }, [workflow]);

  const { tasksByStatus, totalsByStatus, cancelledTasks } = useMemo(() => {
    const map = new Map<TaskStatus, Task[]>();
    const totals = new Map<TaskStatus, number>();
    const cancelled: Task[] = [];
    for (const col of kanbanColumns) {
      map.set(col.status, []);
      totals.set(col.status, 0);
    }
    for (const task of displayTasks) {
      if (cancelledStatuses.has(task.status)) {
        cancelled.push(task);
        continue;
      }
      const list = map.get(task.status);
      if (list) {
        list.push(task);
        totals.set(task.status, (totals.get(task.status) || 0) + 1);
      }
    }
    return { tasksByStatus: map, totalsByStatus: totals, cancelledTasks: cancelled };
  }, [displayTasks, kanbanColumns, cancelledStatuses]);

  const handleDragStart = useCallback((event: DragStartEvent) => {
    const task = event.active.data.current?.task as Task | undefined;
    if (task) setActiveTask(task);
  }, []);

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      setActiveTask(null);
      const { active, over } = event;
      if (!over) return;

      const taskId = active.id as string;
      const overId = over.id as string;

      // Custom collision detection ensures over is always a column
      if (!kanbanColumnIds.has(overId)) return;
      const targetStatus = overId as TaskStatus;

      // Only update if status actually changed
      const currentTask = displayTasks.find((t) => t.id === taskId);
      if (currentTask && currentTask.status !== targetStatus) {
        setPendingUpdate({
          data: displayTasks.map((t) =>
            t.id === taskId ? { ...t, status: targetStatus } : t
          ),
          snapshot: tasks,
        });
        updateTask.mutate({ id: taskId, status: targetStatus });
      }
    },
    [displayTasks, updateTask, kanbanColumnIds, tasks]
  );

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={kanbanCollisionDetection}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
    >
      <div className="flex flex-col md:flex-row gap-4 overflow-x-auto pb-4">
        {kanbanColumns.map((col) => {
          const columnTasks = tasksByStatus.get(col.status) || [];
          const totalCount = totalsByStatus.get(col.status) || 0;
          const isOverWip =
            col.wipLimit !== undefined && columnTasks.length > col.wipLimit;

          const colCategory = categoryOf(workflow, col.status);
          const isCompletedColumn = colCategory === "completed";
          const isPlannedColumn = colCategory === "planned";
          // Get first backlog status for demote action
          const firstBacklogStatus = workflow.backlog[0];

          return (
            <KanbanColumn
              key={col.status}
              status={col.status}
              label={col.label}
              count={columnTasks.length}
              totalCount={totalCount}
              wipLimit={col.wipLimit}
              isOverWip={isOverWip}
              tasks={columnTasks}
              onTaskClick={onTaskClick}
              onDemote={isPlannedColumn ? (id) => updateTask.mutate({ id, status: firstBacklogStatus }) : undefined}
              cancelledTasks={isCompletedColumn ? cancelledTasks : undefined}
              onArchive={isCompletedColumn ? (id) => archiveItem.mutate(id) : undefined}
              onArchiveAll={isCompletedColumn ? () => {
                const ids = [
                  ...columnTasks.map((t) => t.id),
                  ...cancelledTasks.map((t) => t.id),
                ];
                if (ids.length > 0) batchArchive.mutate(ids);
              } : undefined}
            />
          );
        })}
      </div>

      <DragOverlay>
        {activeTask ? (
          <TaskCard task={activeTask} isOverlay />
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}

function KanbanColumn({
  status,
  label,
  count,
  totalCount,
  wipLimit,
  isOverWip,
  tasks,
  onTaskClick,
  onDemote,
  cancelledTasks,
  onArchive,
  onArchiveAll,
}: {
  status: TaskStatus;
  label: string;
  count: number;
  totalCount: number;
  wipLimit?: number;
  isOverWip: boolean;
  tasks: Task[];
  onTaskClick: (id: string) => void;
  onDemote?: (id: string) => void;
  cancelledTasks?: Task[];
  onArchive?: (id: string) => void;
  onArchiveAll?: () => void;
}) {
  const { setNodeRef, isOver } = useDroppable({ id: status });
  const isTruncated = totalCount > count;
  const hasArchivable = tasks.length > 0 || (cancelledTasks && cancelledTasks.length > 0);

  return (
    <div
      ref={setNodeRef}
      className={`min-w-0 flex-1 rounded-lg p-2 transition-colors ${
        isOver ? "bg-accent/50" : ""
      }`}
    >
      <div className="flex items-center gap-2 mb-3 px-1">
        <h2 className="text-sm font-semibold">{label}</h2>
        <span
          className={`text-xs ${isOverWip ? "text-destructive font-bold" : "text-muted-foreground"}`}
        >
          ({isTruncated ? `${count} of ${totalCount}` : count}
          {wipLimit !== undefined && `/${wipLimit}`})
        </span>
        {onArchiveAll && hasArchivable && (
          <>
            <div className="flex-1" />
            <button
              title="Archive all done & cancelled"
              className="flex items-center gap-1 text-xs text-muted-foreground hover:text-primary transition-colors cursor-pointer"
              onClick={onArchiveAll}
            >
              <Archive className="size-3" />
              Archive all
            </button>
          </>
        )}
      </div>

      <div className="space-y-2 min-h-[60px]">
        {tasks.map((task) => (
          <div key={task.id} className="group/card relative">
            <TaskCard
              task={task}
              onClick={() => onTaskClick(task.id)}
              onArchive={onArchive}
            />
            {onDemote && (
              <button
                title="Send to Backlog"
                className="absolute top-2 right-2 size-6 flex items-center justify-center rounded opacity-0 group-hover/card:opacity-100 transition-opacity text-muted-foreground hover:text-primary hover:bg-primary/10 bg-card/80"
                onClick={(e) => { e.stopPropagation(); onDemote(task.id); }}
                onPointerDown={(e) => e.stopPropagation()}
              >
                <ArrowDownLeft className="size-3.5" />
              </button>
            )}
          </div>
        ))}
        {tasks.length === 0 && !cancelledTasks?.length && (
          <div className="rounded-lg border border-dashed p-6 text-center">
            <p className="text-xs text-muted-foreground">No tasks</p>
          </div>
        )}
      </div>

      {cancelledTasks && cancelledTasks.length > 0 && (
        <div className="mt-4">
          <div className="flex items-center gap-2 mb-2 px-1">
            <h3 className="text-xs font-medium text-muted-foreground">Cancelled</h3>
            <span className="text-xs text-muted-foreground">({cancelledTasks.length})</span>
          </div>
          <div className="space-y-2">
            {cancelledTasks.map((task) => (
              <div key={task.id} className="group/card relative">
                <TaskCard
                  task={task}
                  onClick={() => onTaskClick(task.id)}
                  onArchive={onArchive}
                />
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// ── Backlog List View ──────────────────────────────────────────────────────

function BacklogListView({
  tasks,
  onTaskClick,
  sortKey,
}: {
  tasks: Task[];
  onTaskClick: (id: string) => void;
  sortKey: SortKey;
}) {
  const { data: listConfig } = useConfig();
  const listWorkflow = listConfig?.workflows.task;
  const firstPlannedStatus = listWorkflow?.planned?.[0];

  const updateTask = useUpdateTask();
  const [activeTask, setActiveTask] = useState<Task | null>(null);

  // Optimistic local state — updated synchronously on drop so dnd-kit
  // sees the new order before CSS transforms are removed.
  const [pendingReorder, setPendingReorder] = useState<{ data: Task[]; snapshot: Task[] } | null>(null);
  const displayTasks = pendingReorder?.snapshot === tasks ? pendingReorder.data : tasks;

  const isManualSort = sortKey === "manual";

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } })
  );

  const sortTasks = useCallback(
    (list: Task[]) => {
      const sorted = [...list];
      if (sortKey === "manual") {
        // Sort by position (null last), then updated desc, then id
        sorted.sort((a, b) => {
          const pa = a.position;
          const pb = b.position;
          if (pa && pb) return pa < pb ? -1 : pa > pb ? 1 : 0;
          if (pa && !pb) return -1;
          if (!pa && pb) return 1;
          const cmp = b.updated.localeCompare(a.updated);
          return cmp !== 0 ? cmp : a.id.localeCompare(b.id);
        });
      } else {
        sorted.sort((a, b) => {
          let cmp = 0;
          switch (sortKey) {
            case "title":
              cmp = a.title.localeCompare(b.title);
              break;
            case "effort":
              cmp = EFFORT_RANK[a.effort] - EFFORT_RANK[b.effort];
              break;
            case "epic":
              cmp = (a.epic || "").localeCompare(b.epic || "");
              break;
            case "updated":
              cmp = b.updated.localeCompare(a.updated);
              break;
          }
          return cmp;
        });
      }
      return sorted;
    },
    [sortKey]
  );

  const tasksByPriority = useMemo(() => {
    const map = new Map<Priority, Task[]>();
    for (const group of PRIORITY_GROUPS) {
      map.set(group.priority, []);
    }
    for (const task of displayTasks) {
      const list = map.get(task.priority);
      if (list) list.push(task);
    }
    for (const [, list] of map) {
      const sorted = sortTasks(list);
      list.length = 0;
      list.push(...sorted);
    }
    return map;
  }, [displayTasks, sortTasks]);

  const handleDragStart = useCallback((event: DragStartEvent) => {
    const task = event.active.data.current?.task as Task | undefined;
    if (task) setActiveTask(task);
  }, []);

  const handlePromote = useCallback(
    (taskId: string) => {
      if (!firstPlannedStatus) return;
      updateTask.mutate({ id: taskId, status: firstPlannedStatus });
    },
    [updateTask, firstPlannedStatus]
  );

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      setActiveTask(null);
      const { active, over } = event;
      if (!over) return;

      const taskId = active.id as string;
      const overId = over.id as string;

      // Check if dropped on the promote zone — immediately remove from local
      // state so the card disappears without a snap-back animation.
      if (overId === "promote-to-board") {
        if (!firstPlannedStatus) return;
        setPendingReorder({ data: displayTasks.filter((t) => t.id !== taskId), snapshot: tasks });
        updateTask.mutate({ id: taskId, status: firstPlannedStatus });
        return;
      }

      const currentTask = displayTasks.find((t) => t.id === taskId);
      if (!currentTask) return;

      // Determine target priority
      let targetPriority: Priority | null = null;
      const group = PRIORITY_GROUPS.find((g) => g.priority === overId);
      if (group) {
        targetPriority = group.priority;
      } else {
        const overTask = displayTasks.find((t) => t.id === overId);
        if (overTask) {
          targetPriority = overTask.priority;
        }
      }

      if (!targetPriority) return;

      const priorityChanged = currentTask.priority !== targetPriority;

      // Get the sorted task list for the target group
      const targetGroup = tasksByPriority.get(targetPriority) || [];

      // Compute new position (all tasks have positions via server initialization)
      let newPosition: string | undefined;
      if (isManualSort) {
        if (group) {
          // Dropped on empty group area — append to end
          const last = targetGroup.filter((t) => t.id !== taskId).at(-1);
          newPosition = generateKeyBetween(last?.position ?? null, null);
        } else {
          // Dropped on a specific task — insert before or after depending on drag direction
          const originalIndex = targetGroup.findIndex((t) => t.id === taskId);
          const filtered = targetGroup.filter((t) => t.id !== taskId);
          const insertAt = filtered.findIndex((t) => t.id === overId);
          if (insertAt >= 0) {
            const isDraggingDown = originalIndex !== -1 && originalIndex <= insertAt;
            if (isDraggingDown) {
              // Insert after the target task
              const before = filtered[insertAt]?.position ?? null;
              const after = filtered[insertAt + 1]?.position ?? null;
              newPosition = generateKeyBetween(before, after);
            } else {
              // Insert before the target task (cross-priority or dragging up)
              const before = insertAt > 0 ? (filtered[insertAt - 1].position ?? null) : null;
              const after = filtered[insertAt]?.position ?? null;
              newPosition = generateKeyBetween(before, after);
            }
          }
        }
      }

      // Only mutate if something changed
      if (priorityChanged || newPosition) {
        // Apply optimistic reorder synchronously — this is batched with
        // setActiveTask(null) above so React renders both in one pass,
        // before dnd-kit removes CSS transforms.
        setPendingReorder({
          data: displayTasks.map((t) =>
            t.id === taskId
              ? {
                  ...t,
                  ...(newPosition ? { position: newPosition } : {}),
                  ...(priorityChanged ? { priority: targetPriority } : {}),
                }
              : t
          ),
          snapshot: tasks,
        });

        const updates: { id: string; priority?: Priority; position?: string } = { id: taskId };
        if (priorityChanged) updates.priority = targetPriority;
        if (newPosition) updates.position = newPosition;
        updateTask.mutate(updates);
      }
    },
    [displayTasks, tasksByPriority, updateTask, isManualSort, firstPlannedStatus, tasks]
  );

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCorners}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
    >
      {/* Promote drop zone */}
      <div className="mb-3">
        <PromoteDropZone />
      </div>

      {/* Priority groups */}
      <div className="space-y-4">
        {PRIORITY_GROUPS.map((group) => {
          const groupTasks = tasksByPriority.get(group.priority) || [];
          return (
            <PriorityGroup
              key={group.priority}
              priority={group.priority}
              label={group.label}
              tasks={groupTasks}
              onTaskClick={onTaskClick}
              onPromote={handlePromote}
            />
          );
        })}
      </div>

      <DragOverlay>
        {activeTask ? (
          <BacklogRow task={activeTask} isOverlay />
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}

// ── Priority Group ─────────────────────────────────────────────────────────

function PriorityGroup({
  priority,
  label,
  tasks,
  onTaskClick,
  onPromote,
}: {
  priority: Priority;
  label: string;
  tasks: Task[];
  onTaskClick: (id: string) => void;
  onPromote: (id: string) => void;
}) {
  const { setNodeRef, isOver } = useDroppable({ id: priority });

  return (
    <div
      ref={setNodeRef}
      className={`rounded-lg p-2 transition-colors ${
        isOver ? "bg-accent/50" : ""
      }`}
    >
      <div className="flex items-center gap-2 mb-2 px-1">
        <PriorityIndicator priority={priority} />
        <h3 className="text-sm font-semibold">{label}</h3>
        <span className="text-xs text-muted-foreground">({tasks.length})</span>
      </div>

      <SortableContext
        items={tasks.map((t) => t.id)}
        strategy={verticalListSortingStrategy}
      >
        <div className="space-y-1 min-h-[36px]">
          {tasks.map((task) => (
            <BacklogRow
              key={task.id}
              task={task}
              onClick={() => onTaskClick(task.id)}
              onPromote={() => onPromote(task.id)}
            />
          ))}
          {tasks.length === 0 && (
            <div className="rounded border border-dashed p-3 text-center">
              <p className="text-xs text-muted-foreground">
                Drop tasks here to set {label.toLowerCase()} priority
              </p>
            </div>
          )}
        </div>
      </SortableContext>
    </div>
  );
}

// ── Backlog Row ────────────────────────────────────────────────────────────

// Don't animate the item that was just dropped — prevents the "slide back" effect
const skipDropAnimation: AnimateLayoutChanges = (args) => {
  if (args.wasDragging) return false;
  return defaultAnimateLayoutChanges(args);
};

function BacklogRow({
  task,
  onClick,
  onPromote,
  isOverlay,
}: {
  task: Task;
  onClick?: () => void;
  onPromote?: () => void;
  isOverlay?: boolean;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: task.id,
    data: { task },
    animateLayoutChanges: skipDropAnimation,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const effortLabel =
    task.effort === "xs"
      ? "XS"
      : task.effort === "xl"
        ? "XL"
        : task.effort.charAt(0).toUpperCase();

  const { data: rowConfig } = useConfig();
  const isDraft = rowConfig?.workflows.task
    ? categoryOf(rowConfig.workflows.task, task.status) === "draft"
    : false;

  return (
    <div
      ref={isOverlay ? undefined : setNodeRef}
      style={isOverlay ? undefined : style}
      {...(isOverlay ? {} : { ...attributes, ...listeners })}
      className={`group/row rounded-md border px-3 py-2 cursor-pointer transition-colors hover:border-muted-foreground/30 ${
        isDraft
          ? "bg-amber-50 border-amber-200/60 dark:bg-amber-950/30 dark:border-amber-800/40"
          : "bg-card"
      } ${isOverlay ? "shadow-lg border-primary/50" : ""}`}
      onClick={onClick}
    >
      {/* Desktop layout */}
      <div className="hidden md:flex items-center gap-3">
        <PriorityIndicator priority={task.priority} />
        <span className="font-mono text-sm text-muted-foreground w-24 shrink-0">
          {task.id}
        </span>
        {isDraft && <StatusBadge status={task.status} />}
        <span className="text-base font-medium truncate flex-1">{task.title}</span>
        {task.epic && (
          <span
            className="text-sm font-mono px-2 py-0.5 rounded shrink-0"
            style={{
              backgroundColor:
                "color-mix(in oklch, var(--entity-epic) 15%, transparent)",
              color: "var(--entity-epic)",
            }}
          >
            {task.epic}
          </span>
        )}
        {task.effort && (
          <span className="text-sm font-medium w-8 text-center py-0.5 rounded bg-secondary text-secondary-foreground uppercase shrink-0">
            {effortLabel}
          </span>
        )}
        {onPromote && !isOverlay ? (
          <button
            title="Move to Board"
            className="size-6 flex items-center justify-center rounded opacity-0 group-hover/row:opacity-100 transition-opacity text-muted-foreground hover:text-primary hover:bg-primary/10 cursor-pointer shrink-0"
            onClick={(e) => { e.stopPropagation(); onPromote(); }}
            onPointerDown={(e) => e.stopPropagation()}
          >
            <ArrowUpRight className="size-3.5" />
          </button>
        ) : null}
      </div>

      {/* Mobile layout */}
      <div className="md:hidden space-y-1">
        <div className="flex items-center gap-2">
          <PriorityIndicator priority={task.priority} />
          <span className="font-mono text-sm text-muted-foreground">
            {task.id}
          </span>
          <StatusBadge status={task.status} />
          {onPromote && !isOverlay && (
            <button
              title="Move to Board"
              className="ml-auto size-6 flex items-center justify-center rounded text-muted-foreground hover:text-primary hover:bg-primary/10 cursor-pointer"
              onClick={(e) => { e.stopPropagation(); onPromote(); }}
              onPointerDown={(e) => e.stopPropagation()}
            >
              <ArrowUpRight className="size-3.5" />
            </button>
          )}
        </div>
        <p className="text-base font-medium">{task.title}</p>
        <div className="flex items-center gap-2 flex-wrap text-sm text-muted-foreground">
          {task.epic && (
            <span
              className="text-sm font-mono px-2 py-0.5 rounded"
              style={{
                backgroundColor:
                  "color-mix(in oklch, var(--entity-epic) 15%, transparent)",
                color: "var(--entity-epic)",
              }}
            >
              {task.epic}
            </span>
          )}
          {task.effort && (
            <span className="text-sm font-medium px-2 py-0.5 rounded bg-secondary text-secondary-foreground uppercase">
              {effortLabel}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

// ── Promote Drop Zone ──────────────────────────────────────────────────────

function PromoteDropZone() {
  const { setNodeRef, isOver } = useDroppable({ id: "promote-to-board" });

  return (
    <div
      ref={setNodeRef}
      className={`rounded-lg border-2 border-dashed py-5 px-4 text-center transition-colors ${
        isOver
          ? "border-primary bg-primary/10"
          : "border-muted-foreground/25 bg-muted/30"
      }`}
    >
      <p className={`text-sm flex items-center justify-center gap-2 transition-colors ${
        isOver ? "text-primary font-medium" : "text-muted-foreground"
      }`}>
        <ArrowUpRight className="size-4" />
        Drop here to move to Board
      </p>
    </div>
  );
}

// ── Skeleton ───────────────────────────────────────────────────────────────

export function BacklogSkeleton() {
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <Skeleton className="h-7 w-32" />
        <div className="flex-1" />
        <Skeleton className="h-7 w-[130px]" />
        <Skeleton className="h-7 w-[130px]" />
      </div>
      <div className="flex gap-4">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="min-w-[280px] space-y-2">
            <Skeleton className="h-5 w-24" />
            {Array.from({ length: 3 }).map((_, j) => (
              <Skeleton key={j} className="h-28" />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
