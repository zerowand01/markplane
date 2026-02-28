"use client";

import { useState } from "react";
import { useTask, useTasks } from "@/lib/hooks/use-tasks";
import { useEpics } from "@/lib/hooks/use-epics";
import { usePlans } from "@/lib/hooks/use-plans";
import { useNotes } from "@/lib/hooks/use-notes";
import { useUpdateTask, useArchiveItem, useUnarchiveItem } from "@/lib/hooks/use-mutations";
import { CreateDialog } from "./create-dialog";
import { PriorityIndicator } from "./priority-indicator";
import { MarkdownRenderer } from "./markdown-renderer";
import { MarkdownEditor } from "./markdown-editor";
import { InlineEdit } from "./inline-edit";
import { TagEditor } from "./tag-editor";
import { EntityCombobox } from "./entity-combobox";
import { EntityRefEditor } from "./entity-ref-editor";
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
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { STATUS_CONFIG, buildStatusConfig, allStatuses, categoryOf } from "@/lib/constants";
import { useConfig } from "@/lib/hooks/use-config";
import { Circle, Pencil, Archive, ArchiveRestore } from "lucide-react";
import type { Priority, Effort } from "@/lib/types";
const ALL_PRIORITIES: Priority[] = [
  "critical",
  "high",
  "medium",
  "low",
  "someday",
];
const ALL_EFFORTS: Effort[] = ["xs", "small", "medium", "large", "xl"];

