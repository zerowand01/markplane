"use client";

import { Suspense, useState, useCallback, useMemo } from "react";
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
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useDroppable } from "@dnd-kit/core";
import { useTasks } from "@/lib/hooks/use-tasks";
import { useEpics } from "@/lib/hooks/use-epics";
import { useUpdateTask } from "@/lib/hooks/use-mutations";
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
import { ArrowUpRight, ArrowDownLeft } from "lucide-react";
import type { Task, TaskStatus, Priority, Effort } from "@/lib/types";

// ── Constants ──────────────────────────────────────────────────────────────

const KANBAN_COLUMNS: { status: TaskStatus; label: string; wipLimit?: number }[] = [
  { status: "planned", label: "Planned" },
  { status: "in-progress", label: "In Progress", wipLimit: 5 },
  { status: "done", label: "Done" },
];

type ViewMode = "board" | "backlog";

const BOARD_STATUSES: TaskStatus[] = ["planned", "in-progress", "done"];
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

type SortKey = "title" | "effort" | "epic" | "updated";
type SortDir = "asc" | "desc";

// ── Main Page ──────────────────────────────────────────────────────────────

export default function BacklogPage() {
  return (
    <Suspense fallback={<BacklogSkeleton />}>
      <BacklogContent />
    </Suspense>
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
  const [filterEpic, setFilterEpic] = useState<string>("all");
  const [filterAssignee, setFilterAssignee] = useState<string>("all");
  const [filterTag, setFilterTag] = useState<string>("all");
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(
    searchParams.get("task")
  );

  const { data: tasks, isLoading, error, refetch } = useTasks();
  const { data: epics } = useEpics();

  const filteredTasks = useMemo(() => {
    if (!tasks) return [];
    const viewStatuses = view === "board" ? BOARD_STATUSES : BACKLOG_STATUSES;
    return tasks.filter((t) => {
      if (!viewStatuses.includes(t.status)) return false;
      if (filterPriority !== "all" && t.priority !== filterPriority)
        return false;
      if (filterEpic !== "all" && t.epic !== filterEpic) return false;
      if (filterAssignee !== "all" && t.assignee !== filterAssignee)
        return false;
      if (filterTag !== "all" && !t.tags.includes(filterTag)) return false;
      return true;
    });
  }, [tasks, view, filterPriority, filterEpic, filterAssignee, filterTag]);

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

  const openCount = (tasks || []).filter(
    (t) => t.status !== "done" && t.status !== "cancelled"
  ).length;

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
      {/* Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Backlog</h1>
          <p className="text-sm text-muted-foreground mt-1">
            {openCount} open tasks
          </p>
        </div>

        {/* View toggle */}
        <div className="flex gap-4 border-b">
          {(["board", "backlog"] as const).map((v) => (
            <button
              key={v}
              className={`text-sm capitalize pb-2 -mb-px transition-colors ${
                view === v
                  ? "text-primary border-b-2 border-primary font-medium"
                  : "text-muted-foreground hover:text-foreground"
              }`}
              onClick={() => changeView(v)}
            >
              {v}
            </button>
          ))}
        </div>
      </div>

      {/* Filter bar */}
      <div className="flex gap-2 flex-wrap overflow-x-auto pb-1">
        <Select value={filterPriority} onValueChange={setFilterPriority}>
          <SelectTrigger className="w-[140px] h-8 text-xs">
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
          <Select value={filterEpic} onValueChange={setFilterEpic}>
            <SelectTrigger className="w-[140px] h-8 text-xs">
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
            <SelectTrigger className="w-[140px] h-8 text-xs">
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
            <SelectTrigger className="w-[140px] h-8 text-xs">
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

        {(filterPriority !== "all" ||
          filterEpic !== "all" ||
          filterAssignee !== "all" ||
          filterTag !== "all") && (
          <Button
            variant="ghost"
            size="sm"
            className="text-xs h-8"
            onClick={() => {
              setFilterPriority("all");
              setFilterEpic("all");
              setFilterAssignee("all");
              setFilterTag("all");
            }}
          >
            Clear filters
          </Button>
        )}
      </div>

      {/* Views */}
      {view === "board" && (
        <KanbanView tasks={filteredTasks} onTaskClick={openTask} />
      )}
      {view === "backlog" && (
        <BacklogListView tasks={filteredTasks} onTaskClick={openTask} />
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
  const [activeTask, setActiveTask] = useState<Task | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } })
  );

  const DONE_LIMIT = 10;

  const { tasksByStatus, totalsByStatus } = useMemo(() => {
    const map = new Map<TaskStatus, Task[]>();
    const totals = new Map<TaskStatus, number>();
    for (const col of KANBAN_COLUMNS) {
      map.set(col.status, []);
      totals.set(col.status, 0);
    }
    for (const task of tasks) {
      const list = map.get(task.status);
      if (list) {
        list.push(task);
        totals.set(task.status, (totals.get(task.status) || 0) + 1);
      }
    }
    // Truncate Done column to most recent items
    const doneTasks = map.get("done");
    if (doneTasks && doneTasks.length > DONE_LIMIT) {
      doneTasks.sort((a, b) => b.updated.localeCompare(a.updated));
      map.set("done", doneTasks.slice(0, DONE_LIMIT));
    }
    return { tasksByStatus: map, totalsByStatus: totals };
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
            />
          );
        })}
      </div>

      <DragOverlay>
        {activeTask ? (
          <div className="w-[280px] opacity-90">
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
}) {
  const { setNodeRef, isOver } = useDroppable({ id: status });
  const isTruncated = totalCount > count;

  return (
    <div
      ref={setNodeRef}
      className={`min-w-[280px] flex-shrink-0 rounded-lg p-2 transition-colors ${
        isOver ? "bg-accent/50" : ""
      } md:w-auto w-full`}
    >
      <div className="flex items-center gap-2 mb-3 px-1">
        <h2 className="text-sm font-semibold">{label}</h2>
        <span
          className={`text-xs ${isOverWip ? "text-destructive font-bold" : "text-muted-foreground"}`}
        >
          ({isTruncated ? `${count} of ${totalCount}` : count}
          {wipLimit !== undefined && `/${wipLimit}`})
        </span>
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
          {tasks.length === 0 && (
            <div className="rounded-lg border border-dashed p-6 text-center">
              <p className="text-xs text-muted-foreground">No tasks</p>
            </div>
          )}
        </div>
      </SortableContext>
    </div>
  );
}

