"use client";

import { useSummary } from "@/lib/hooks/use-summary";
import { MetricsCard } from "@/components/domain/metrics-card";
import { StatusBadge } from "@/components/domain/status-badge";
import { PriorityIndicator } from "@/components/domain/priority-indicator";
import { EpicProgress } from "@/components/domain/epic-progress";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { postAction } from "@/lib/api";
import { toast } from "sonner";
import { RefreshCw } from "lucide-react";
import Link from "next/link";
import { PageTransition } from "@/components/domain/page-transition";

export default function DashboardPage() {
  const { data, isLoading, error, refetch } = useSummary();
  const queryClient = useQueryClient();

  const syncMutation = useMutation({
    mutationFn: () => postAction("/api/sync"),
    onSuccess: () => {
      queryClient.invalidateQueries();
      toast.success("Sync complete");
    },
    onError: (err: Error) => {
      toast.error("Sync failed", { description: err.message });
    },
  });

  if (isLoading) {
    return <div className="p-4 md:p-6"><DashboardSkeleton /></div>;
  }

  if (error) {
    return (
      <div className="p-4 md:p-6 flex flex-col items-center justify-center min-h-[400px] gap-4">
        <p className="text-muted-foreground">Failed to load dashboard data.</p>
        <p className="text-sm text-destructive">{error.message}</p>
        <p className="text-xs text-muted-foreground">
          Make sure <code className="font-mono">markplane serve --dev</code> is running.
        </p>
        <Button variant="outline" size="sm" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  if (!data) return null;

  const openCount =
    data.counts.in_progress + data.counts.planned + data.counts.backlog + data.counts.draft;

  return (
    <PageTransition>
    <div className="p-4 md:p-6 space-y-6 max-w-[1200px]">
      {/* Metric cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <MetricsCard label="Open Tasks" value={openCount} />
        <MetricsCard label="In Progress" value={data.counts.in_progress} />
        <MetricsCard
          label="Blocked"
          value={data.counts.blocked}
          accent={data.counts.blocked > 0 ? "warning" : "default"}
        />
        <MetricsCard label="Done" value={data.counts.done} />
      </div>

      {/* Active work */}
      {data.in_progress_tasks.length > 0 && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base font-semibold">Active Work</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {data.in_progress_tasks.map((task) => (
              <Link
                key={task.id}
                href={`/backlog`}
                className="flex items-center gap-3 rounded-md px-3 py-2 hover:bg-accent/50 transition-colors"
              >
                <PriorityIndicator priority={task.priority} />
                <span className="font-mono text-xs text-muted-foreground">
                  {task.id}
                </span>
                <span className="text-sm flex-1 truncate">{task.title}</span>
                <StatusBadge status={task.status} />
                {task.assignee && (
                  <span className="text-xs text-muted-foreground">
                    @{task.assignee}
                  </span>
                )}
              </Link>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Blocked items */}
      {data.blocked_tasks.length > 0 && (
        <Card className="border-l-2 border-l-status-blocked">
          <CardHeader className="pb-3">
            <CardTitle className="text-base font-semibold">Blocked Items</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {data.blocked_tasks.map((task) => (
              <div
                key={task.id}
                className="flex items-center gap-3 rounded-md px-3 py-2"
              >
                <span className="text-status-blocked">⚠</span>
                <span className="font-mono text-xs text-muted-foreground">
                  {task.id}
                </span>
                <span className="text-sm flex-1 truncate">{task.title}</span>
                {task.depends_on.length > 0 && (
                  <span className="text-xs text-muted-foreground">
                    blocked by {task.depends_on.join(", ")}
                  </span>
                )}
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Epic progress */}
      {data.active_epics.length > 0 && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base font-semibold">Epic Progress</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {data.active_epics.map((epic) => (
              <Link key={epic.id} href="/roadmap">
                <EpicProgress epic={epic} />
              </Link>
            ))}
          </CardContent>
        </Card>
      )}

      {/* AI Context panel */}
      <Card className="border-l-2 border-l-primary">
        <CardHeader className="pb-3 flex flex-row items-center justify-between">
          <div className="flex items-center gap-2">
            <CardTitle className="text-base font-semibold">AI Context</CardTitle>
            <ContextFreshness lastSynced={data.context_last_synced} />
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => syncMutation.mutate()}
            disabled={syncMutation.isPending}
            className="gap-1.5"
          >
            <RefreshCw
              className={`size-3.5 ${syncMutation.isPending ? "animate-spin" : ""}`}
            />
            Sync Now
          </Button>
        </CardHeader>
        <CardContent>
          {data.context_summary ? (
            <pre className="text-xs font-mono text-muted-foreground whitespace-pre-wrap leading-relaxed">
              {data.context_summary}
            </pre>
          ) : (
            <p className="text-sm text-muted-foreground">
              No context summary available. Click &quot;Sync Now&quot; to generate.
            </p>
          )}
        </CardContent>
      </Card>
    </div>
    </PageTransition>
  );
}

function ContextFreshness({ lastSynced }: { lastSynced: string | null }) {
  if (!lastSynced) return null;

  const syncTime = new Date(lastSynced).getTime();
  const ageMinutes = (Date.now() - syncTime) / 60_000;
  const isFresh = ageMinutes < 5;

  const label = isFresh
    ? "fresh"
    : ageMinutes < 60
      ? `${Math.round(ageMinutes)}m ago`
      : ageMinutes < 1440
        ? `${Math.round(ageMinutes / 60)}h ago`
        : `${Math.round(ageMinutes / 1440)}d ago`;

  return (
    <span
      className="text-[10px] font-medium px-1.5 py-0.5 rounded"
      style={{
        backgroundColor: isFresh
          ? "color-mix(in oklch, var(--status-done) 15%, transparent)"
          : "color-mix(in oklch, var(--priority-high) 15%, transparent)",
        color: isFresh ? "var(--status-done)" : "var(--priority-high)",
      }}
    >
      {label}
    </span>
  );
}

function DashboardSkeleton() {
  return (
    <div className="space-y-6 max-w-[1200px]">
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        {Array.from({ length: 4 }).map((_, i) => (
          <Skeleton key={i} className="h-20" />
        ))}
      </div>
      <Skeleton className="h-40" />
      <Skeleton className="h-32" />
    </div>
  );
}
