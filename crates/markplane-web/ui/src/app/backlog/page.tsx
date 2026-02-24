"use client";

import { Suspense, useState, useCallback, useMemo, useEffect } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import {
  DndContext,
  DragOverlay,
  closestCorners,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import type { DragStartEvent, DragEndEvent } from "@dnd-kit/core";
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
import { ArrowUpRight, ArrowDownLeft, Archive } from "lucide-react";
import { generateKeyBetween } from "fractional-indexing";
import type { Task, TaskStatus, Priority, Effort } from "@/lib/types";

// ── Constants ──────────────────────────────────────────────────────────────

const KANBAN_COLUMNS: { status: TaskStatus; label: string; wipLimit?: number }[] = [
  { status: "planned", label: "Planned" },
  { status: "in-progress", label: "In Progress", wipLimit: 5 },
  { status: "done", label: "Done" },
];

type ViewMode = "board" | "backlog";

const BOARD_STATUSES: TaskStatus[] = ["planned", "in-progress", "done", "cancelled"];
const BACKLOG_STATUSES: TaskStatus[] = ["draft", "backlog"];

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

type StatusFilter = "all" | "draft" | "backlog";

type SortKey = "manual" | "title" | "effort" | "epic" | "updated";

const SORT_OPTIONS: { key: SortKey; label: string }[] = [
  { key: "manual", label: "Manual" },
  { key: "updated", label: "Updated" },
  { key: "title", label: "Title" },
  { key: "effort", label: "Effort" },
  { key: "epic", label: "Epic" },
];

// ── Main Page ──────────────────────────────────────────────────────────────

export default function BacklogPage() {
  return (
    <div className="p-4 md:p-6">
      <Suspense fallback={<BacklogSkeleton />}>
        <BacklogContent />
      </Suspense>
    </div>
  );
}

function BacklogContent() {
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
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(
    searchParams.get("task")
  );

  // Sync selectedTaskId when URL changes (e.g. WikiLinkChip navigation)
  useEffect(() => {
    setSelectedTaskId(searchParams.get("task"));
  }, [searchParams]);

  const { data: tasks, isLoading, error, refetch } = useTasks();
  const { data: epics } = useEpics();

  const filteredTasks = useMemo(() => {
    if (!tasks) return [];
    const viewStatuses = view === "board" ? BOARD_STATUSES : BACKLOG_STATUSES;
    return tasks.filter((t) => {
      if (!viewStatuses.includes(t.status)) return false;
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
  }, [tasks, view, filterPriority, filterStatus, filterEpic, filterAssignee, filterTag]);

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

  if (isLoading) return <BacklogSkeleton />;

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
              className={`capitalize text-base pb-2 -mb-px transition-colors ${
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
                <SelectItem value="backlog">Backlog only</SelectItem>
                <SelectItem value="draft">Drafts only</SelectItem>
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
        <KanbanView tasks={filteredTasks} onTaskClick={openTask} />
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
    </div>
    </PageTransition>
  );
}

// ── Kanban View ────────────────────────────────────────────────────────────

function KanbanView({
  tasks,
  onTaskClick,
}: {
  tasks: Task[];
  onTaskClick: (id: string) => void;
}) {
  const updateTask = useUpdateTask();
  const archiveItem = useArchiveItem();
  const batchArchive = useBatchArchive();
  const [activeTask, setActiveTask] = useState<Task | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } })
  );

  const { tasksByStatus, totalsByStatus, cancelledTasks } = useMemo(() => {
    const map = new Map<TaskStatus, Task[]>();
    const totals = new Map<TaskStatus, number>();
    const cancelled: Task[] = [];
    for (const col of KANBAN_COLUMNS) {
      map.set(col.status, []);
      totals.set(col.status, 0);
    }
    for (const task of tasks) {
      if (task.status === "cancelled") {
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
  }, [tasks]);

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
      // The over target is either a column (droppable) or another card (sortable)
      const overId = over.id as string;

      // Determine target status
      let targetStatus: TaskStatus | null = null;
      // Check if dropped on a column directly
      const col = KANBAN_COLUMNS.find((c) => c.status === overId);
      if (col) {
        targetStatus = col.status;
      } else {
        // Dropped on a card — find which column that card belongs to
        const overTask = tasks.find((t) => t.id === overId);
        if (overTask) {
          targetStatus = overTask.status;
        }
      }

      if (!targetStatus) return;

      // Only update if status actually changed
      const currentTask = tasks.find((t) => t.id === taskId);
      if (currentTask && currentTask.status !== targetStatus) {
        updateTask.mutate({ id: taskId, status: targetStatus });
      }
    },
    [tasks, updateTask]
  );

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCorners}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
    >
      <div className="flex flex-col md:flex-row gap-4 overflow-x-auto pb-4">
        {KANBAN_COLUMNS.map((col) => {
          const columnTasks = tasksByStatus.get(col.status) || [];
          const totalCount = totalsByStatus.get(col.status) || 0;
          const isOverWip =
            col.wipLimit !== undefined && columnTasks.length > col.wipLimit;

          const isDoneColumn = col.status === "done";

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
              onDemote={col.status === "planned" ? (id) => updateTask.mutate({ id, status: "backlog" }) : undefined}
              cancelledTasks={isDoneColumn ? cancelledTasks : undefined}
              onArchive={isDoneColumn ? (id) => archiveItem.mutate(id) : undefined}
              onArchiveAll={isDoneColumn ? () => {
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
          <div className="w-[300px] opacity-90">
            <TaskCard task={activeTask} />
          </div>
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

      <SortableContext
        items={tasks.map((t) => t.id)}
        strategy={verticalListSortingStrategy}
      >
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
      </SortableContext>

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
  const updateTask = useUpdateTask();
  const [activeTask, setActiveTask] = useState<Task | null>(null);

  // Optimistic local state — updated synchronously on drop so dnd-kit
  // sees the new order before CSS transforms are removed.
  const [pendingReorder, setPendingReorder] = useState<Task[] | null>(null);
  const displayTasks = pendingReorder ?? tasks;

  // Clear optimistic state when server data arrives
  useEffect(() => {
    setPendingReorder(null);
  }, [tasks]);

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
      updateTask.mutate({ id: taskId, status: "planned" });
    },
    [updateTask]
  );

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      setActiveTask(null);
      const { active, over } = event;
      if (!over) return;

      const taskId = active.id as string;
      const overId = over.id as string;

      // Check if dropped on the promote zone
      if (overId === "promote-to-board") {
        updateTask.mutate({ id: taskId, status: "planned" });
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
        setPendingReorder(
          displayTasks.map((t) =>
            t.id === taskId
              ? {
                  ...t,
                  ...(newPosition ? { position: newPosition } : {}),
                  ...(priorityChanged ? { priority: targetPriority } : {}),
                }
              : t
          )
        );

        const updates: { id: string; priority?: Priority; position?: string } = { id: taskId };
        if (priorityChanged) updates.priority = targetPriority;
        if (newPosition) updates.position = newPosition;
        updateTask.mutate(updates);
      }
    },
    [displayTasks, tasksByPriority, updateTask, isManualSort]
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

  const isDraft = task.status === "draft";

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
        {isDraft && <StatusBadge status="draft" />}
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

function BacklogSkeleton() {
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