// ── Backlog List View ──────────────────────────────────────────────────────

function BacklogListView({
  tasks,
  onTaskClick,
}: {
  tasks: Task[];
  onTaskClick: (id: string) => void;
}) {
  const updateTask = useUpdateTask();
  const [activeTask, setActiveTask] = useState<Task | null>(null);
  const [sortKey, setSortKey] = useState<SortKey>("updated");
  const [sortDir, setSortDir] = useState<SortDir>("desc");

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } })
  );

  const sortTasks = useCallback(
    (list: Task[]) => {
      const sorted = [...list];
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
            cmp = a.updated.localeCompare(b.updated);
            break;
        }
        return sortDir === "asc" ? cmp : -cmp;
      });
      return sorted;
    },
    [sortKey, sortDir]
  );

  const tasksByPriority = useMemo(() => {
    const map = new Map<Priority, Task[]>();
    for (const group of PRIORITY_GROUPS) {
      map.set(group.priority, []);
    }
    for (const task of tasks) {
      const list = map.get(task.priority);
      if (list) list.push(task);
    }
    // Sort within each group
    for (const [priority, list] of map) {
      map.set(priority, sortTasks(list));
    }
    return map;
  }, [tasks, sortTasks]);

  const toggleSort = useCallback(
    (key: SortKey) => {
      if (sortKey === key) {
        setSortDir((d) => (d === "asc" ? "desc" : "asc"));
      } else {
        setSortKey(key);
        setSortDir(key === "updated" ? "desc" : "asc");
      }
    },
    [sortKey]
  );

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

      // Determine target priority
      let targetPriority: Priority | null = null;
      // Check if dropped on a priority group directly
      const group = PRIORITY_GROUPS.find((g) => g.priority === overId);
      if (group) {
        targetPriority = group.priority;
      } else {
        // Dropped on a task — find that task's priority
        const overTask = tasks.find((t) => t.id === overId);
        if (overTask) {
          targetPriority = overTask.priority;
        }
      }

      if (!targetPriority) return;

      // Only update if priority actually changed
      const currentTask = tasks.find((t) => t.id === taskId);
      if (currentTask && currentTask.priority !== targetPriority) {
        updateTask.mutate({ id: taskId, priority: targetPriority });
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
      {/* Promote drop zone */}
      <PromoteDropZone />

      {/* Sort headers (desktop) */}
      <div className="hidden md:grid grid-cols-[120px_1fr_80px_80px_100px_28px] gap-2 px-3 pb-1 mt-3 text-xs text-muted-foreground">
        <div />
        <SortableHeader label="Title" sortId="title" currentSort={sortKey} currentDir={sortDir} onToggle={toggleSort} />
        <SortableHeader label="Effort" sortId="effort" currentSort={sortKey} currentDir={sortDir} onToggle={toggleSort} />
        <SortableHeader label="Epic" sortId="epic" currentSort={sortKey} currentDir={sortDir} onToggle={toggleSort} />
        <SortableHeader label="Updated" sortId="updated" currentSort={sortKey} currentDir={sortDir} onToggle={toggleSort} />
        <div />
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
  } = useSortable({ id: task.id, data: { task } });

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

  return (
    <div
      ref={isOverlay ? undefined : setNodeRef}
      style={isOverlay ? undefined : style}
      {...(isOverlay ? {} : { ...attributes, ...listeners })}
      className={`group/row rounded-md border bg-card px-3 py-2 cursor-pointer transition-colors hover:border-muted-foreground/30 ${
        isOverlay ? "shadow-lg border-primary/50" : ""
      }`}
      onClick={onClick}
    >
      {/* Desktop layout */}
      <div className="hidden md:grid grid-cols-[120px_1fr_80px_80px_100px_28px] gap-2 items-center">
        <div className="flex items-center gap-2">
          <PriorityIndicator priority={task.priority} />
          <span className="font-mono text-xs text-muted-foreground">
            {task.id}
          </span>
        </div>
        <span className="text-sm font-medium truncate">{task.title}</span>
        {task.effort ? (
          <span className="text-[10px] font-medium px-1.5 py-0.5 rounded bg-secondary text-secondary-foreground uppercase w-fit">
            {effortLabel}
          </span>
        ) : (
          <span />
        )}
        {task.epic ? (
          <span
            className="text-[10px] font-mono px-1.5 py-0.5 rounded w-fit"
            style={{
              backgroundColor:
                "color-mix(in oklch, var(--entity-epic) 15%, transparent)",
              color: "var(--entity-epic)",
            }}
          >
            {task.epic}
          </span>
        ) : (
          <span />
        )}
        <span className="text-xs text-muted-foreground">{task.updated}</span>
        {onPromote && !isOverlay ? (
          <button
            title="Move to Board"
            className="size-6 flex items-center justify-center rounded opacity-0 group-hover/row:opacity-100 transition-opacity text-muted-foreground hover:text-primary hover:bg-primary/10"
            onClick={(e) => { e.stopPropagation(); onPromote(); }}
            onPointerDown={(e) => e.stopPropagation()}
          >
            <ArrowUpRight className="size-3.5" />
          </button>
        ) : (
          <span />
        )}
      </div>

      {/* Mobile layout */}
      <div className="md:hidden space-y-1">
        <div className="flex items-center gap-2">
          <PriorityIndicator priority={task.priority} />
          <span className="font-mono text-xs text-muted-foreground">
            {task.id}
          </span>
          <StatusBadge status={task.status} />
          {onPromote && !isOverlay && (
            <button
              title="Move to Board"
              className="ml-auto size-6 flex items-center justify-center rounded text-muted-foreground hover:text-primary hover:bg-primary/10"
              onClick={(e) => { e.stopPropagation(); onPromote(); }}
              onPointerDown={(e) => e.stopPropagation()}
            >
              <ArrowUpRight className="size-3.5" />
            </button>
          )}
        </div>
        <p className="text-sm font-medium">{task.title}</p>
        <div className="flex items-center gap-2 flex-wrap text-xs text-muted-foreground">
          {task.effort && (
            <span className="text-[10px] font-medium px-1.5 py-0.5 rounded bg-secondary text-secondary-foreground uppercase">
              {effortLabel}
            </span>
          )}
          {task.epic && (
            <span
              className="text-[10px] font-mono px-1.5 py-0.5 rounded"
              style={{
                backgroundColor:
                  "color-mix(in oklch, var(--entity-epic) 15%, transparent)",
                color: "var(--entity-epic)",
              }}
            >
              {task.epic}
            </span>
          )}
          <span>{task.updated}</span>
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
      className={`rounded-lg border-2 border-dashed p-2.5 text-center transition-colors ${
        isOver
          ? "border-primary bg-primary/5"
          : "border-muted-foreground/20"
      }`}
    >
      <p className="text-xs text-muted-foreground flex items-center justify-center gap-1.5">
        <ArrowUpRight className="size-3.5" />
        Drop to move to Board
      </p>
    </div>
  );
}

// ── Sortable Header ────────────────────────────────────────────────────────

function SortableHeader({
  label,
  sortId,
  currentSort,
  currentDir,
  onToggle,
}: {
  label: string;
  sortId: SortKey;
  currentSort: SortKey;
  currentDir: SortDir;
  onToggle: (key: SortKey) => void;
}) {
  return (
    <button
      className="text-left cursor-pointer select-none hover:text-foreground transition-colors"
      onClick={() => onToggle(sortId)}
    >
      {label}
      {currentSort === sortId && (
        <span className="ml-1">{currentDir === "asc" ? "\u2191" : "\u2193"}</span>
      )}
    </button>
  );
}

// ── Skeleton ───────────────────────────────────────────────────────────────

function BacklogSkeleton() {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <Skeleton className="h-8 w-32" />
        <div className="flex gap-4">
          <Skeleton className="h-6 w-14" />
          <Skeleton className="h-6 w-14" />
        </div>
      </div>
      <div className="flex gap-2">
        <Skeleton className="h-8 w-[140px]" />
        <Skeleton className="h-8 w-[140px]" />
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
