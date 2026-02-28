"use client";

import { useState, useMemo, useCallback, useEffect } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { useEpics } from "@/lib/hooks/use-epics";
import { useTasks } from "@/lib/hooks/use-tasks";
import { useConfig } from "@/lib/hooks/use-config";
import { useUpdateEpic } from "@/lib/hooks/use-mutations";
import { CATEGORY_CONFIG, categoryOf } from "@/lib/constants";
import { EpicDetailSheet } from "@/components/domain/epic-detail-sheet";
import { TaskDetailSheet } from "@/components/domain/task-detail-sheet";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { PriorityIndicator } from "@/components/domain/priority-indicator";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";
import { CreateDialog } from "@/components/domain/create-dialog";
import { Plus } from "lucide-react";
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  useDroppable,
  useDraggable,
  rectIntersection,
  closestCorners,
} from "@dnd-kit/core";
import type { DragStartEvent, DragEndEvent, CollisionDetection } from "@dnd-kit/core";
import type { Epic, EpicStatus, Task, Priority, StatusCategory } from "@/lib/types";

const PRIORITY_RANK: Record<Priority, number> = {
  critical: 0,
  high: 1,
  medium: 2,
  low: 3,
  someday: 4,
};

function sortByPriority(epics: Epic[]): Epic[] {
  return [...epics].sort(
    (a, b) => PRIORITY_RANK[a.priority] - PRIORITY_RANK[b.priority]
  );
}

// ── Drag-and-drop ────────────────────────────────────────────────────────

const ROADMAP_COLUMN_IDS = new Set<string>(["now", "next", "later"]);

const roadmapCollisionDetection: CollisionDetection = (args) => {
  const columnOnly = {
    ...args,
    droppableContainers: args.droppableContainers.filter((c) =>
      ROADMAP_COLUMN_IDS.has(c.id as string)
    ),
  };
  const collisions = rectIntersection(columnOnly);
  if (collisions.length > 0) return collisions;
  return closestCorners(columnOnly);
};

// ── Card Components (graduated density) ──────────────────────────────────

function NowCard({
  epic,
  tasks,
  onClick,
  isOverlay,
}: {
  epic: Epic;
  tasks: Task[];
  onClick: () => void;
  isOverlay?: boolean;
}) {
  const epicTasks = tasks.filter((t) => t.epic === epic.id);
  const percent = Math.round(epic.progress * 100);
  const { data: nowCardConfig } = useConfig();
  const nowCardWorkflow = nowCardConfig?.workflows.task;

  const {
    attributes,
    listeners,
    setNodeRef,
    isDragging,
  } = useDraggable({ id: epic.id, data: { epic } });

  // Group epic tasks by category for the status breakdown
  const BREAKDOWN_CATEGORIES: StatusCategory[] = ["active", "planned", "backlog", "completed"];
  const statusGroups = useMemo(() => {
    if (!nowCardWorkflow) return [];
    return BREAKDOWN_CATEGORIES
      .map((cat) => {
        const catStatuses = new Set(nowCardWorkflow[cat] ?? []);
        const count = epicTasks.filter((t) => catStatuses.has(t.status)).length;
        return { category: cat, label: CATEGORY_CONFIG[cat].label, count };
      })
      .filter((g) => g.count > 0);
  }, [epicTasks, nowCardWorkflow]);

  return (
    <div
      ref={isOverlay ? undefined : setNodeRef}
      style={isOverlay ? undefined : { opacity: isDragging ? 0.5 : 1 }}
      {...(isOverlay ? {} : { ...attributes, ...listeners })}
    >
      <div
        className={`rounded-lg border bg-card p-4 space-y-2 cursor-pointer hover:border-ring transition-colors ${
          isOverlay ? "shadow-lg border-primary/50" : ""
        }`}
        onClick={onClick}
      >
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <span className="font-mono text-sm shrink-0" style={{ color: "var(--entity-epic)" }}>
              {epic.id}
            </span>
            <h3 className="text-base font-semibold">{epic.title}</h3>
          </div>
          <PriorityIndicator priority={epic.priority} showLabel />
        </div>

        <div className="space-y-1.5">
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span>
              {epic.done_count}/{epic.task_count} tasks done
            </span>
            <span>{percent}%</span>
          </div>
          <Progress value={percent} className="h-2" />
        </div>

        {statusGroups.length > 0 && (
          <div className="flex flex-wrap gap-3 text-xs text-muted-foreground">
            {statusGroups.map((group) => (
              <span key={group.category} className="flex items-center gap-1">
                <span
                  className="w-2 h-2 rounded-full"
                  style={{ backgroundColor: `var(--status-category-${group.category})` }}
                />
                {group.count} {group.label.toLowerCase()}
              </span>
            ))}
          </div>
        )}

        {(epic.started || epic.target) && (
          <div className="text-xs text-muted-foreground">
            {epic.started && <span>Started: {epic.started}</span>}
            {epic.started && epic.target && <span> &middot; </span>}
            {epic.target && <span>Target: {epic.target}</span>}
          </div>
        )}
      </div>
    </div>
  );
}

