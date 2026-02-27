"use client";

import { useState, useEffect, useRef } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useCreateTask, useCreateEpic, useCreatePlan, useCreateNote } from "@/lib/hooks/use-mutations";
import { useEpics } from "@/lib/hooks/use-epics";
import { useTasks } from "@/lib/hooks/use-tasks";
import { useConfig } from "@/lib/hooks/use-config";
import type { Priority, Effort } from "@/lib/types";

interface CreateDialogProps {
  kind: "task" | "epic" | "note" | "plan";
  open: boolean;
  onOpenChange: (open: boolean) => void;
  taskId?: string;
  onCreated?: (id: string) => void;
}

export function CreateDialog({
  kind,
  open,
  onOpenChange,
  taskId,
  onCreated,
}: CreateDialogProps) {
  const { data: config } = useConfig();
  const [title, setTitle] = useState("");
  const [itemType, setItemType] = useState("feature");
  const [priority, setPriority] = useState<Priority>("medium");
  const [effort, setEffort] = useState<Effort>("medium");
  const [noteType, setNoteType] = useState("research");
  const [epic, setEpic] = useState<string>("none");
  const [planTaskId, setPlanTaskId] = useState<string>("none");
  const inputRef = useRef<HTMLInputElement>(null);

  const createTask = useCreateTask();
  const createEpic = useCreateEpic();
  const createPlan = useCreatePlan();
  const createNote = useCreateNote();
  const { data: epics } = useEpics();
  const { data: allTasks } = useTasks();

  const isPending =
    createTask.isPending || createEpic.isPending || createPlan.isPending || createNote.isPending;

  // Reset state when dialog opens/closes
  useEffect(() => {
    if (open) {
      setTitle("");
      setItemType(config?.task_types[0] ?? "feature");
      setPriority("medium");
      setEffort("medium");
      setNoteType(config?.note_types[0] ?? "research");
      setEpic("none");
      setPlanTaskId("none");
      // Auto-focus title input
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [open, config]);

  const handleSubmit = () => {
    if (!title.trim() || isPending) return;

    const onSuccess = (id: string) => {
      onOpenChange(false);
      onCreated?.(id);
    };

    switch (kind) {
      case "task":
        createTask.mutate(
          {
            title: title.trim(),
            type: itemType,
            priority,
            effort,
            epic: epic !== "none" ? epic : undefined,
          },
          { onSuccess: (data) => onSuccess(data.id) }
        );
        break;
      case "epic":
        createEpic.mutate(
          { title: title.trim(), priority },
          { onSuccess: (data) => onSuccess(data.id) }
        );
        break;
      case "plan": {
        const resolvedTaskId = taskId ?? (planTaskId !== "none" ? planTaskId : undefined);
        createPlan.mutate(
          { title: title.trim(), task_id: resolvedTaskId },
          { onSuccess: (data) => onSuccess(data.id) }
        );
        break;
      }
      case "note":
        createNote.mutate(
          { title: title.trim(), type: noteType },
          { onSuccess: (data) => onSuccess(data.id) }
        );
        break;
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && title.trim() && !isPending) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const kindLabel = kind.charAt(0).toUpperCase() + kind.slice(1);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>New {kindLabel}</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2" onKeyDown={handleKeyDown}>
          <div className="space-y-2">
            <label className="text-sm font-medium">Title</label>
            <Input
              ref={inputRef}
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder={`${kindLabel} title...`}
            />
          </div>

          {kind === "task" && (
            <>
              <div className="grid grid-cols-3 gap-3">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Type</label>
                  <Select value={itemType} onValueChange={setItemType}>
                    <SelectTrigger className="w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {(config?.task_types ?? ["feature", "bug", "enhancement", "chore", "research", "spike"]).map((t) => (
                        <SelectItem key={t} value={t}>
                          {t.charAt(0).toUpperCase() + t.slice(1)}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">Priority</label>
                  <Select value={priority} onValueChange={(v) => setPriority(v as Priority)}>
                    <SelectTrigger className="w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="critical">Critical</SelectItem>
                      <SelectItem value="high">High</SelectItem>
                      <SelectItem value="medium">Medium</SelectItem>
                      <SelectItem value="low">Low</SelectItem>
                      <SelectItem value="someday">Someday</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">Effort</label>
                  <Select value={effort} onValueChange={(v) => setEffort(v as Effort)}>
                    <SelectTrigger className="w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="xs">XS</SelectItem>
                      <SelectItem value="small">Small</SelectItem>
                      <SelectItem value="medium">Medium</SelectItem>
                      <SelectItem value="large">Large</SelectItem>
                      <SelectItem value="xl">XL</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
              {epics && epics.length > 0 && (
                <div className="space-y-2">
                  <label className="text-sm font-medium">Epic</label>
                  <Select value={epic} onValueChange={setEpic}>
                    <SelectTrigger className="w-full">
                      <SelectValue placeholder="No epic" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="none">No epic</SelectItem>
                      {epics.map((e) => (
                        <SelectItem key={e.id} value={e.id}>
                          {e.id}: {e.title}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}
            </>
          )}

          {kind === "epic" && (
            <div className="space-y-2">
              <label className="text-sm font-medium">Priority</label>
              <Select value={priority} onValueChange={(v) => setPriority(v as Priority)}>
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="critical">Critical</SelectItem>
                  <SelectItem value="high">High</SelectItem>
                  <SelectItem value="medium">Medium</SelectItem>
                  <SelectItem value="low">Low</SelectItem>
                  <SelectItem value="someday">Someday</SelectItem>
                </SelectContent>
              </Select>
            </div>
          )}

          {kind === "note" && (
            <div className="space-y-2">
              <label className="text-sm font-medium">Type</label>
              <Select value={noteType} onValueChange={setNoteType}>
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {(config?.note_types ?? ["research", "analysis", "idea", "decision", "meeting"]).map((t) => (
                    <SelectItem key={t} value={t}>
                      {t.charAt(0).toUpperCase() + t.slice(1)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

          {kind === "plan" && !taskId && allTasks && allTasks.length > 0 && (
            <div className="space-y-2">
              <label className="text-sm font-medium">Implements task</label>
              <Select value={planTaskId} onValueChange={setPlanTaskId}>
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="No task" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">No task</SelectItem>
                  {allTasks
                    .filter((t) => !t.plan)
                    .map((t) => (
                      <SelectItem key={t.id} value={t.id}>
                        {t.id}: {t.title}
                      </SelectItem>
                    ))}
                </SelectContent>
              </Select>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!title.trim() || isPending}>
            {isPending ? "Creating..." : `Create ${kindLabel}`}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
