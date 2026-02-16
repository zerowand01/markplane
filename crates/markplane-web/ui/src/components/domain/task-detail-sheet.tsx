"use client";

import { useTask } from "@/lib/hooks/use-tasks";
import { useUpdateTask } from "@/lib/hooks/use-mutations";
import { StatusBadge } from "./status-badge";
import { PriorityIndicator } from "./priority-indicator";
import { MarkdownRenderer } from "./markdown-renderer";
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
import { STATUS_CONFIG } from "@/lib/constants";
import type { TaskStatus, Priority } from "@/lib/types";

const ALL_STATUSES: TaskStatus[] = [
  "draft",
  "backlog",
  "planned",
  "in-progress",
  "done",
  "cancelled",
];
const ALL_PRIORITIES: Priority[] = [
  "critical",
  "high",
  "medium",
  "low",
  "someday",
];

export function TaskDetailSheet({
  taskId,
  open,
  onOpenChange,
}: {
  taskId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const { data: task, isLoading } = useTask(taskId || "");
  const updateTask = useUpdateTask();

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <ResizableSheetContent>
        {isLoading || !task ? (
          <SheetHeader>
            <SheetTitle>
              <Skeleton className="h-6 w-48" />
            </SheetTitle>
            <div className="space-y-4 pt-4">
              <Skeleton className="h-4 w-32" />
              <Skeleton className="h-20 w-full" />
              <Skeleton className="h-40 w-full" />
            </div>
          </SheetHeader>
        ) : (
          <>
            <SheetHeader>
              <div className="flex items-center gap-2">
                <span
                  className="font-mono text-sm"
                  style={{ color: "var(--entity-task)" }}
                >
                  {task.id}
                </span>
                {task.type && (
                  <span className="text-xs px-2 py-0.5 rounded bg-secondary text-secondary-foreground uppercase">
                    {task.type}
                  </span>
                )}
              </div>
              <SheetTitle className="text-left text-xl">
                {task.title}
              </SheetTitle>
            </SheetHeader>

            <div className="space-y-4 px-4 pb-6">
              {/* Metadata grid */}
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <span className="text-sm text-muted-foreground block mb-1">
                    Status
                  </span>
                  <DropdownMenu>
                    <DropdownMenuTrigger className="cursor-pointer">
                      <StatusBadge status={task.status} />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      {ALL_STATUSES.map((s) => (
                        <DropdownMenuItem
                          key={s}
                          onClick={() =>
                            updateTask.mutate({ id: task.id, status: s })
                          }
                        >
                          <span
                            className="mr-2"
                            style={{ color: `var(--status-${s})` }}
                          >
                            {STATUS_CONFIG[s].icon}
                          </span>
                          {STATUS_CONFIG[s].label}
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>

                <div>
                  <span className="text-sm text-muted-foreground block mb-1">
                    Priority
                  </span>
                  <DropdownMenu>
                    <DropdownMenuTrigger className="cursor-pointer">
                      <PriorityIndicator
                        priority={task.priority}
                        showLabel
                      />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      {ALL_PRIORITIES.map((p) => (
                        <DropdownMenuItem
                          key={p}
                          onClick={() =>
                            updateTask.mutate({ id: task.id, priority: p })
                          }
                        >
                          <PriorityIndicator priority={p} showLabel />
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>

                {task.effort && (
                  <div>
                    <span className="text-sm text-muted-foreground block mb-1">
                      Effort
                    </span>
                    <span className="text-sm font-medium px-2 py-0.5 rounded bg-secondary text-secondary-foreground uppercase">
                      {task.effort}
                    </span>
                  </div>
                )}

                {task.epic && (
                  <div>
                    <span className="text-sm text-muted-foreground block mb-1">
                      Epic
                    </span>
                    <span
                      className="text-sm font-mono px-2 py-0.5 rounded"
                      style={{
                        backgroundColor:
                          "color-mix(in oklch, var(--entity-epic) 15%, transparent)",
                        color: "var(--entity-epic)",
                      }}
                    >
                      {task.epic}
                    </span>
                  </div>
                )}

                {task.plan && (
                  <div>
                    <span className="text-sm text-muted-foreground block mb-1">
                      Plan
                    </span>
                    <span
                      className="text-sm font-mono px-2 py-0.5 rounded"
                      style={{
                        backgroundColor:
                          "color-mix(in oklch, var(--entity-plan) 15%, transparent)",
                        color: "var(--entity-plan)",
                      }}
                    >
                      {task.plan}
                    </span>
                  </div>
                )}

                {task.assignee && (
                  <div>
                    <span className="text-sm text-muted-foreground block mb-1">
                      Assignee
                    </span>
                    <span className="text-sm">@{task.assignee}</span>
                  </div>
                )}
              </div>

              {/* Tags */}
              {task.tags.length > 0 && (
                <div>
                  <span className="text-sm text-muted-foreground block mb-1">
                    Tags
                  </span>
                  <div className="flex flex-wrap gap-1.5">
                    {task.tags.map((tag) => (
                      <span
                        key={tag}
                        className="text-sm text-muted-foreground bg-secondary px-2 py-0.5 rounded"
                      >
                        #{tag}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              {/* Dependencies */}
              {(task.depends_on.length > 0 || task.blocks.length > 0) && (
                <div className="space-y-2">
                  {task.depends_on.length > 0 && (
                    <div>
                      <span className="text-sm text-muted-foreground block mb-1">
                        Depends on
                      </span>
                      <div className="flex flex-wrap gap-1.5">
                        {task.depends_on.map((dep) => (
                          <span
                            key={dep}
                            className="text-sm font-mono text-muted-foreground bg-secondary px-2 py-0.5 rounded"
                          >
                            {dep}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}
                  {task.blocks.length > 0 && (
                    <div>
                      <span className="text-sm text-muted-foreground block mb-1">
                        Blocks
                      </span>
                      <div className="flex flex-wrap gap-1.5">
                        {task.blocks.map((b) => (
                          <span
                            key={b}
                            className="text-sm font-mono text-muted-foreground bg-secondary px-2 py-0.5 rounded"
                          >
                            {b}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}

              {/* Dates */}
              <div className="flex gap-4 text-sm text-muted-foreground">
                <span>Created {task.created}</span>
                <span>Updated {task.updated}</span>
              </div>

              <Separator />

              {/* Body markdown */}
              {task.body.trim() ? (
                <MarkdownRenderer content={task.body} />
              ) : (
                <p className="text-sm text-muted-foreground italic">
                  No description.
                </p>
              )}
            </div>
          </>
        )}
      </ResizableSheetContent>
    </Sheet>
  );
}
