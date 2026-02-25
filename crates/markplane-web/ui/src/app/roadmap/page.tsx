"use client";

import { Suspense, useState, useMemo } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { useEpics } from "@/lib/hooks/use-epics";
import { useTasks } from "@/lib/hooks/use-tasks";
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
import type { Epic, Task, Priority, TaskStatus } from "@/lib/types";

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

// ── Card Components (graduated density) ──────────────────────────────────

function NowCard({
  epic,
  tasks,
  onClick,
}: {
  epic: Epic;
  tasks: Task[];
  onClick: () => void;
}) {
  const epicTasks = tasks.filter((t) => t.epic === epic.id);
  const percent = Math.round(epic.progress * 100);

  const statusGroups = (
    [
      { status: "in-progress" as TaskStatus, label: "In Progress", count: epicTasks.filter((t) => t.status === "in-progress").length },
      { status: "planned" as TaskStatus, label: "Planned", count: epicTasks.filter((t) => t.status === "planned").length },
      { status: "backlog" as TaskStatus, label: "Backlog", count: epicTasks.filter((t) => t.status === "backlog").length },
      { status: "done" as TaskStatus, label: "Done", count: epicTasks.filter((t) => t.status === "done").length },
    ] as const
  ).filter((g) => g.count > 0);

  return (
    <div
      className="rounded-lg border bg-card p-4 space-y-2 cursor-pointer hover:border-ring transition-colors"
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
            <span key={group.status} className="flex items-center gap-1">
              <span
                className="w-2 h-2 rounded-full"
                style={{ backgroundColor: `var(--status-${group.status})` }}
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
  );
}

function NextCard({ epic, onClick }: { epic: Epic; onClick: () => void }) {
  const percent = Math.round(epic.progress * 100);

  return (
    <div
      className="rounded-lg border bg-card p-4 space-y-2 cursor-pointer hover:border-ring transition-colors"
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
  );
}

function LaterCard({ epic, onClick }: { epic: Epic; onClick: () => void }) {
  return (
    <div
      className="rounded-lg border bg-card p-3 flex items-center gap-3 cursor-pointer hover:border-ring transition-colors"
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
  );
}

// ── Main Content ─────────────────────────────────────────────────────────

function RoadmapContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const { data: epicsData, isLoading: epicsLoading, error, refetch } = useEpics();
  const { data: tasksData } = useTasks();
  const [showDone, setShowDone] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);

  const selectedEpicId = searchParams.get("epic");

  const epics = epicsData ?? [];
  const tasks = tasksData ?? [];

  const { now, next, later, done } = useMemo(() => ({
    now: sortByPriority(epics.filter((e) => e.status === "now")),
    next: sortByPriority(epics.filter((e) => e.status === "next")),
    later: sortByPriority(epics.filter((e) => e.status === "later")),
    done: epics.filter((e) => e.status === "done"),
  }), [epics]);

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
          <>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {/* Now lane */}
            <div className="space-y-3">
              <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
                Now ({now.length})
              </h2>
              {now.length === 0 ? (
                <div className="rounded-lg border border-dashed p-8 text-center">
                  <p className="text-sm text-muted-foreground">No epics in progress</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {now.map((epic) => (
                    <NowCard
                      key={epic.id}
                      epic={epic}
                      tasks={tasks}
                      onClick={() => openEpic(epic.id)}
                    />
                  ))}
                </div>
              )}
            </div>

            {/* Next lane */}
            <div className="space-y-3">
              <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
                Next ({next.length})
              </h2>
              {next.length === 0 ? (
                <div className="rounded-lg border border-dashed p-8 text-center">
                  <p className="text-sm text-muted-foreground">
                    No epics planned next
                  </p>
                </div>
              ) : (
                <div className="space-y-3">
                  {next.map((epic) => (
                    <NextCard
                      key={epic.id}
                      epic={epic}
                      onClick={() => openEpic(epic.id)}
                    />
                  ))}
                </div>
              )}
            </div>

            {/* Later lane */}
            <div className="space-y-3">
              <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
                Later ({later.length})
              </h2>
              {later.length === 0 ? (
                <div className="rounded-lg border border-dashed p-8 text-center">
                  <p className="text-sm text-muted-foreground">
                    No epics planned for later
                  </p>
                </div>
              ) : (
                <div className="space-y-3">
                  {later.map((epic) => (
                    <LaterCard
                      key={epic.id}
                      epic={epic}
                      onClick={() => openEpic(epic.id)}
                    />
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Completed section */}
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
                    />
                  ))}
                </div>
              )}
            </div>
          )}
          </>
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

export default function RoadmapPage() {
  return (
    <div className="p-4 md:p-6">
      <Suspense
        fallback={
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
        }
      >
        <RoadmapContent />
      </Suspense>
    </div>
  );
}
