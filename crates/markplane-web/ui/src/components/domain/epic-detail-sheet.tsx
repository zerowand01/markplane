"use client";

import { useEpic } from "@/lib/hooks/use-epics";
import { useTasks } from "@/lib/hooks/use-tasks";
import { EpicStatusBadge } from "./status-badge";
import { EpicProgress } from "./epic-progress";
import { StatusBadge } from "./status-badge";
import { PriorityIndicator } from "./priority-indicator";
import { MarkdownRenderer } from "./markdown-renderer";
import {
  Sheet,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { ResizableSheetContent } from "./resizable-sheet-content";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export function EpicDetailSheet({
  epicId,
  open,
  onOpenChange,
  onTaskClick,
}: {
  epicId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onTaskClick?: (id: string) => void;
}) {
  const { data: epic, isLoading } = useEpic(epicId || "");
  const { data: allTasks } = useTasks();

  const linkedTasks = allTasks?.filter((t) => t.epic === epicId) || [];

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <ResizableSheetContent>
        {isLoading || !epic ? (
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
                  style={{ color: "var(--entity-epic)" }}
                >
                  {epic.id}
                </span>
                <EpicStatusBadge status={epic.status} />
              </div>
              <SheetTitle className="text-left text-xl">
                {epic.title}
              </SheetTitle>
            </SheetHeader>

            <div className="space-y-4 px-4 pb-6">
              {/* Progress */}
              <EpicProgress epic={epic} />

              {/* Metadata */}
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <span className="text-sm text-muted-foreground block mb-1">
                    Priority
                  </span>
                  <PriorityIndicator priority={epic.priority} showLabel />
                </div>
                {epic.started && (
                  <div>
                    <span className="text-sm text-muted-foreground block mb-1">
                      Started
                    </span>
                    <span className="text-sm">{epic.started}</span>
                  </div>
                )}
                {epic.target && (
                  <div>
                    <span className="text-sm text-muted-foreground block mb-1">
                      Target
                    </span>
                    <span className="text-sm">{epic.target}</span>
                  </div>
                )}
              </div>

              {/* Tags */}
              {epic.tags.length > 0 && (
                <div>
                  <span className="text-sm text-muted-foreground block mb-1">
                    Tags
                  </span>
                  <div className="flex flex-wrap gap-1.5">
                    {epic.tags.map((tag) => (
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

              {/* Status breakdown */}
              {Object.keys(epic.status_breakdown).length > 0 && (
                <div>
                  <span className="text-sm text-muted-foreground block mb-1">
                    Status Breakdown
                  </span>
                  <div className="flex gap-3 text-sm">
                    {Object.entries(epic.status_breakdown).map(
                      ([status, count]) => (
                        <span
                          key={status}
                          className="flex items-center gap-1"
                        >
                          <span
                            className="inline-block size-2 rounded-full"
                            style={{
                              backgroundColor: `var(--status-${status})`,
                            }}
                          />
                          {count} {status}
                        </span>
                      )
                    )}
                  </div>
                </div>
              )}

              <Separator />

              {/* Body markdown */}
              {epic.body.trim() ? (
                <MarkdownRenderer content={epic.body} />
              ) : (
                <p className="text-sm text-muted-foreground italic">
                  No description.
                </p>
              )}

              {/* Linked tasks table */}
              {linkedTasks.length > 0 && (
                <>
                  <Separator />
                  <div>
                    <h3 className="text-sm font-semibold mb-2">
                      Tasks ({linkedTasks.length})
                    </h3>
                    <div className="rounded-md border">
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead className="w-[80px]">ID</TableHead>
                            <TableHead>Title</TableHead>
                            <TableHead>Status</TableHead>
                            <TableHead>Priority</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {linkedTasks.map((task) => (
                            <TableRow
                              key={task.id}
                              className={
                                onTaskClick ? "cursor-pointer" : ""
                              }
                              onClick={() => onTaskClick?.(task.id)}
                            >
                              <TableCell className="font-mono text-xs text-muted-foreground">
                                {task.id}
                              </TableCell>
                              <TableCell className="text-sm font-medium truncate max-w-[200px]">
                                {task.title}
                              </TableCell>
                              <TableCell>
                                <StatusBadge status={task.status} />
                              </TableCell>
                              <TableCell>
                                <PriorityIndicator
                                  priority={task.priority}
                                />
                              </TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    </div>
                  </div>
                </>
              )}
            </div>
          </>
        )}
      </ResizableSheetContent>
    </Sheet>
  );
}