function NextCard({ epic, onClick, isOverlay }: { epic: Epic; onClick: () => void; isOverlay?: boolean }) {
  const percent = Math.round(epic.progress * 100);

  const {
    attributes,
    listeners,
    setNodeRef,
    isDragging,
  } = useDraggable({ id: epic.id, data: { epic } });

  return (
    <div
      ref={isOverlay ? undefined : setNodeRef}
      style={isOverlay ? undefined : { opacity: isDragging ? 0.5 : 1 }}
      {...(isOverlay ? {} : { ...attributes, ...listeners })}
    >
      <div
        className={`rounded-lg border bg-card p-4 space-y-2 cursor-pointer hover:border-ring transition-colors ${
          isOverlay ? "shadow-lg border-primary/50" : ""
        }`}
        onClick={onClick}
      >
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <span className="font-mono text-sm shrink-0" style={{ color: "var(--entity-epic)" }}>
              {epic.id}
            </span>
            <h3 className="text-base font-semibold">{epic.title}</h3>
          </div>
          <PriorityIndicator priority={epic.priority} showLabel />
        </div>

        <div className="space-y-1">
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span>
              {epic.done_count}/{epic.task_count} tasks
            </span>
            <span>{percent}%</span>
          </div>
          <Progress value={percent} className="h-1.5" />
        </div>

        {epic.target && (
          <div className="text-xs text-muted-foreground">Target: {epic.target}</div>
        )}
      </div>
    </div>
  );
}

function LaterCard({ epic, onClick, isOverlay, isDone }: { epic: Epic; onClick: () => void; isOverlay?: boolean; isDone?: boolean }) {
  const {
    attributes,
    listeners,
    setNodeRef,
    isDragging,
  } = useDraggable({ id: epic.id, data: { epic }, disabled: isDone });

  return (
    <div
      ref={isOverlay || isDone ? undefined : setNodeRef}
      style={isOverlay || isDone ? undefined : { opacity: isDragging ? 0.5 : 1 }}
      {...(isOverlay || isDone ? {} : { ...attributes, ...listeners })}
    >
      <div
        className={`rounded-lg border bg-card p-3 flex items-center gap-3 cursor-pointer hover:border-ring transition-colors ${
          isOverlay ? "shadow-lg border-primary/50" : ""
        }`}
        onClick={onClick}
      >
        <PriorityIndicator priority={epic.priority} />
        <span
          className="font-mono text-sm shrink-0"
          style={{ color: "var(--entity-epic)" }}
        >
          {epic.id}
        </span>
        <span className="text-base font-medium truncate flex-1">{epic.title}</span>
        <span className="text-xs text-muted-foreground shrink-0">
          {epic.task_count} tasks
        </span>
      </div>
    </div>
  );
}

// ── Droppable Column ─────────────────────────────────────────────────────

