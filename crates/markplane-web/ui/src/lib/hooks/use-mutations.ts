"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { patchJson, postJson, postAction } from "@/lib/api";
import { toast } from "sonner";
import type { Task, Epic, Plan, Note, ProjectConfig, CreateTaskRequest, CreateEpicRequest, CreatePlanRequest, CreateNoteRequest, UpdateTaskRequest, UpdateEpicRequest, UpdatePlanRequest, UpdateNoteRequest } from "@/lib/types";

// Ordered by display priority — first match wins when multiple fields change
const TASK_FIELD_LABELS: [string, string][] = [
  ["status", "Status"],
  ["priority", "Priority"],
  ["effort", "Effort"],
  ["type", "Type"],
  ["title", "Title"],
  ["tags", "Tags"],
  ["epic", "Epic"],
  ["assignee", "Assignee"],
  ["position", "Position"],
  ["body", "Content"],
];
const EPIC_FIELD_LABELS: [string, string][] = [
  ["status", "Status"],
  ["priority", "Priority"],
  ["title", "Title"],
  ["tags", "Tags"],
  ["started", "Start date"],
  ["target", "Target date"],
  ["depends_on", "Dependencies"],
  ["body", "Content"],
];
const PLAN_FIELD_LABELS: [string, string][] = [
  ["status", "Status"],
  ["title", "Title"],
  ["epic", "Epic"],
  ["body", "Content"],
];
const NOTE_FIELD_LABELS: [string, string][] = [
  ["status", "Status"],
  ["type", "Type"],
  ["title", "Title"],
  ["tags", "Tags"],
  ["related", "Related"],
  ["body", "Content"],
];

// Fields that support toast-based undo (discrete property changes, not body/title)
const TASK_UNDO_FIELDS = ["status", "priority", "effort", "type", "assignee", "tags", "epic", "position"];
const EPIC_UNDO_FIELDS = ["status", "priority", "tags", "started", "target"];
const PLAN_UNDO_FIELDS = ["status", "epic"];
const NOTE_UNDO_FIELDS = ["status", "type", "tags"];

// Nullable fields where the server requires "" (not null) to clear the value.
// Sending JSON null maps to Patch::Unchanged (no-op); "" maps to Patch::Clear.
const NULLABLE_FIELDS = new Set(["assignee", "epic", "position", "started", "target"]);

function detectField(
  updates: Record<string, unknown>,
  labels: [string, string][],
  fallback: string,
): string {
  return labels.find(([k]) => k in updates)?.[1] ?? fallback;
}

function buildUndoPayload(
  previous: Record<string, unknown> | undefined,
  changes: Record<string, unknown>,
  undoableFields: string[],
): Record<string, unknown> | null {
  if (!previous) return null;
  const payload: Record<string, unknown> = {};
  let count = 0;
  for (const field of undoableFields) {
    if (field in changes && changes[field] !== undefined) {
      const prev = previous[field];
      payload[field] = (prev == null && NULLABLE_FIELDS.has(field)) ? "" : prev;
      count++;
    }
  }
  return count > 0 ? payload : null;
}

