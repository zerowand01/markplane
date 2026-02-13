"use client";

import { usePlan } from "@/lib/hooks/use-plans";
import { MarkdownRenderer } from "./markdown-renderer";
import { WikiLinkChip } from "./wiki-link-chip";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { PLAN_STATUS_CONFIG } from "@/lib/constants";

export function PlanDetailSheet({
  planId,
  open,
  onOpenChange,
}: {
  planId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const { data: plan, isLoading } = usePlan(planId || "");

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="sm:max-w-[540px] overflow-y-auto">
        {isLoading || !plan ? (
          <SheetHeader>
            <SheetTitle>
              <Skeleton className="h-6 w-48" />
            </SheetTitle>
            <div className="space-y-4 pt-4">
              <Skeleton className="h-4 w-32" />
              <Skeleton className="h-40 w-full" />
            </div>
          </SheetHeader>
        ) : (
          <>
            <SheetHeader>
              <div className="flex items-center gap-2">
                <span
                  className="font-mono text-xs"
                  style={{ color: "var(--entity-plan)" }}
                >
                  {plan.id}
                </span>
              </div>
              <SheetTitle className="text-left text-lg">
                {plan.title}
              </SheetTitle>
            </SheetHeader>

            <div className="space-y-4 pt-4">
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <span className="text-xs text-muted-foreground block mb-1">
                    Status
                  </span>
                  <span
                    className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded"
                    style={{
                      backgroundColor: `color-mix(in oklch, var(--status-${plan.status}) 15%, transparent)`,
                      color: `var(--status-${plan.status})`,
                    }}
                  >
                    <span>{PLAN_STATUS_CONFIG[plan.status]?.icon}</span>
                    <span>{PLAN_STATUS_CONFIG[plan.status]?.label}</span>
                  </span>
                </div>

                {plan.epic && (
                  <div>
                    <span className="text-xs text-muted-foreground block mb-1">
                      Epic
                    </span>
                    <WikiLinkChip id={plan.epic} />
                  </div>
                )}
              </div>

              {plan.implements.length > 0 && (
                <div>
                  <span className="text-xs text-muted-foreground block mb-1">
                    Implements
                  </span>
                  <div className="flex flex-wrap gap-1.5">
                    {plan.implements.map((id) => (
                      <WikiLinkChip key={id} id={id} />
                    ))}
                  </div>
                </div>
              )}

              <div className="flex gap-4 text-xs text-muted-foreground">
                <span>Created {plan.created}</span>
                <span>Updated {plan.updated}</span>
              </div>

              <Separator />

              {plan.body.trim() ? (
                <MarkdownRenderer content={plan.body} />
              ) : (
                <p className="text-sm text-muted-foreground italic">
                  No content.
                </p>
              )}
            </div>
          </>
        )}
      </SheetContent>
    </Sheet>
  );
}