function RoadmapColumn({
  status,
  label,
  children,
}: {
  status: EpicStatus;
  label: string;
  children: React.ReactNode;
}) {
  const { setNodeRef, isOver } = useDroppable({ id: status });

  return (
    <div ref={setNodeRef} className="space-y-3">
      <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
        {label}
      </h2>
      <div
        className={`space-y-3 min-h-[60px] rounded-lg p-1 transition-colors ${
          isOver ? "bg-accent/50" : ""
        }`}
      >
        {children}
      </div>
    </div>
  );
}

// ── Main Content ─────────────────────────────────────────────────────────

export function RoadmapContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const { data: epicsData, isLoading: epicsLoading, error, refetch } = useEpics();
  const { data: tasksData } = useTasks();
  const updateEpic = useUpdateEpic();
  const [showDone, setShowDone] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [activeEpic, setActiveEpic] = useState<Epic | null>(null);
  const [pendingEpics, setPendingEpics] = useState<Epic[] | null>(null);

  const selectedEpicId = searchParams.get("epic");

  const epics = epicsData ?? [];
  const tasks = tasksData ?? [];
  const displayEpics = pendingEpics ?? epics;

  // Clear optimistic state when server data arrives
  useEffect(() => {
    setPendingEpics(null);
  }, [epics]);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } })
  );

  const handleDragStart = useCallback((event: DragStartEvent) => {
    const epic = event.active.data.current?.epic as Epic | undefined;
    if (epic) setActiveEpic(epic);
  }, []);

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      setActiveEpic(null);
      const { active, over } = event;
      if (!over) return;

      const epicId = active.id as string;
      const overId = over.id as string;

      if (!ROADMAP_COLUMN_IDS.has(overId)) return;
      const targetStatus = overId as EpicStatus;

      const currentEpic = displayEpics.find((e) => e.id === epicId);
      if (currentEpic && currentEpic.status !== targetStatus) {
        setPendingEpics(
          displayEpics.map((e) =>
            e.id === epicId ? { ...e, status: targetStatus } : e
          )
        );
        updateEpic.mutate({ id: epicId, status: targetStatus });
      }
    },
    [displayEpics, updateEpic]
  );

  const { now, next, later, done } = useMemo(() => ({
    now: sortByPriority(displayEpics.filter((e) => e.status === "now")),
    next: sortByPriority(displayEpics.filter((e) => e.status === "next")),
    later: sortByPriority(displayEpics.filter((e) => e.status === "later")),
    done: displayEpics.filter((e) => e.status === "done"),
  }), [displayEpics]);

  const openEpic = (epicId: string) => {
    const params = new URLSearchParams(searchParams);
    params.set("epic", epicId);
    router.push(`/roadmap?${params.toString()}`);
  };

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
        <p className="text-muted-foreground">Failed to load roadmap.</p>
        <p className="text-sm text-destructive">{error.message}</p>
        <Button variant="outline" size="sm" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  return (
    <PageTransition>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h1 className="text-lg font-semibold">Roadmap</h1>
          <Button
            variant="outline"
            className="text-xs gap-1 cursor-pointer"
            style={{
              color: "var(--entity-epic)",
              borderColor: "var(--entity-epic)",
              backgroundColor: "color-mix(in oklch, var(--entity-epic) 8%, transparent)",
            }}
            onClick={() => setCreateOpen(true)}
          >
            <Plus className="size-3.5" /> New Epic
          </Button>
        </div>

        {epicsLoading ? (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="space-y-3">
                <Skeleton className="h-5 w-24" />
                <Skeleton className="h-32 w-full" />
                <Skeleton className="h-32 w-full" />
              </div>
            ))}
          </div>
        ) : epics.length === 0 ? (
          <EmptyState
            title="No epics yet"
            description='Create strategic epics with markplane add "title" --type epic'
          />
        ) : (
          <DndContext
            sensors={sensors}
            collisionDetection={roadmapCollisionDetection}
            onDragStart={handleDragStart}
            onDragEnd={handleDragEnd}
          >
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {/* Now lane */}
            <RoadmapColumn status="now" label={`Now (${now.length})`}>
              {now.length === 0 ? (
                <div className="rounded-lg border border-dashed p-8 text-center">
                  <p className="text-sm text-muted-foreground">No epics in progress</p>
                </div>
              ) : (
                now.map((epic) => (
                  <NowCard
                    key={epic.id}
                    epic={epic}
                    tasks={tasks}
                    onClick={() => openEpic(epic.id)}
                  />
                ))
              )}
            </RoadmapColumn>

            {/* Next lane */}
            <RoadmapColumn status="next" label={`Next (${next.length})`}>
              {next.length === 0 ? (
                <div className="rounded-lg border border-dashed p-8 text-center">
                  <p className="text-sm text-muted-foreground">
                    No epics planned next
                  </p>
                </div>
              ) : (
                next.map((epic) => (
                  <NextCard
                    key={epic.id}
                    epic={epic}
                    onClick={() => openEpic(epic.id)}
                  />
                ))
              )}
            </RoadmapColumn>

            {/* Later lane */}
            <RoadmapColumn status="later" label={`Later (${later.length})`}>
              {later.length === 0 ? (
                <div className="rounded-lg border border-dashed p-8 text-center">
                  <p className="text-sm text-muted-foreground">
                    No epics planned for later
                  </p>
                </div>
              ) : (
                later.map((epic) => (
                  <LaterCard
                    key={epic.id}
                    epic={epic}
                    onClick={() => openEpic(epic.id)}
                  />
                ))
              )}
            </RoadmapColumn>
          </div>

          {/* Completed section — not droppable */}
          {done.length > 0 && (
            <div>
              <button
                className="flex items-center gap-2 hover:text-foreground transition-colors"
                onClick={() => setShowDone(!showDone)}
              >
                <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
                  Completed ({done.length})
                </h2>
                <span className="text-xs text-muted-foreground">{showDone ? "\u25BC" : "\u25B6"}</span>
              </button>
              {showDone && (
                <div className="space-y-2 mt-3">
                  {done.map((epic) => (
                    <LaterCard
                      key={epic.id}
                      epic={epic}
                      onClick={() => openEpic(epic.id)}
                      isDone
                    />
                  ))}
                </div>
              )}
            </div>
          )}

          <DragOverlay>
            {activeEpic ? (
              <div className="w-[300px]">
                {activeEpic.status === "now" ? (
                  <NowCard epic={activeEpic} tasks={tasks} onClick={() => {}} isOverlay />
                ) : activeEpic.status === "next" ? (
                  <NextCard epic={activeEpic} onClick={() => {}} isOverlay />
                ) : (
                  <LaterCard epic={activeEpic} onClick={() => {}} isOverlay />
                )}
              </div>
            ) : null}
          </DragOverlay>
          </DndContext>
        )}

        <EpicDetailSheet
          epicId={selectedEpicId}
          open={!!selectedEpicId}
          onOpenChange={(open) => {
            if (!open) {
              const params = new URLSearchParams(searchParams);
              params.delete("epic");
              router.push(`/roadmap?${params.toString()}`);
            }
          }}
          onTaskClick={(id) => setSelectedTaskId(id)}
        />

        <TaskDetailSheet
          taskId={selectedTaskId}
          open={!!selectedTaskId}
          onOpenChange={(open) => {
            if (!open) setSelectedTaskId(null);
          }}
        />

        <CreateDialog
          kind="epic"
          open={createOpen}
          onOpenChange={setCreateOpen}
          onCreated={(id) => openEpic(id)}
        />
      </div>
    </PageTransition>
  );
}

export function RoadmapSkeleton() {
  return (
    <div className="space-y-4">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="space-y-3">
            <Skeleton className="h-5 w-24" />
            <Skeleton className="h-32 w-full" />
          </div>
        ))}
      </div>
    </div>
  );
}
