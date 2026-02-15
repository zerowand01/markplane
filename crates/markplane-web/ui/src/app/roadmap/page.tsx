"use client";

import { Suspense, useState, useMemo } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { useEpics } from "@/lib/hooks/use-epics";
import { useTasks } from "@/lib/hooks/use-tasks";
import { EpicDetailSheet } from "@/components/domain/epic-detail-sheet";
import { TaskDetailSheet } from "@/components/domain/task-detail-sheet";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { PriorityIndicator } from "@/components/domain/priority-indicator";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";
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
    <Card className="cursor-pointer hover:border-ring transition-colors" onClick={onClick}>
      <CardContent className="p-5 space-y-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs" style={{ color: "var(--entity-epic)" }}>
              {epic.id}
            </span>
            <h3 className="text-sm font-semibold">{epic.title}</h3>
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
      </CardContent>
    </Card>
  );
}

function NextCard({ epic, onClick }: { epic: Epic; onClick: () => void }) {
  const percent = Math.round(epic.progress * 100);

  return (
    <Card className="cursor-pointer hover:border-ring transition-colors" onClick={onClick}>
      <CardContent className="p-4 space-y-2">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs" style={{ color: "var(--entity-epic)" }}>
              {epic.id}
            </span>
            <h3 className="text-sm font-semibold">{epic.title}</h3>
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
      </CardContent>
    </Card>
  );
}

function LaterCard({ epic, onClick }: { epic: Epic; onClick: () => void }) {
  return (
    <Card className="cursor-pointer hover:border-ring transition-colors" onClick={onClick}>
      <CardContent className="p-3 flex items-center gap-3">
        <PriorityIndicator priority={epic.priority} />
        <span
          className="font-mono text-xs shrink-0"
          style={{ color: "var(--entity-epic)" }}
        >
          {epic.id}
        </span>
        <span className="text-sm font-medium truncate flex-1">{epic.title}</span>
        <span className="text-xs text-muted-foreground shrink-0">
          {epic.task_count} tasks
        </span>
      </CardContent>
    </Card>
  );
}

// ── Main Content ─────────────────────────────────────────────────────────

function RoadmapContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const { data: epicsData, isLoading: epicsLoading, error, refetch } = useEpics();
  const { data: tasksData } = useTasks();
  const [showDone, setShowDone] = useState(false);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);

  const selectedEpicId = searchParams.get("epic");

  const epics = epicsData ?? [];
  const tasks = tasksData ?? [];

  const { now, next, later, done } = useMemo(() => {
    const active = sortByPriority(epics.filter((e) => e.status === "active"));
    const planned = epics.filter((e) => e.status === "planned");
    const highPriority = sortByPriority(
      planned.filter((e) => e.priority === "critical" || e.priority === "high")
    );
    const lowPriority = sortByPriority(
      planned.filter(
        (e) =>
          e.priority === "medium" ||
          e.priority === "low" ||
          e.priority === "someday"
      )
    );
    const completed = epics.filter((e) => e.status === "done");
    return { now: active, next: highPriority, later: lowPriority, done: completed };
  }, [epics]);

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
        <div>
          <h1 className="text-2xl font-bold">Roadmap</h1>
          <p className="text-sm text-muted-foreground mt-1">
            Now / Next / Later &mdash; strategic epic timeline
          </p>
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
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {/* Now lane */}
            <div className="space-y-3">
              <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
                Now ({now.length})
              </h2>
              {now.length === 0 ? (
                <div className="rounded-lg border border-dashed p-8 text-center">
                  <p className="text-sm text-muted-foreground">No active epics</p>
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
                    No high-priority planned epics
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
              {later.length === 0 && done.length === 0 ? (
                <div className="rounded-lg border border-dashed p-8 text-center">
                  <p className="text-sm text-muted-foreground">
                    No lower-priority planned epics
                  </p>
                </div>
              ) : (
                <>
                  {later.length > 0 && (
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

                  {/* Done section (collapsed by default) */}
                  {done.length > 0 && (
                    <div className="pt-2">
                      <button
                        className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                        onClick={() => setShowDone(!showDone)}
                      >
                        {showDone ? "\u2212" : "+"} Completed ({done.length})
                      </button>
                      {showDone && (
                        <div className="space-y-3 mt-3">
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
            </div>
          </div>
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
      </div>
    </PageTransition>
  );
}

export default function RoadmapPage() {
  return (
    <Suspense
      fallback={
        <div className="space-y-4">
          <Skeleton className="h-8 w-32" />
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
  );
}
