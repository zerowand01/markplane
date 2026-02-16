"use client";

import { Suspense } from "react";
import { useSearchParams } from "next/navigation";
import dynamic from "next/dynamic";

import { useGraph } from "@/lib/hooks/use-graph";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";

const GraphView = dynamic(() => import("@/components/domain/graph-view"), {
  ssr: false,
  loading: () => <Skeleton className="h-screen w-full" />,
});

function GraphContent() {
  const searchParams = useSearchParams();
  const focusId = searchParams.get("focus") ?? undefined;

  const { data: graphData, isLoading, error, refetch } = useGraph(focusId);

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
          title="No dependencies mapped"
          description="Add depends_on or blocks fields to task frontmatter to see the dependency graph"
        />
      ) : (
        <GraphView graphData={graphData} focusId={focusId} />
      )}
    </div>
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
