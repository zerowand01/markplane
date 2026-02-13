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
import type { DragStartEvent, DragEndEvent, DragOverEvent } from "@dnd-kit/core";
import {
  SortableContext,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
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
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Card, CardContent } from "@/components/ui/card";
import { STATUS_CONFIG } from "@/lib/constants";
import { PageTransition } from "@/components/domain/page-transition";
import type { Task, TaskStatus, Priority, Effort } from "@/lib/types";

// ── Constants ──────────────────────────────────────────────────────────────

const KANBAN_COLUMNS: { status: TaskStatus; label: string; wipLimit?: number }[] = [
  { status: "in-progress", label: "In Progress", wipLimit: 5 },
  { status: "planned", label: "Planned" },
  { status: "backlog", label: "Backlog" },
  { status: "draft", label: "Drafts" },
];

type ViewMode = "kanban" | "list" | "table";

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

  const viewParam = searchParams.get("view") as ViewMode | null;
  const [view, setView] = useState<ViewMode>(viewParam || "kanban");
  const [filterStatus, setFilterStatus] = useState<string>("all");
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
    return tasks.filter((t) => {
      if (filterStatus !== "all" && t.status !== filterStatus) return false;
      if (filterPriority !== "all" && t.priority !== filterPriority)
        return false;
      if (filterEpic !== "all" && t.epic !== filterEpic) return false;
      if (filterAssignee !== "all" && t.assignee !== filterAssignee)
        return false;
      if (filterTag !== "all" && !t.tags.includes(filterTag)) return false;
      return true;
    });
  }, [tasks, filterStatus, filterPriority, filterEpic, filterAssignee, filterTag]);

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
      if (v === "kanban") {
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
        <div className="flex gap-1 bg-secondary rounded-lg p-1">
          {(["kanban", "list", "table"] as const).map((v) => (
            <Button
              key={v}
              variant={view === v ? "default" : "ghost"}
              size="sm"
              className="text-xs capitalize"
              onClick={() => changeView(v)}
            >
              {v}
            </Button>
          ))}
        </div>
      </div>

      {/* Filter bar */}
      <div className="flex gap-2 flex-wrap overflow-x-auto pb-1">
        <Select value={filterStatus} onValueChange={setFilterStatus}>
          <SelectTrigger className="w-[120px] sm:w-[140px] h-8 text-xs shrink-0">
            <SelectValue placeholder="Status" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All statuses</SelectItem>
            <SelectItem value="in-progress">In Progress</SelectItem>
            <SelectItem value="planned">Planned</SelectItem>
            <SelectItem value="backlog">Backlog</SelectItem>
            <SelectItem value="draft">Draft</SelectItem>
            <SelectItem value="done">Done</SelectItem>
            <SelectItem value="cancelled">Cancelled</SelectItem>
          </SelectContent>
        </Select>

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

        {(filterStatus !== "all" ||
          filterPriority !== "all" ||
          filterEpic !== "all" ||
          filterAssignee !== "all" ||
          filterTag !== "all") && (
          <Button
            variant="ghost"
            size="sm"
            className="text-xs h-8"
            onClick={() => {
              setFilterStatus("all");
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
      {view === "kanban" && (
        <KanbanView tasks={filteredTasks} onTaskClick={openTask} />
      )}
      {view === "list" && (
        <ListView tasks={filteredTasks} onTaskClick={openTask} />
      )}
      {view === "table" && (
        <TableView tasks={filteredTasks} onTaskClick={openTask} />
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

  const tasksByStatus = useMemo(() => {
    const map = new Map<TaskStatus, Task[]>();
    for (const col of KANBAN_COLUMNS) {
      map.set(col.status, []);
    }
    for (const task of tasks) {
      const list = map.get(task.status);
      if (list) list.push(task);
    }
    return map;
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
          const isOverWip =
            col.wipLimit !== undefined && columnTasks.length > col.wipLimit;

          return (
            <KanbanColumn
              key={col.status}
              status={col.status}
              label={col.label}
              count={columnTasks.length}
              wipLimit={col.wipLimit}
              isOverWip={isOverWip}
              tasks={columnTasks}
              onTaskClick={onTaskClick}
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
  wipLimit,
  isOverWip,
  tasks,
  onTaskClick,
}: {
  status: TaskStatus;
  label: string;
  count: number;
  wipLimit?: number;
  isOverWip: boolean;
  tasks: Task[];
  onTaskClick: (id: string) => void;
}) {
  const { setNodeRef, isOver } = useDroppable({ id: status });

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
          ({count}
          {wipLimit !== undefined && `/${wipLimit}`})
        </span>
      </div>

      <SortableContext
        items={tasks.map((t) => t.id)}
        strategy={verticalListSortingStrategy}
      >
        <div className="space-y-2 min-h-[60px]">
          {tasks.map((task) => (
            <TaskCard
              key={task.id}
              task={task}
              onClick={() => onTaskClick(task.id)}
            />
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

// ── List View ──────────────────────────────────────────────────────────────

function ListView({
  tasks,
  onTaskClick,
}: {
  tasks: Task[];
  onTaskClick: (id: string) => void;
}) {
  const activeTasks = tasks.filter(
    (t) => t.status !== "done" && t.status !== "cancelled"
  );

  return (
    <div className="space-y-2 max-w-[900px]">
      {activeTasks.map((task) => (
        <Card
          key={task.id}
          className="hover:border-muted-foreground/30 transition-colors cursor-pointer"
          onClick={() => onTaskClick(task.id)}
        >
          <CardContent className="p-3 flex items-center gap-3">
            <PriorityIndicator priority={task.priority} />
            <span className="font-mono text-xs text-muted-foreground w-20 shrink-0">
              {task.id}
            </span>
            <StatusBadge status={task.status} />
            <span className="text-sm font-medium truncate flex-1">
              {task.title}
            </span>
            {task.epic && (
              <span
                className="text-[10px] font-mono px-1.5 py-0.5 rounded shrink-0"
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
              <span className="text-[10px] font-medium px-1.5 py-0.5 rounded bg-secondary text-secondary-foreground uppercase shrink-0">
                {task.effort === "xs"
                  ? "XS"
                  : task.effort === "xl"
                    ? "XL"
                    : task.effort.charAt(0).toUpperCase()}
              </span>
            )}
            {task.assignee && (
              <span className="text-xs text-muted-foreground shrink-0">
                @{task.assignee}
              </span>
            )}
          </CardContent>
        </Card>
      ))}
      {activeTasks.length === 0 && (
        <p className="text-center text-sm text-muted-foreground py-8">
          No tasks match the current filters.
        </p>
      )}
    </div>
  );
}

// ── Table View ─────────────────────────────────────────────────────────────

function TableView({
  tasks,
  onTaskClick,
}: {
  tasks: Task[];
  onTaskClick: (id: string) => void;
}) {
  const [sortKey, setSortKey] = useState<string>("priority");
  const [sortAsc, setSortAsc] = useState(true);

  const activeTasks = tasks.filter(
    (t) => t.status !== "done" && t.status !== "cancelled"
  );

  const priorityRank: Record<Priority, number> = {
    critical: 0,
    high: 1,
    medium: 2,
    low: 3,
    someday: 4,
  };

  const effortRank: Record<Effort, number> = {
    xs: 0,
    small: 1,
    medium: 2,
    large: 3,
    xl: 4,
  };

  const sorted = useMemo(() => {
    const list = [...activeTasks];
    list.sort((a, b) => {
      let cmp = 0;
      switch (sortKey) {
        case "title":
          cmp = a.title.localeCompare(b.title);
          break;
        case "status":
          cmp = a.status.localeCompare(b.status);
          break;
        case "priority":
          cmp = priorityRank[a.priority] - priorityRank[b.priority];
          break;
        case "effort":
          cmp = effortRank[a.effort] - effortRank[b.effort];
          break;
        case "epic":
          cmp = (a.epic || "").localeCompare(b.epic || "");
          break;
        case "updated":
          cmp = a.updated.localeCompare(b.updated);
          break;
        default:
          cmp = 0;
      }
      return sortAsc ? cmp : -cmp;
    });
    return list;
  }, [activeTasks, sortKey, sortAsc]);

  const toggleSort = (key: string) => {
    if (sortKey === key) {
      setSortAsc(!sortAsc);
    } else {
      setSortKey(key);
      setSortAsc(true);
    }
  };

  const SortHeader = ({ label, sortId }: { label: string; sortId: string }) => (
    <TableHead
      className="cursor-pointer select-none hover:text-foreground"
      onClick={() => toggleSort(sortId)}
    >
      {label}
      {sortKey === sortId && (
        <span className="ml-1">{sortAsc ? "\u2191" : "\u2193"}</span>
      )}
    </TableHead>
  );

  return (
    <>
      {/* Desktop table */}
      <div className="rounded-md border hidden md:block">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[80px]">ID</TableHead>
              <SortHeader label="Title" sortId="title" />
              <SortHeader label="Status" sortId="status" />
              <SortHeader label="Priority" sortId="priority" />
              <SortHeader label="Effort" sortId="effort" />
              <SortHeader label="Epic" sortId="epic" />
              <TableHead>Assignee</TableHead>
              <SortHeader label="Updated" sortId="updated" />
            </TableRow>
          </TableHeader>
          <TableBody>
            {sorted.map((task) => (
              <TableRow
                key={task.id}
                className="cursor-pointer"
                onClick={() => onTaskClick(task.id)}
              >
                <TableCell className="font-mono text-xs text-muted-foreground">
                  {task.id}
                </TableCell>
                <TableCell className="font-medium max-w-[300px] truncate">
                  {task.title}
                </TableCell>
                <TableCell>
                  <StatusBadge status={task.status} />
                </TableCell>
                <TableCell>
                  <PriorityIndicator priority={task.priority} showLabel />
                </TableCell>
                <TableCell className="uppercase text-xs">
                  {task.effort}
                </TableCell>
                <TableCell>
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
                </TableCell>
                <TableCell className="text-xs text-muted-foreground">
                  {task.assignee ? `@${task.assignee}` : ""}
                </TableCell>
                <TableCell className="text-xs text-muted-foreground">
                  {task.updated}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
        {sorted.length === 0 && (
          <p className="text-center text-sm text-muted-foreground py-8">
            No tasks match the current filters.
          </p>
        )}
      </div>

      {/* Mobile card layout */}
      <div className="md:hidden space-y-2">
        {sorted.map((task) => (
          <Card
            key={task.id}
            className="cursor-pointer hover:border-muted-foreground/30 transition-colors"
            onClick={() => onTaskClick(task.id)}
          >
            <CardContent className="p-3 space-y-2">
              <div className="flex items-center gap-2">
                <PriorityIndicator priority={task.priority} />
                <span className="font-mono text-xs text-muted-foreground">{task.id}</span>
                <StatusBadge status={task.status} />
              </div>
              <p className="text-sm font-medium">{task.title}</p>
              <div className="flex items-center gap-2 flex-wrap text-xs text-muted-foreground">
                {task.epic && (
                  <span
                    className="text-[10px] font-mono px-1.5 py-0.5 rounded"
                    style={{
                      backgroundColor: "color-mix(in oklch, var(--entity-epic) 15%, transparent)",
                      color: "var(--entity-epic)",
                    }}
                  >
                    {task.epic}
                  </span>
                )}
                {task.assignee && <span>@{task.assignee}</span>}
                <span>{task.updated}</span>
              </div>
            </CardContent>
          </Card>
        ))}
        {sorted.length === 0 && (
          <p className="text-center text-sm text-muted-foreground py-8">
            No tasks match the current filters.
          </p>
        )}
      </div>
    </>
  );
}

// ── Skeleton ───────────────────────────────────────────────────────────────

function BacklogSkeleton() {
  return (
    <div className="space-y-4">
      <Skeleton className="h-8 w-32" />
      <div className="flex gap-2">
        <Skeleton className="h-8 w-[140px]" />
        <Skeleton className="h-8 w-[160px]" />
      </div>
      <div className="flex gap-4">
        {Array.from({ length: 4 }).map((_, i) => (
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
