"use client";

import { useState, useCallback } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { useSummary } from "@/lib/hooks/use-summary";
import { StatusBadge } from "@/components/domain/status-badge";
import { PriorityIndicator } from "@/components/domain/priority-indicator";
import { EpicProgress } from "@/components/domain/epic-progress";
import { MarkdownRenderer } from "@/components/domain/markdown-renderer";
import { TaskDetailSheet } from "@/components/domain/task-detail-sheet";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { postAction } from "@/lib/api";
import { toast } from "sonner";
import { RefreshCw } from "lucide-react";
import Link from "next/link";
import { PageTransition } from "@/components/domain/page-transition";

function relativeTime(dateStr: string): string {
  const diff = Date.now() - new Date(dateStr).getTime();
  const minutes = Math.floor(diff / 60_000);
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

export default function DashboardPage() {
  const { data, isLoading, error, refetch } = useSummary();
  const queryClient = useQueryClient();
  const router = useRouter();
  const searchParams = useSearchParams();

  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(
    () => searchParams.get("task")
  );

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
    router.replace(qs ? `?${qs}` : "/dashboard/", { scroll: false });
  }, [router, searchParams]);

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

  return (
    <PageTransition>
    <div className="p-4 md:p-6 space-y-6 max-w-[1200px]">
      {/* Two-column layout: action stream (left) + strategic context (right) */}
      <div className="grid grid-cols-1 lg:grid-cols-[3fr_2fr] gap-6">
        {/* Left column — action stream, urgency-descending */}
        <div className="space-y-6 min-w-0">
          {/* Blocked items — top priority, only if any */}
          {data.blocked_tasks.length > 0 && (
            <section className="rounded-lg border border-l-2 border-l-status-blocked p-4">
              <h2 className="text-sm font-semibold mb-3">Blocked Items</h2>
              <div className="space-y-1">
                {data.blocked_tasks.map((task) => (
                  <button
                    key={task.id}
                    onClick={() => openTask(task.id)}
                    className="flex items-center gap-3 rounded-md px-3 py-2 w-full text-left hover:bg-accent/50 transition-colors"
                  >
                    <span className="text-status-blocked text-sm">!</span>
                    <span className="font-mono text-xs text-muted-foreground">
                      {task.id}
                    </span>
                    <span className="text-sm flex-1 truncate">{task.title}</span>
                    {task.depends_on.length > 0 && (
                      <span className="text-xs text-muted-foreground shrink-0">
                        blocked by {task.depends_on.join(", ")}
                      </span>
                    )}
                  </button>
                ))}
              </div>
            </section>
          )}

          {/* Active work — always visible */}
          <section className="rounded-lg border p-4">
            <h2 className="text-sm font-semibold mb-3">Active Work</h2>
            {data.in_progress_tasks.length > 0 ? (
              <div className="space-y-1">
                {data.in_progress_tasks.map((task) => (
                  <button
                    key={task.id}
                    onClick={() => openTask(task.id)}
                    className="flex items-center gap-3 rounded-md px-3 py-2 w-full text-left hover:bg-accent/50 transition-colors"
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
                  </button>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground px-3 py-2">No tasks in progress</p>
            )}
          </section>

          {/* Recent completions */}
          {data.recent_completions.length > 0 && (
            <section className="rounded-lg border p-4">
              <h2 className="text-sm font-semibold mb-3">Recent Completions</h2>
              <div className="space-y-1">
                {data.recent_completions.map((task) => (
                  <button
                    key={task.id}
                    onClick={() => openTask(task.id)}
                    className="flex items-center gap-3 rounded-md px-3 py-2 w-full text-left hover:bg-accent/50 transition-colors"
                  >
                    <span className="font-mono text-xs text-muted-foreground">
                      {task.id}
                    </span>
                    <span className="text-sm flex-1 truncate text-muted-foreground line-through decoration-muted-foreground/40">{task.title}</span>
                    <span className="text-xs text-muted-foreground shrink-0">
                      {relativeTime(task.updated)}
                    </span>
                  </button>
                ))}
              </div>
            </section>
          )}
        </div>

        {/* Right column — strategic context */}
        <div className="space-y-6">
          {/* Epic progress */}
          {data.active_epics.length > 0 && (
            <section className="rounded-lg border p-4">
              <h2 className="text-sm font-semibold mb-3">Epic Progress</h2>
              <div className="space-y-4">
                {data.active_epics.map((epic) => (
                  <Link key={epic.id} href="/roadmap">
                    <EpicProgress epic={epic} />
                  </Link>
                ))}
              </div>
            </section>
          )}

          {/* Up Next — always visible */}
          <section className="rounded-lg border p-4">
            <h2 className="text-sm font-semibold mb-3">Up Next</h2>
            {data.next_up_tasks.length > 0 ? (
              <div className="space-y-1">
                {data.next_up_tasks.map((task) => (
                  <button
                    key={task.id}
                    onClick={() => openTask(task.id)}
                    className="flex items-center gap-3 rounded-md px-3 py-2 w-full text-left hover:bg-accent/50 transition-colors"
                  >
                    <PriorityIndicator priority={task.priority} />
                    <span className="font-mono text-xs text-muted-foreground">
                      {task.id}
                    </span>
                    <span className="text-sm flex-1 truncate">{task.title}</span>
                  </button>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground px-3 py-2">No planned tasks</p>
            )}
          </section>
        </div>
      </div>

      {/* Full-width: AI Context panel */}
      <section className="rounded-lg border border-l-2 border-l-primary p-4">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <h2 className="text-sm font-semibold">AI Context</h2>
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
        </div>
        {data.context_summary ? (
          <div className="prose-sm">
            <MarkdownRenderer content={data.context_summary} />
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">
            No context summary available. Click &quot;Sync Now&quot; to generate.
          </p>
        )}
      </section>
    </div>

    <TaskDetailSheet
      taskId={selectedTaskId}
      open={!!selectedTaskId}
      onOpenChange={(open) => {
        if (!open) closeTask();
      }}
    />
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
      <div className="grid grid-cols-1 lg:grid-cols-[3fr_2fr] gap-6">
        <div className="space-y-6">
          <Skeleton className="h-40" />
          <Skeleton className="h-32" />
        </div>
        <div className="space-y-6">
          <Skeleton className="h-32" />
          <Skeleton className="h-40" />
        </div>
      </div>
      <Skeleton className="h-48" />
    </div>
  );
}