export function useUpdateTask() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, ...body }: UpdateTaskRequest & { id: string }) =>
      patchJson<Task>(`/api/tasks/${id}`, body),
    onMutate: async ({ id, ...updates }) => {
      // Cancel outgoing refetches so they don't overwrite our optimistic update
      await queryClient.cancelQueries({ queryKey: ["tasks"] });

      // Snapshot previous tasks cache (all filter variants)
      const previousQueries = queryClient.getQueriesData<{
        data: Task[];
        total: number;
      }>({ queryKey: ["tasks", "list"] });

      // Optimistically update every cached task list that contains this task
      queryClient.setQueriesData<{ data: Task[]; total: number }>(
        { queryKey: ["tasks", "list"] },
        (old) => {
          if (!old) return old;
          return {
            ...old,
            data: old.data.map((t) =>
              t.id === id ? { ...t, ...updates } : t
            ),
          };
        }
      );

      // Also update the single-task cache if present
      const previousTask = queryClient.getQueryData<Task>(["tasks", "detail", id]);
      if (previousTask) {
        queryClient.setQueryData<Task>(["tasks", "detail", id], {
          ...previousTask,
          ...updates,
        });
      }

      return { previousQueries, previousTask, id };
    },
    onSuccess: (_data, variables, context) => {
      const { id, ...updates } = variables;
      const field = detectField(updates as Record<string, unknown>, TASK_FIELD_LABELS, "Task");

      const prev = context?.previousTask ??
        context?.previousQueries
          ?.flatMap(([, d]) => d?.data ?? [])
          .find((t) => t.id === id);
      const undoPayload = buildUndoPayload(
        prev as Record<string, unknown> | undefined,
        updates as Record<string, unknown>,
        TASK_UNDO_FIELDS,
      );

      if (undoPayload) {
        toast.success(`${field} updated`, {
          action: { label: "Undo", onClick: () => {
            patchJson<Task>(`/api/tasks/${id}`, undoPayload)
              .then(() => {
                toast.success("Undone");
                queryClient.invalidateQueries({ queryKey: ["tasks"] });
                queryClient.invalidateQueries({ queryKey: ["summary"] });
              })
              .catch((err: Error) => {
                toast.error("Undo failed", { description: err.message });
              });
          }},
          duration: 5000,
        });
      } else {
        toast.success(`${field} updated`);
      }
    },
    onError: (err, _vars, context) => {
      toast.error("Failed to update task", { description: err.message });
      // Rollback all task list caches
      if (context?.previousQueries) {
        for (const [key, data] of context.previousQueries) {
          queryClient.setQueryData(key, data);
        }
      }
      // Rollback single-task cache
      if (context?.previousTask) {
        queryClient.setQueryData(["tasks", "detail", context.id], context.previousTask);
      }
    },
    onSettled: () => {
      // Refetch to ensure server state is authoritative
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
      queryClient.invalidateQueries({ queryKey: ["summary"] });
    },
  });
}

export function useCreateTask() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreateTaskRequest) =>
      postJson<Task>("/api/tasks", body),
    onSuccess: (data) => {
      toast.success("Task created", { description: `${data.id}: ${data.title}` });
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
      queryClient.invalidateQueries({ queryKey: ["summary"] });
    },
    onError: (err) => {
      toast.error("Failed to create task", { description: err.message });
    },
  });
}

export function useCreateEpic() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreateEpicRequest) =>
      postJson<Epic>("/api/epics", body),
    onSuccess: (data) => {
      toast.success("Epic created", { description: `${data.id}: ${data.title}` });
      queryClient.invalidateQueries({ queryKey: ["epics"] });
      queryClient.invalidateQueries({ queryKey: ["summary"] });
    },
    onError: (err) => {
      toast.error("Failed to create epic", { description: err.message });
    },
  });
}

export function useCreatePlan() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreatePlanRequest) =>
      postJson<Plan>("/api/plans", body),
    onSuccess: (data) => {
      toast.success("Plan created", { description: `${data.id}: ${data.title}` });
      queryClient.invalidateQueries({ queryKey: ["plans"] });
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
      queryClient.invalidateQueries({ queryKey: ["summary"] });
    },
    onError: (err) => {
      toast.error("Failed to create plan", { description: err.message });
    },
  });
}

export function useCreateNote() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreateNoteRequest) =>
      postJson<Note>("/api/notes", body),
    onSuccess: (data) => {
      toast.success("Note created", { description: `${data.id}: ${data.title}` });
      queryClient.invalidateQueries({ queryKey: ["notes"] });
    },
    onError: (err) => {
      toast.error("Failed to create note", { description: err.message });
    },
  });
}

