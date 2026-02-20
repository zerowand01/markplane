"use client";

import { useState } from "react";
import { usePlan } from "@/lib/hooks/use-plans";
import { useUpdatePlan, useArchiveItem } from "@/lib/hooks/use-mutations";
import { MarkdownRenderer } from "./markdown-renderer";
import { MarkdownEditor } from "./markdown-editor";
import { WikiLinkChip } from "./wiki-link-chip";
import { InlineEdit } from "./inline-edit";
import { FieldRow, EmptyValue } from "./field-row";
import {
  Sheet,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { ResizableSheetContent } from "./resizable-sheet-content";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { Pencil, Archive } from "lucide-react";
import { Button } from "@/components/ui/button";
import { PLAN_STATUS_CONFIG } from "@/lib/constants";
import type { PlanStatus } from "@/lib/types";

const ALL_PLAN_STATUSES: PlanStatus[] = ["draft", "approved", "in-progress", "done"];

export function PlanDetailSheet({
  planId,
  open,
  onOpenChange,
}: {
  planId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [isEditingBody, setIsEditingBody] = useState(false);
  const { data: plan, isLoading } = usePlan(planId || "", {
    enabled: !isEditingBody,
  });
  const updatePlan = useUpdatePlan();
  const archiveItem = useArchiveItem();

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <ResizableSheetContent>
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
              <div className="flex items-center gap-2 pr-8">
                <span
                  className="font-mono text-sm"
                  style={{ color: "var(--entity-plan)" }}
                >
                  {plan.id}
                </span>
                <div className="flex-1" />
                {plan.status === "done" && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 px-2 gap-1 text-muted-foreground hover:text-foreground cursor-pointer"
                    onClick={() => {
                      archiveItem.mutate(plan.id);
                      onOpenChange(false);
                    }}
                    disabled={archiveItem.isPending}
                  >
                    <Archive className="size-3.5" />
                    <span className="text-xs">Archive</span>
                  </Button>
                )}
              </div>
              <SheetTitle className="text-left text-xl">
                <InlineEdit
                  value={plan.title}
                  onSave={(title) =>
                    updatePlan.mutate({ id: plan.id, title })
                  }
                />
              </SheetTitle>
            </SheetHeader>

            <div className="space-y-4 px-4 pb-6">
              <div className="space-y-0.5 text-sm">
                <FieldRow label="Status" editable>
                  <DropdownMenu>
                    <DropdownMenuTrigger className="cursor-pointer">
                      <span
                        className="inline-flex items-center gap-1.5"
                        style={{ color: `var(--status-${plan.status})` }}
                      >
                        {(() => {
                          const Icon = PLAN_STATUS_CONFIG[plan.status]?.icon;
                          return Icon ? <Icon className="size-3.5 text-current" /> : null;
                        })()}
                        <span className="text-sm">{PLAN_STATUS_CONFIG[plan.status]?.label}</span>
                      </span>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      {ALL_PLAN_STATUSES.map((s) => (
                        <DropdownMenuItem
                          key={s}
                          onClick={() =>
                            updatePlan.mutate({ id: plan.id, status: s })
                          }
                        >
                          {(() => {
                            const Icon = PLAN_STATUS_CONFIG[s]?.icon;
                            return Icon ? (
                              <Icon
                                className="mr-2 size-4 text-current"
                                style={{ color: `var(--status-${s})` }}
                              />
                            ) : null;
                          })()}
                          {PLAN_STATUS_CONFIG[s]?.label}
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuContent>
                  </DropdownMenu>
                </FieldRow>

                <FieldRow label="Epic">
                  {plan.epic ? (
                    <WikiLinkChip id={plan.epic} />
                  ) : (
                    <EmptyValue>No epic</EmptyValue>
                  )}
                </FieldRow>

                <FieldRow label="Implements">
                  {plan.implements.length > 0 ? (
                    <div className="flex flex-wrap gap-1.5">
                      {plan.implements.map((id) => (
                        <WikiLinkChip key={id} id={id} />
                      ))}
                    </div>
                  ) : (
                    <EmptyValue />
                  )}
                </FieldRow>

                <FieldRow label="Created">
                  <span className="text-muted-foreground">{plan.created}</span>
                </FieldRow>

                <FieldRow label="Updated">
                  <span className="text-muted-foreground">{plan.updated}</span>
                </FieldRow>
              </div>

              <Separator />

              {isEditingBody ? (
                <MarkdownEditor
                  content={plan.body}
                  onSave={(body) => {
                    updatePlan.mutate({ id: plan.id, body });
                    setIsEditingBody(false);
                  }}
                  onCancel={() => setIsEditingBody(false)}
                  isLoading={updatePlan.isPending}
                />
              ) : (
                <div className="group relative">
                  <button
                    type="button"
                    onClick={() => setIsEditingBody(true)}
                    className="sticky top-11 float-right ml-2 p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-accent transition-opacity cursor-pointer z-10"
                  >
                    <Pencil className="size-4 text-primary/50 group-hover:text-primary" />
                  </button>
                  {plan.body.trim() ? (
                    <MarkdownRenderer content={plan.body} />
                  ) : (
                    <button
                      type="button"
                      onClick={() => setIsEditingBody(true)}
                      className="text-sm text-muted-foreground italic hover:text-foreground cursor-pointer"
                    >
                      Click to add content...
                    </button>
                  )}
                </div>
              )}
            </div>
          </>
        )}
      </ResizableSheetContent>
    </Sheet>
  );
}
