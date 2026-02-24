"use client";

import { useState } from "react";
import { useEpic } from "@/lib/hooks/use-epics";
import { useUpdateEpic, useArchiveItem } from "@/lib/hooks/use-mutations";
import { useTasks } from "@/lib/hooks/use-tasks";
import { EpicProgress } from "./epic-progress";
import { StatusBadge } from "./status-badge";
import { PriorityIndicator } from "./priority-indicator";
import { MarkdownRenderer } from "./markdown-renderer";
import { MarkdownEditor } from "./markdown-editor";
import { InlineEdit } from "./inline-edit";
import { TagEditor } from "./tag-editor";
import { FieldRow } from "./field-row";
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
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { Pencil, Archive } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { EPIC_STATUS_CONFIG } from "@/lib/constants";
import type { EpicStatus, Priority } from "@/lib/types";

const ALL_EPIC_STATUSES: EpicStatus[] = ["now", "next", "later", "done"];
const ALL_PRIORITIES: Priority[] = ["critical", "high", "medium", "low", "someday"];

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
  const [isEditingBody, setIsEditingBody] = useState(false);
  const { data: epic, isLoading } = useEpic(epicId || "", {
    enabled: !isEditingBody,
  });
  const updateEpic = useUpdateEpic();
  const archiveItem = useArchiveItem();
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
              <div className="flex items-center gap-2 pr-8">
                <span
                  className="font-mono text-sm"
                  style={{ color: "var(--entity-epic)" }}
                >
                  {epic.id}
                </span>
                <div className="flex-1" />
                {epic.status === "done" && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 px-2 gap-1 text-muted-foreground hover:text-foreground cursor-pointer"
                    onClick={() => {
                      archiveItem.mutate(epic.id);
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
                  value={epic.title}
                  onSave={(title) =>
                    updateEpic.mutate({ id: epic.id, title })
                  }
                />
              </SheetTitle>
            </SheetHeader>

            <div className="space-y-4 px-4 pb-6">
              {/* Progress */}
              <EpicProgress epic={epic} showHeader={false} />

              {/* Metadata */}
              <div className="space-y-0.5 text-sm">
                <FieldRow label="Status" editable>
                  <DropdownMenu>
                    <DropdownMenuTrigger className="cursor-pointer">
                      <span
                        className="inline-flex items-center gap-1.5"
                        style={{ color: `var(--status-${epic.status})` }}
                      >
                        {(() => {
                          const Icon = EPIC_STATUS_CONFIG[epic.status].icon;
                          return <Icon className="size-3.5 text-current" />;
                        })()}
                        <span className="text-sm">{EPIC_STATUS_CONFIG[epic.status].label}</span>
                      </span>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      {ALL_EPIC_STATUSES.map((s) => (
                        <DropdownMenuItem
                          key={s}
                          onClick={() =>
                            updateEpic.mutate({ id: epic.id, status: s })
                          }
                        >
                          {(() => {
                            const Icon = EPIC_STATUS_CONFIG[s].icon;
                            return (
                              <Icon
                                className="mr-2 size-4 text-current"
                                style={{ color: `var(--status-${s})` }}
                              />
                            );
                          })()}
                          {EPIC_STATUS_CONFIG[s].label}
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuContent>
                  </DropdownMenu>
                </FieldRow>

                <FieldRow label="Priority" editable>
                  <DropdownMenu>
                    <DropdownMenuTrigger className="cursor-pointer">
                      <PriorityIndicator
                        priority={epic.priority}
                        showLabel
                      />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      {ALL_PRIORITIES.map((p) => (
                        <DropdownMenuItem
                          key={p}
                          onClick={() =>
                            updateEpic.mutate({ id: epic.id, priority: p })
                          }
                        >
                          <PriorityIndicator priority={p} showLabel />
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuContent>
                  </DropdownMenu>
                </FieldRow>

                <FieldRow label="Started" editable>
                  <DateEditor
                    value={epic.started}
                    onSave={(started) =>
                      updateEpic.mutate({ id: epic.id, started })
                    }
                    placeholder="Not started"
                  />
                </FieldRow>

                <FieldRow label="Target" editable>
                  <DateEditor
                    value={epic.target}
                    onSave={(target) =>
                      updateEpic.mutate({ id: epic.id, target })
                    }
                    placeholder="No target"
                  />
                </FieldRow>

                <FieldRow label="Tags" editable>
                  <TagEditor
                    tags={epic.tags}
                    onSave={(tags) =>
                      updateEpic.mutate({ id: epic.id, tags })
                    }
                  />
                </FieldRow>
              </div>

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
              {isEditingBody ? (
                <MarkdownEditor
                  content={epic.body}
                  onSave={(body) => {
                    updateEpic.mutate({ id: epic.id, body });
                    setIsEditingBody(false);
                  }}
                  onCancel={() => setIsEditingBody(false)}
                  isLoading={updateEpic.isPending}
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
                  {epic.body.trim() ? (
                    <MarkdownRenderer content={epic.body} />
                  ) : (
                    <button
                      type="button"
                      onClick={() => setIsEditingBody(true)}
                      className="text-sm text-muted-foreground italic hover:text-foreground cursor-pointer"
                    >
                      Click to add description...
                    </button>
                  )}
                </div>
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

function DateEditor({
  value,
  onSave,
  placeholder = "Not set",
}: {
  value: string | null;
  onSave: (value: string) => void;
  placeholder?: string;
}) {
  return (
    <Popover>
      <PopoverTrigger className="cursor-pointer text-left">
        {value ? (
          <span className="text-sm">{value}</span>
        ) : (
          <span className="text-sm text-muted-foreground italic">
            {placeholder}
          </span>
        )}
      </PopoverTrigger>
      <PopoverContent className="w-auto p-3" align="start">
        <div className="space-y-2">
          <Input
            type="date"
            defaultValue={value ?? ""}
            onChange={(e) => onSave(e.target.value)}
            className="h-8 text-sm"
          />
          {value && (
            <button
              type="button"
              onClick={() => onSave("")}
              className="text-xs text-muted-foreground hover:text-foreground cursor-pointer"
            >
              Clear date
            </button>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}
