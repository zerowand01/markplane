"use client";

import { Suspense, useState, useCallback } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { useEpics } from "@/lib/hooks/use-epics";
import { EpicStatusBadge } from "@/components/domain/status-badge";
import { EpicProgress } from "@/components/domain/epic-progress";
import { EpicDetailSheet } from "@/components/domain/epic-detail-sheet";
import { TaskDetailSheet } from "@/components/domain/task-detail-sheet";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";
import type { Epic, EpicStatus } from "@/lib/types";

const STATUS_ORDER: EpicStatus[] = ["active", "planned", "done"];

const SECTION_LABELS: Record<EpicStatus, string> = {
  active: "Active Epics",
  planned: "Planned Epics",
  done: "Completed",
};

export default function EpicsPage() {
  return (
    <Suspense fallback={<EpicsSkeleton />}>
      <EpicsContent />
    </Suspense>
  );
}

function EpicsContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const { data: epics, isLoading, error, refetch } = useEpics();

  const [selectedEpicId, setSelectedEpicId] = useState<string | null>(
    searchParams.get("epic")
  );
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);

  const openEpic = useCallback(
    (id: string) => {
      setSelectedEpicId(id);
      const params = new URLSearchParams(searchParams.toString());
      params.set("epic", id);
      router.replace(`?${params.toString()}`, { scroll: false });
    },
    [router, searchParams]
  );

  const closeEpic = useCallback(() => {
    setSelectedEpicId(null);
    const params = new URLSearchParams(searchParams.toString());
    params.delete("epic");
    const qs = params.toString();
    router.replace(qs ? `?${qs}` : "/epics/", { scroll: false });
  }, [router, searchParams]);

  if (isLoading) {
    return <EpicsSkeleton />;
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
        <p className="text-muted-foreground">Failed to load epics.</p>
        <p className="text-sm text-destructive">{error.message}</p>
        <Button variant="outline" size="sm" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  if (!epics) return null;

  // Group by status
  const epicsByStatus = new Map<EpicStatus, Epic[]>();
  for (const status of STATUS_ORDER) {
    epicsByStatus.set(status, []);
  }
  for (const epic of epics) {
    const existing = epicsByStatus.get(epic.status);
    if (existing) {
      existing.push(epic);
    }
  }

  return (
    <PageTransition>
    <div className="space-y-6 max-w-[1200px]">
      <h1 className="text-2xl font-bold tracking-tight">Epics</h1>

      {epics.length === 0 && (
        <EmptyState
          title="No epics yet"
          description="Create strategic epics with markplane epic &quot;title&quot;"
        />
      )}

      {STATUS_ORDER.map((status) => {
        const statusEpics = epicsByStatus.get(status) || [];
        if (statusEpics.length === 0) return null;

        return (
          <div key={status} className="space-y-3">
            <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wide">
              {SECTION_LABELS[status]}
            </h2>
            <div className="space-y-3">
              {statusEpics.map((epic) => (
                <EpicCard
                  key={epic.id}
                  epic={epic}
                  onClick={() => openEpic(epic.id)}
                />
              ))}
            </div>
          </div>
        );
      })}

      {/* Epic detail sheet */}
      <EpicDetailSheet
        epicId={selectedEpicId}
        open={!!selectedEpicId}
        onOpenChange={(open) => {
          if (!open) closeEpic();
        }}
        onTaskClick={(id) => {
          setSelectedTaskId(id);
        }}
      />

      {/* Task detail from epic's task list */}
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

function EpicCard({ epic, onClick }: { epic: Epic; onClick: () => void }) {
  const statusBreakdown = Object.entries(epic.status_breakdown || {});

  return (
    <Card
      className="hover:border-muted-foreground/30 transition-colors cursor-pointer"
      onClick={onClick}
    >
      <CardContent className="p-4 space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span
              className="font-mono text-xs"
              style={{ color: "var(--entity-epic)" }}
            >
              {epic.id}
            </span>
            <span className="font-semibold">{epic.title}</span>
          </div>
          <EpicStatusBadge status={epic.status} />
        </div>

        <EpicProgress epic={epic} />

        {statusBreakdown.length > 0 && (
          <div className="flex gap-3 text-xs text-muted-foreground">
            {statusBreakdown.map(([status, count]) => (
              <span key={status} className="flex items-center gap-1">
                <span
                  className="inline-block size-2 rounded-full"
                  style={{
                    backgroundColor: `var(--status-${status})`,
                  }}
                />
                {count} {status}
              </span>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function EpicsSkeleton() {
  return (
    <div className="space-y-6 max-w-[1200px]">
      <Skeleton className="h-8 w-24" />
      {Array.from({ length: 3 }).map((_, i) => (
        <Skeleton key={i} className="h-32" />
      ))}
    </div>
  );
}