export function useUpdateEpic() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, ...body }: UpdateEpicRequest & { id: string }) =>
      patchJson<Epic>(`/api/epics/${id}`, body),
    onMutate: async ({ id, ...updates }) => {
      await queryClient.cancelQueries({ queryKey: ["epics"] });

      // Update list caches
      const previousQueries = queryClient.getQueriesData<{
        data: Epic[];
        total: number;
      }>({ queryKey: ["epics", "list"] });
      queryClient.setQueriesData<{ data: Epic[]; total: number }>(
        { queryKey: ["epics", "list"] },
        (old) => {
          if (!old) return old;
          return {
            ...old,
            data: old.data.map((e) =>
              e.id === id ? { ...e, ...updates } : e
            ),
          };
        }
      );

      // Update single-item cache
      const previousEpic = queryClient.getQueryData<Epic>(["epics", "detail", id]);
      if (previousEpic) {
        queryClient.setQueryData<Epic>(["epics", "detail", id], { ...previousEpic, ...updates });
      }
      return { previousQueries, previousEpic, id };
    },
    onSuccess: (_data, variables, context) => {
      const { id, ...updates } = variables;
      const field = detectField(updates as Record<string, unknown>, EPIC_FIELD_LABELS, "Epic");

      const prev = context?.previousEpic ??
        context?.previousQueries
          ?.flatMap(([, d]) => d?.data ?? [])
          .find((e) => e.id === id);
      const undoPayload = buildUndoPayload(
        prev as Record<string, unknown> | undefined,
        updates as Record<string, unknown>,
        EPIC_UNDO_FIELDS,
      );

      if (undoPayload) {
        toast.success(`${field} updated`, {
          action: { label: "Undo", onClick: () => {
            patchJson<Epic>(`/api/epics/${id}`, undoPayload)
              .then(() => {
                toast.success("Undone");
                queryClient.invalidateQueries({ queryKey: ["epics"] });
                queryClient.invalidateQueries({ queryKey: ["summary"] });
              })
              .catch((err: Error) => {
                toast.error("Undo failed", { description: err.message });
              });
          }},
          duration: 5000,
        });
      } else {
        toast.success(`${field} updated`);
      }
    },
    onError: (err, _vars, context) => {
      toast.error("Failed to update epic", { description: err.message });
      if (context?.previousQueries) {
        for (const [key, data] of context.previousQueries) {
          queryClient.setQueryData(key, data);
        }
      }
      if (context?.previousEpic) {
        queryClient.setQueryData(["epics", "detail", context.id], context.previousEpic);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["epics"] });
      queryClient.invalidateQueries({ queryKey: ["summary"] });
    },
  });
}

export function useUpdatePlan() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, ...body }: UpdatePlanRequest & { id: string }) =>
      patchJson<Plan>(`/api/plans/${id}`, body),
    onMutate: async ({ id, ...updates }) => {
      await queryClient.cancelQueries({ queryKey: ["plans"] });
      const previousPlan = queryClient.getQueryData<Plan>(["plans", "detail", id]);
      if (previousPlan) {
        queryClient.setQueryData<Plan>(["plans", "detail", id], { ...previousPlan, ...updates });
      }
      return { previousPlan, id };
    },
    onSuccess: (_data, variables, context) => {
      const { id, ...updates } = variables;
      const field = detectField(updates as Record<string, unknown>, PLAN_FIELD_LABELS, "Plan");

      const undoPayload = buildUndoPayload(
        context?.previousPlan as Record<string, unknown> | undefined,
        updates as Record<string, unknown>,
        PLAN_UNDO_FIELDS,
      );

      if (undoPayload) {
        toast.success(`${field} updated`, {
          action: { label: "Undo", onClick: () => {
            patchJson<Plan>(`/api/plans/${id}`, undoPayload)
              .then(() => {
                toast.success("Undone");
                queryClient.invalidateQueries({ queryKey: ["plans"] });
              })
              .catch((err: Error) => {
                toast.error("Undo failed", { description: err.message });
              });
          }},
          duration: 5000,
        });
      } else {
        toast.success(`${field} updated`);
      }
    },
    onError: (err, _vars, context) => {
      toast.error("Failed to update plan", { description: err.message });
      if (context?.previousPlan) {
        queryClient.setQueryData(["plans", "detail", context.id], context.previousPlan);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["plans"] });
    },
  });
}

