"use client";

import { useState } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { usePlans } from "@/lib/hooks/use-plans";
import { PlanDetailSheet } from "@/components/domain/plan-detail-sheet";
import { CreateDialog } from "@/components/domain/create-dialog";
import { Button } from "@/components/ui/button";

import { Skeleton } from "@/components/ui/skeleton";
import { PLAN_STATUS_CONFIG } from "@/lib/constants";
import { WikiLinkChip } from "@/components/domain/wiki-link-chip";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";
import { Plus } from "lucide-react";

export function PlansContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const { data, isLoading, error, refetch } = usePlans();
  const [createOpen, setCreateOpen] = useState(false);

  const selectedPlanId = searchParams.get("plan");

  const plans = data ?? [];

  const grouped = {
    "in-progress": plans.filter((p) => p.status === "in-progress"),
    approved: plans.filter((p) => p.status === "approved"),
    draft: plans.filter((p) => p.status === "draft"),
    done: plans.filter((p) => p.status === "done"),
  };

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
        <p className="text-muted-foreground">Failed to load plans.</p>
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
        <h1 className="text-lg font-semibold">Plans</h1>
        <Button
          variant="outline"
          className="text-xs gap-1 cursor-pointer"
          style={{
            color: "var(--entity-plan)",
            borderColor: "var(--entity-plan)",
            backgroundColor: "color-mix(in oklch, var(--entity-plan) 8%, transparent)",
          }}
          onClick={() => setCreateOpen(true)}
        >
          <Plus className="size-3.5" /> New Plan
        </Button>
      </div>

      {isLoading ? (
        <div className="space-y-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <Skeleton key={i} className="h-20 w-full" />
          ))}
        </div>
      ) : plans.length === 0 ? (
        <EmptyState
          title="No plans yet"
          description="Create an implementation plan with markplane plan TASK-xxx"
        />
      ) : (
        Object.entries(grouped).map(
          ([status, items]) =>
            items.length > 0 && (
              <div key={status} className="space-y-3">
                <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
                  {PLAN_STATUS_CONFIG[status]?.label ?? status} ({items.length})
                </h2>
                <div className="space-y-2">
                  {items.map((plan) => (
                    <div
                      key={plan.id}
                      className="rounded-lg border bg-card p-4 cursor-pointer hover:border-ring transition-colors"
                      onClick={() => {
                        const params = new URLSearchParams(searchParams);
                        params.set("plan", plan.id);
                        router.push(`/plans?${params.toString()}`);
                      }}
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2 mb-1">
                            <span
                              className="font-mono text-sm"
                              style={{ color: "var(--entity-plan)" }}
                            >
                              {plan.id}
                            </span>
                            <span
                              className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded"
                              style={{
                                backgroundColor: `color-mix(in oklch, var(--status-${plan.status}) 15%, transparent)`,
                                color: `var(--status-${plan.status})`,
                              }}
                            >
                              {(() => {
                                const Icon = PLAN_STATUS_CONFIG[plan.status]?.icon;
                                return Icon ? <Icon className="size-3 text-current" /> : null;
                              })()}{" "}
                              {PLAN_STATUS_CONFIG[plan.status]?.label}
                            </span>
                          </div>
                          <h3 className="text-base font-medium">
                            {plan.title}
                          </h3>
                          {plan.implements.length > 0 && (
                            <div className="flex items-center gap-1.5 mt-2">
                              <span className="text-sm text-muted-foreground">
                                Implements:
                              </span>
                              {plan.implements.map((id) => (
                                <WikiLinkChip key={id} id={id} />
                              ))}
                            </div>
                          )}
                        </div>
                        <span className="text-xs text-muted-foreground whitespace-nowrap">
                          {plan.updated}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )
        )
      )}

      <PlanDetailSheet
        planId={selectedPlanId}
        open={!!selectedPlanId}
        onOpenChange={(open) => {
          if (!open) {
            const params = new URLSearchParams(searchParams);
            params.delete("plan");
            router.push(`/plans?${params.toString()}`);
          }
        }}
      />

      <CreateDialog
        kind="plan"
        open={createOpen}
        onOpenChange={setCreateOpen}
        onCreated={(id) => {
          const params = new URLSearchParams(searchParams);
          params.set("plan", id);
          router.push(`/plans?${params.toString()}`);
        }}
      />
    </div>
    </PageTransition>
  );
}

export function PlansSkeleton() {
  return (
    <div className="space-y-3">
      {Array.from({ length: 3 }).map((_, i) => (
        <Skeleton key={i} className="h-20 w-full" />
      ))}
    </div>
  );
}
