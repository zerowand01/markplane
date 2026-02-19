"use client";

import { useState } from "react";
import { useTask } from "@/lib/hooks/use-tasks";
import { useEpics } from "@/lib/hooks/use-epics";
import { useUpdateTask } from "@/lib/hooks/use-mutations";
import { StatusBadge } from "./status-badge";
import { PriorityIndicator } from "./priority-indicator";
import { MarkdownRenderer } from "./markdown-renderer";
import { MarkdownEditor } from "./markdown-editor";
import { InlineEdit } from "./inline-edit";
import { TagEditor } from "./tag-editor";
import { EntityCombobox } from "./entity-combobox";
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
import { STATUS_CONFIG } from "@/lib/constants";
import { Pencil } from "lucide-react";
import type { TaskStatus, Priority, Effort, ItemType } from "@/lib/types";

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
const ALL_EFFORTS: Effort[] = ["xs", "small", "medium", "large", "xl"];
const ALL_TYPES: ItemType[] = [
  "feature",
  "bug",
  "enhancement",
  "chore",
  "research",
  "spike",
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
  const [isEditingBody, setIsEditingBody] = useState(false);
  const { data: task, isLoading } = useTask(taskId || "", {
    enabled: !isEditingBody,
  });
  const updateTask = useUpdateTask();
  const { data: epics } = useEpics();

  const epicOptions =
    epics?.map((e) => ({ id: e.id, title: e.title })) ?? [];

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
                <DropdownMenu>
                  <DropdownMenuTrigger className="cursor-pointer">
                    <span className="text-xs px-2 py-0.5 rounded bg-secondary text-secondary-foreground uppercase">
                      {task.type}
                    </span>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent>
                    {ALL_TYPES.map((t) => (
                      <DropdownMenuItem
                        key={t}
                        onClick={() =>
                          updateTask.mutate({ id: task.id, type: t })
                        }
                      >
                        <span className="uppercase text-xs">{t}</span>
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
              <SheetTitle className="text-left text-xl">
                <InlineEdit
                  value={task.title}
                  onSave={(title) =>
                    updateTask.mutate({ id: task.id, title })
                  }
                />
              </SheetTitle>
            </SheetHeader>

            <div className="space-y-4 px-4 pb-6">
              {/* Metadata */}
              <div className="space-y-2 text-sm">
                <div className="flex items-center gap-4">
                  <span className="text-muted-foreground w-20 shrink-0">Status</span>
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
                          {(() => {
                            const Icon = STATUS_CONFIG[s].icon;
                            return (
                              <Icon
                                className="mr-2 size-4 text-current"
                                style={{ color: `var(--status-${s})` }}
                              />
                            );
                          })()}
                          {STATUS_CONFIG[s].label}
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>

                <div className="flex items-center gap-4">
                  <span className="text-muted-foreground w-20 shrink-0">Priority</span>
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

                <div className="flex items-center gap-4">
                  <span className="text-muted-foreground w-20 shrink-0">Effort</span>
                  <DropdownMenu>
                    <DropdownMenuTrigger className="cursor-pointer">
                      <span className="text-sm font-medium px-2 py-0.5 rounded bg-secondary text-secondary-foreground uppercase">
                        {task.effort}
                      </span>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      {ALL_EFFORTS.map((e) => (
                        <DropdownMenuItem
                          key={e}
                          onClick={() =>
                            updateTask.mutate({ id: task.id, effort: e })
                          }
                        >
                          <span className="uppercase text-xs">{e}</span>
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>

                <div className="flex items-center gap-4">
                  <span className="text-muted-foreground w-20 shrink-0">Epic</span>
                  <EntityCombobox
                    value={task.epic}
                    options={epicOptions}
                    onSelect={(id) =>
                      updateTask.mutate({ id: task.id, epic: id ?? "" })
                    }
                    placeholder="No epic"
                    emptyLabel="No epic"
                    entityColor="var(--entity-epic)"
                  />
                </div>

                {task.plan && (
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground">Plan</span>
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

                <div className="flex items-center gap-4">
                  <span className="text-muted-foreground w-20 shrink-0">Assignee</span>
                  <AssigneeEditor
                    value={task.assignee}
                    onSave={(assignee) =>
                      updateTask.mutate({ id: task.id, assignee })
                    }
                  />
                </div>
              </div>

              {/* Tags */}
              <TagEditor
                tags={task.tags}
                onSave={(tags) =>
                  updateTask.mutate({ id: task.id, tags })
                }
              />

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
              {isEditingBody ? (
                <MarkdownEditor
                  content={task.body}
                  onSave={(body) => {
                    updateTask.mutate({ id: task.id, body });
                    setIsEditingBody(false);
                  }}
                  onCancel={() => setIsEditingBody(false)}
                  isLoading={updateTask.isPending}
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
                  {task.body.trim() ? (
                    <MarkdownRenderer content={task.body} />
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
            </div>
          </>
        )}
      </ResizableSheetContent>
    </Sheet>
  );
}

function AssigneeEditor({
  value,
  onSave,
}: {
  value: string | null;
  onSave: (value: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState(value ?? "");

  const handleSave = () => {
    onSave(draft.trim());
    setOpen(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleSave();
    } else if (e.key === "Escape") {
      setDraft(value ?? "");
      setOpen(false);
    }
  };

  return (
    <Popover
      open={open}
      onOpenChange={(o) => {
        setOpen(o);
        if (o) setDraft(value ?? "");
      }}
    >
      <PopoverTrigger className="cursor-pointer text-left">
        {value ? (
          <span className="text-sm">@{value}</span>
        ) : (
          <span className="text-sm text-muted-foreground italic">
            Unassigned
          </span>
        )}
      </PopoverTrigger>
      <PopoverContent className="w-52 p-3" align="start">
        <div className="space-y-2">
          <Input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Assignee name..."
            className="h-7 text-sm"
            autoFocus
          />
          <p className="text-xs text-muted-foreground">
            Press Enter to save, leave empty to unassign
          </p>
        </div>
      </PopoverContent>
    </Popover>
  );
}