export function useUpdateNote() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, ...body }: UpdateNoteRequest & { id: string }) =>
      patchJson<Note>(`/api/notes/${id}`, body),
    onMutate: async ({ id, ...updates }) => {
      await queryClient.cancelQueries({ queryKey: ["notes"] });
      const previousNote = queryClient.getQueryData<Note>(["notes", "detail", id]);
      if (previousNote) {
        queryClient.setQueryData<Note>(["notes", "detail", id], { ...previousNote, ...updates });
      }
      return { previousNote, id };
    },
    onSuccess: (_data, variables, context) => {
      const { id, ...updates } = variables;
      const field = detectField(updates as Record<string, unknown>, NOTE_FIELD_LABELS, "Note");

      const undoPayload = buildUndoPayload(
        context?.previousNote as Record<string, unknown> | undefined,
        updates as Record<string, unknown>,
        NOTE_UNDO_FIELDS,
      );

      if (undoPayload) {
        toast.success(`${field} updated`, {
          action: { label: "Undo", onClick: () => {
            patchJson<Note>(`/api/notes/${id}`, undoPayload)
              .then(() => {
                toast.success("Undone");
                queryClient.invalidateQueries({ queryKey: ["notes"] });
              })
              .catch((err: Error) => {
                toast.error("Undo failed", { description: err.message });
              });
          }},
          duration: 5000,
        });
      } else {
        toast.success(`${field} updated`);
      }
    },
    onError: (err, _vars, context) => {
      toast.error("Failed to update note", { description: err.message });
      if (context?.previousNote) {
        queryClient.setQueryData(["notes", "detail", context.id], context.previousNote);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["notes"] });
    },
  });
}

function invalidateAllEntities(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: ["tasks"] });
  queryClient.invalidateQueries({ queryKey: ["epics"] });
  queryClient.invalidateQueries({ queryKey: ["plans"] });
  queryClient.invalidateQueries({ queryKey: ["notes"] });
  queryClient.invalidateQueries({ queryKey: ["archived"] });
  queryClient.invalidateQueries({ queryKey: ["summary"] });
}

export function useArchiveItem() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => postAction<{ id: string; status: string }>(`/api/items/${id}/archive`),
    onSuccess: (data) => {
      invalidateAllEntities(queryClient);
      toast.success("Archived", {
        description: data.id,
        action: { label: "Undo", onClick: () => {
          postAction(`/api/items/${data.id}/unarchive`)
            .then(() => {
              toast.success("Undone");
              invalidateAllEntities(queryClient);
            })
            .catch((err: Error) => {
              toast.error("Undo failed", { description: err.message });
            });
        }},
        duration: 5000,
      });
    },
    onError: (err) => {
      toast.error("Failed to archive", { description: err.message });
    },
  });
}

export function useBatchArchive() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (ids: string[]) => {
      const results = await Promise.all(
        ids.map((id) => postAction<{ id: string; status: string }>(`/api/items/${id}/archive`))
      );
      return results;
    },
    onSuccess: (data, variables) => {
      invalidateAllEntities(queryClient);
      toast.success(`Archived ${data.length} item${data.length === 1 ? "" : "s"}`, {
        action: { label: "Undo", onClick: () => {
          Promise.all(variables.map((id) => postAction(`/api/items/${id}/unarchive`)))
            .then(() => {
              toast.success("Undone");
              invalidateAllEntities(queryClient);
            })
            .catch((err: Error) => {
              toast.error("Undo failed", { description: err.message });
            });
        }},
        duration: 5000,
      });
    },
    onError: (err) => {
      toast.error("Failed to archive items", { description: err.message });
    },
  });
}

export function useUpdateConfig() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: Partial<ProjectConfig>) =>
      patchJson<ProjectConfig>("/api/config", body),
    onMutate: async (updates) => {
      await queryClient.cancelQueries({ queryKey: ["config"] });
      const previous = queryClient.getQueryData<ProjectConfig>(["config"]);
      if (previous) {
        queryClient.setQueryData<ProjectConfig>(["config"], { ...previous, ...updates });
      }
      return { previous };
    },
    onSuccess: () => {
      toast.success("Settings updated");
    },
    onError: (err, _vars, context) => {
      toast.error("Failed to update settings", { description: err.message });
      if (context?.previous) {
        queryClient.setQueryData(["config"], context.previous);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["config"] });
    },
  });
}

export function useUnarchiveItem() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => postAction<{ id: string; status: string }>(`/api/items/${id}/unarchive`),
    onSuccess: (data) => {
      toast.success("Restored from archive", { description: data.id });
      invalidateAllEntities(queryClient);
    },
    onError: (err) => {
      toast.error("Failed to restore", { description: err.message });
    },
  });
}