export function TaskDetailSheet({
  taskId,
  open,
  onOpenChange,
  archived,
}: {
  taskId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  archived?: boolean;
}) {
  const [isEditingBody, setIsEditingBody] = useState(false);
  const [createPlanOpen, setCreatePlanOpen] = useState(false);
  const { data: task, isLoading } = useTask(taskId || "", {
    enabled: !isEditingBody,
  });
  const updateTask = useUpdateTask();
  const archiveItem = useArchiveItem();
  const unarchiveItem = useUnarchiveItem();
  const { data: epics } = useEpics();
  const { data: allTasks } = useTasks();
  const { data: plans } = usePlans();
  const { data: notes } = useNotes();
  const { data: config } = useConfig();
  const workflow = config?.workflows.task;
  const statusConfig = workflow ? buildStatusConfig(workflow) : null;
  const statusList = workflow ? allStatuses(workflow) : [];

  const taskCategory = task && workflow ? categoryOf(workflow, task.status) : undefined;
  const isClosedStatus = taskCategory === "completed" || taskCategory === "cancelled";

  const epicOptions =
    epics?.map((e) => ({ id: e.id, title: e.title })) ?? [];
  const taskOptions =
    allTasks
      ?.filter((t) => t.id !== taskId)
      .map((t) => ({ id: t.id, title: t.title })) ?? [];
  const planOptions =
    plans?.map((p) => ({ id: p.id, title: p.title })) ?? [];
  const relatedOptions = [
    ...(allTasks?.filter((t) => t.id !== taskId).map((t) => ({ id: t.id, title: t.title })) ?? []),
    ...(epics?.map((e) => ({ id: e.id, title: e.title })) ?? []),
    ...(plans?.map((p) => ({ id: p.id, title: p.title })) ?? []),
    ...(notes?.map((n) => ({ id: n.id, title: n.title })) ?? []),
  ];

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
              <div className="flex items-center gap-2 pr-8">
                <span
                  className="font-mono text-sm"
                  style={{ color: "var(--entity-task)" }}
                >
                  {task.id}
                </span>
                <div className="flex-1" />
                {archived ? (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 px-2 gap-1 text-muted-foreground hover:text-foreground cursor-pointer"
                    onClick={() => {
                      unarchiveItem.mutate(task.id);
                      onOpenChange(false);
                    }}
                    disabled={unarchiveItem.isPending}
                  >
                    <ArchiveRestore className="size-3.5" />
                    <span className="text-xs">Restore</span>
                  </Button>
                ) : isClosedStatus && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 px-2 gap-1 text-muted-foreground hover:text-foreground cursor-pointer"
                    onClick={() => {
                      archiveItem.mutate(task.id);
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
                  value={task.title}
                  onSave={(title) =>
                    updateTask.mutate({ id: task.id, title })
                  }
                />
              </SheetTitle>
            </SheetHeader>

            <div className="space-y-4 px-4 pb-6">
              {/* Metadata */}
              <div className="space-y-0.5 text-sm">
                <FieldRow label="Status" editable>
                  <DropdownMenu>
                    <DropdownMenuTrigger className="cursor-pointer">
                      <span
                        className="inline-flex items-center gap-1.5"
                        style={{ color: `var(--status-${task.status})` }}
                      >
                        {(() => {
                          const cfg = statusConfig?.[task.status] ?? STATUS_CONFIG[task.status];
                          const Icon = cfg?.icon ?? Circle;
                          return <Icon className="size-3.5 text-current" />;
                        })()}
                        <span className="text-sm">
                          {statusConfig?.[task.status]?.label ?? STATUS_CONFIG[task.status]?.label ?? task.status}
                        </span>
                      </span>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      {statusList.map((s) => {
                        const cfg = statusConfig?.[s] ?? STATUS_CONFIG[s];
                        const Icon = cfg?.icon ?? Circle;
                        return (
                          <DropdownMenuItem
                            key={s}
                            onClick={() =>
                              updateTask.mutate({ id: task.id, status: s })
                            }
                          >
                            <Icon
                              className="mr-2 size-4 text-current"
                              style={{ color: `var(--status-${s})` }}
                            />
                            {cfg?.label ?? s}
                          </DropdownMenuItem>
                        );
                      })}
                    </DropdownMenuContent>
                  </DropdownMenu>
                </FieldRow>

                <FieldRow label="Priority" editable>
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
                </FieldRow>

                <FieldRow label="Type" editable>
                  <DropdownMenu>
                    <DropdownMenuTrigger className="cursor-pointer">
                      <span className="text-sm uppercase">{task.type}</span>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      {(config?.task_types ?? ["feature", "bug", "enhancement", "chore", "research", "spike"]).map((t) => (
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
                </FieldRow>

                <FieldRow label="Effort" editable>
                  <DropdownMenu>
                    <DropdownMenuTrigger className="cursor-pointer">
                      <span className="text-sm uppercase">{task.effort}</span>
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
                </FieldRow>

                <FieldRow label="Epic" editable>
                  <EntityCombobox
                    value={task.epic}
                    options={epicOptions}
                    onSelect={(id) =>
                      updateTask.mutate({ id: task.id, epic: id ?? "" })
                    }
                    placeholder="No epic"
                    emptyLabel="No epic"
                    linkValue
                  />
                </FieldRow>

                <FieldRow label="Plan" editable>
                  <EntityCombobox
                    value={task.plan}
                    options={planOptions}
                    onSelect={(id) =>
                      updateTask.mutate({ id: task.id, plan: id ?? "" })
                    }
                    placeholder="No plan"
                    emptyLabel="No plan"
                    linkValue
                    onCreateNew={() => setCreatePlanOpen(true)}
                    createNewLabel="Create new plan"
                  />
                </FieldRow>

                <FieldRow label="Depends on" editable>
                  <EntityRefEditor
                    ids={task.depends_on}
                    options={taskOptions}
                    onAdd={(id) =>
                      updateTask.mutate({
                        id: task.id,
                        depends_on: [...task.depends_on, id],
                      })
                    }
                    onRemove={(id) =>
                      updateTask.mutate({
                        id: task.id,
                        depends_on: task.depends_on.filter((d) => d !== id),
                      })
                    }
                  />
                </FieldRow>

                <FieldRow label="Blocks" editable>
                  <EntityRefEditor
                    ids={task.blocks}
                    options={taskOptions}
                    onAdd={(id) =>
                      updateTask.mutate({
                        id: task.id,
                        blocks: [...task.blocks, id],
                      })
                    }
                    onRemove={(id) =>
                      updateTask.mutate({
                        id: task.id,
                        blocks: task.blocks.filter((b) => b !== id),
                      })
                    }
                  />
                </FieldRow>

                <FieldRow label="Related" editable>
                  <EntityRefEditor
                    ids={task.related}
                    options={relatedOptions}
                    onAdd={(id) =>
                      updateTask.mutate({
                        id: task.id,
                        related: [...task.related, id],
                      })
                    }
                    onRemove={(id) =>
                      updateTask.mutate({
                        id: task.id,
                        related: task.related.filter((r) => r !== id),
                      })
                    }
                  />
                </FieldRow>

                <FieldRow label="Assignee" editable>
                  <AssigneeEditor
                    value={task.assignee}
                    onSave={(assignee) =>
                      updateTask.mutate({ id: task.id, assignee })
                    }
                  />
                </FieldRow>

                <FieldRow label="Tags" editable>
                  <TagEditor
                    tags={task.tags}
                    onSave={(tags) =>
                      updateTask.mutate({ id: task.id, tags })
                    }
                  />
                </FieldRow>

                <FieldRow label="Created">
                  <span className="text-muted-foreground">{task.created}</span>
                </FieldRow>

                <FieldRow label="Updated">
                  <span className="text-muted-foreground">{task.updated}</span>
                </FieldRow>
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

        {task && (
          <CreateDialog
            kind="plan"
            taskId={task.id}
            open={createPlanOpen}
            onOpenChange={setCreatePlanOpen}
          />
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
