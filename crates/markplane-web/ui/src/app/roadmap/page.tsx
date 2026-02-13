"use client";

import { Suspense } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { useEpics } from "@/lib/hooks/use-epics";
import { useTasks } from "@/lib/hooks/use-tasks";
import { EpicDetailSheet } from "@/components/domain/epic-detail-sheet";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { EPIC_STATUS_CONFIG } from "@/lib/constants";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";
import type { Epic, Task, TaskStatus } from "@/lib/types";

function EpicRoadmapCard({
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

  const statusGroups = ([
    {
      status: "in-progress" as TaskStatus,
      label: "In Progress",
      count: epicTasks.filter((t) => t.status === "in-progress").length,
    },
    {
      status: "planned" as TaskStatus,
      label: "Planned",
      count: epicTasks.filter((t) => t.status === "planned").length,
    },
    {
      status: "backlog" as TaskStatus,
      label: "Backlog",
      count: epicTasks.filter((t) => t.status === "backlog").length,
    },
    {
      status: "done" as TaskStatus,
      label: "Done",
      count: epicTasks.filter((t) => t.status === "done").length,
    },
  ] as const).filter((g) => g.count > 0);

  return (
    <Card
      className="cursor-pointer hover:border-ring transition-colors"
      onClick={onClick}
    >
      <CardContent className="p-5 space-y-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <span
              className="font-mono text-xs"
              style={{ color: "var(--entity-epic)" }}
            >
              {epic.id}
            </span>
            <h3 className="text-sm font-semibold">{epic.title}</h3>
          </div>
          <span
            className="inline-flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded shrink-0"
            style={{
              backgroundColor: `color-mix(in oklch, var(--status-${epic.status}) 15%, transparent)`,
              color: `var(--status-${epic.status})`,
            }}
          >
            {EPIC_STATUS_CONFIG[epic.status]?.icon}{" "}
            {EPIC_STATUS_CONFIG[epic.status]?.label}
          </span>
        </div>

        <div className="space-y-1.5">
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span>
              {epic.done_count}/{epic.task_count} tasks
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
                  style={{
                    backgroundColor: `var(--status-${group.status})`,
                  }}
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

function RoadmapContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const { data: epicsData, isLoading: epicsLoading, error, refetch } = useEpics();
  const { data: tasksData } = useTasks();

  const selectedEpicId = searchParams.get("epic");

  const epics = epicsData ?? [];
  const tasks = tasksData ?? [];

  const activeEpics = epics.filter((e) => e.status === "active");
  const plannedEpics = epics.filter((e) => e.status === "planned");
  const doneEpics = epics.filter((e) => e.status === "done");

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
          Epic progress and project timeline
        </p>
      </div>

      {epicsLoading ? (
        <div className="space-y-4">
          {Array.from({ length: 3 }).map((_, i) => (
            <Skeleton key={i} className="h-32 w-full" />
          ))}
        </div>
      ) : epics.length === 0 ? (
        <EmptyState
          title="No epics yet"
          description="Create strategic epics with markplane epic &quot;title&quot;"
        />
      ) : (
        <>
          {activeEpics.length > 0 && (
            <div className="space-y-3">
              <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
                Active Epics ({activeEpics.length})
              </h2>
              <div className="space-y-3">
                {activeEpics.map((epic) => (
                  <EpicRoadmapCard
                    key={epic.id}
                    epic={epic}
                    tasks={tasks}
                    onClick={() => {
                      const params = new URLSearchParams(searchParams);
                      params.set("epic", epic.id);
                      router.push(`/roadmap?${params.toString()}`);
                    }}
                  />
                ))}
              </div>
            </div>
          )}

          {plannedEpics.length > 0 && (
            <div className="space-y-3">
              <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
                Planned Epics ({plannedEpics.length})
              </h2>
              <div className="space-y-3">
                {plannedEpics.map((epic) => (
                  <EpicRoadmapCard
                    key={epic.id}
                    epic={epic}
                    tasks={tasks}
                    onClick={() => {
                      const params = new URLSearchParams(searchParams);
                      params.set("epic", epic.id);
                      router.push(`/roadmap?${params.toString()}`);
                    }}
                  />
                ))}
              </div>
            </div>
          )}

          {doneEpics.length > 0 && (
            <div className="space-y-3">
              <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
                Completed ({doneEpics.length})
              </h2>
              <div className="space-y-3">
                {doneEpics.map((epic) => (
                  <EpicRoadmapCard
                    key={epic.id}
                    epic={epic}
                    tasks={tasks}
                    onClick={() => {
                      const params = new URLSearchParams(searchParams);
                      params.set("epic", epic.id);
                      router.push(`/roadmap?${params.toString()}`);
                    }}
                  />
                ))}
              </div>
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
          {Array.from({ length: 3 }).map((_, i) => (
            <Skeleton key={i} className="h-32 w-full" />
          ))}
        </div>
      }
    >
      <RoadmapContent />
    </Suspense>
  );
}
