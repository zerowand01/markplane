"use client";

import { Suspense, useCallback } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import dynamic from "next/dynamic";

import { useGraph } from "@/lib/hooks/use-graph";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";
import { TaskDetailSheet } from "@/components/domain/task-detail-sheet";
import { EpicDetailSheet } from "@/components/domain/epic-detail-sheet";
import { PlanDetailSheet } from "@/components/domain/plan-detail-sheet";
import { NoteDetailSheet } from "@/components/domain/note-detail-sheet";

const GraphView = dynamic(() => import("@/components/domain/graph-view"), {
  ssr: false,
  loading: () => <Skeleton className="h-screen w-full" />,
});

const PARAM_FOR_PREFIX: Record<string, string> = {
  TASK: "task",
  EPIC: "epic",
  PLAN: "plan",
  NOTE: "note",
};

function getSelectedFromParams(searchParams: URLSearchParams): { id: string; prefix: string; param: string } | null {
  for (const [prefix, param] of Object.entries(PARAM_FOR_PREFIX)) {
    const value = searchParams.get(param);
    if (value) return { id: value, prefix, param };
  }
  return null;
}

function GraphContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const focusId = searchParams.get("focus") ?? undefined;

  const selected = getSelectedFromParams(searchParams);

  const { data: graphData, isLoading, error, refetch } = useGraph(focusId);

  const openItem = useCallback(
    (id: string) => {
      const prefix = id.split("-")[0];
      const param = PARAM_FOR_PREFIX[prefix];
      if (!param) return;
      const params = new URLSearchParams(searchParams.toString());
      // Clear any other entity params first
      for (const p of Object.values(PARAM_FOR_PREFIX)) {
        params.delete(p);
      }
      params.set(param, id);
      router.replace(`?${params.toString()}`, { scroll: false });
    },
    [router, searchParams],
  );

  const closeItem = useCallback(() => {
    const params = new URLSearchParams(searchParams.toString());
    for (const p of Object.values(PARAM_FOR_PREFIX)) {
      params.delete(p);
    }
    const qs = params.toString();
    router.replace(qs ? `?${qs}` : "/graph/", { scroll: false });
  }, [router, searchParams]);

  const onSheetOpenChange = useCallback(
    (open: boolean) => {
      if (!open) closeItem();
    },
    [closeItem],
  );

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
        <p className="text-muted-foreground">Failed to load graph.</p>
        <p className="text-sm text-destructive">{error.message}</p>
        <Button variant="outline" size="sm" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  return (
    <PageTransition>
    <div className="space-y-4">
      {focusId && (
        <p className="text-sm text-muted-foreground">
          Focused on <code className="text-xs font-mono">{focusId}</code>
        </p>
      )}

      {isLoading || !graphData ? (
        <Skeleton className="h-screen w-full" />
      ) : graphData.nodes.length === 0 ? (
        <EmptyState
          title="No relationships found"
          description="Add depends_on, blocks, or epic fields to task frontmatter to see the graph"
        />
      ) : (
        <GraphView graphData={graphData} focusId={focusId} onNodeClick={openItem} />
      )}
    </div>

    <TaskDetailSheet
      taskId={selected?.prefix === "TASK" ? selected.id : null}
      open={selected?.prefix === "TASK"}
      onOpenChange={onSheetOpenChange}
    />
    <EpicDetailSheet
      epicId={selected?.prefix === "EPIC" ? selected.id : null}
      open={selected?.prefix === "EPIC"}
      onOpenChange={onSheetOpenChange}
      onTaskClick={openItem}
    />
    <PlanDetailSheet
      planId={selected?.prefix === "PLAN" ? selected.id : null}
      open={selected?.prefix === "PLAN"}
      onOpenChange={onSheetOpenChange}
    />
    <NoteDetailSheet
      noteId={selected?.prefix === "NOTE" ? selected.id : null}
      open={selected?.prefix === "NOTE"}
      onOpenChange={onSheetOpenChange}
    />
    </PageTransition>
  );
}

export default function GraphPage() {
  return (
    <Suspense fallback={<Skeleton className="h-[calc(100vh-120px)] w-full" />}>
      <GraphContent />
    </Suspense>
  );
}
