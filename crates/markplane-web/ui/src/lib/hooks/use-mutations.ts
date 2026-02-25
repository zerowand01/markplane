"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { patchJson, postJson, postAction } from "@/lib/api";
import { toast } from "sonner";
import type { Task, Epic, Plan, Note, CreateTaskRequest, CreateEpicRequest, CreatePlanRequest, CreateNoteRequest, UpdateTaskRequest, UpdateEpicRequest, UpdatePlanRequest, UpdateNoteRequest } from "@/lib/types";

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
    onSuccess: (_data, variables) => {
      const field = variables.status
        ? "Status"
        : variables.priority
          ? "Priority"
          : variables.effort
            ? "Effort"
            : variables.type
              ? "Type"
              : variables.title
                ? "Title"
                : variables.tags
                  ? "Tags"
                  : variables.epic !== undefined
                    ? "Epic"
                    : variables.assignee !== undefined
                      ? "Assignee"
                      : variables.body !== undefined
                        ? "Content"
                        : "Task";
      toast.success(`${field} updated`);
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
    onSuccess: (_data, variables) => {
      const field = variables.status
        ? "Status"
        : variables.priority
          ? "Priority"
          : variables.title
            ? "Title"
            : variables.tags
              ? "Tags"
              : variables.started !== undefined
                ? "Start date"
                : variables.target !== undefined
                  ? "Target date"
                  : variables.body !== undefined
                    ? "Content"
                    : "Epic";
      toast.success(`${field} updated`);
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
    onSuccess: (_data, variables) => {
      const field = variables.status
        ? "Status"
        : variables.title
          ? "Title"
          : variables.body !== undefined
            ? "Content"
            : "Plan";
      toast.success(`${field} updated`);
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
    onSuccess: (_data, variables) => {
      const field = variables.status
        ? "Status"
        : variables.type
          ? "Type"
          : variables.title
            ? "Title"
            : variables.tags
              ? "Tags"
              : variables.related
                ? "Related"
                : variables.body !== undefined
                  ? "Content"
                  : "Note";
      toast.success(`${field} updated`);
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

export function useArchiveItem() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => postAction<{ id: string; status: string }>(`/api/items/${id}/archive`),
    onSuccess: (data) => {
      toast.success("Archived", { description: data.id });
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
      queryClient.invalidateQueries({ queryKey: ["epics"] });
      queryClient.invalidateQueries({ queryKey: ["plans"] });
      queryClient.invalidateQueries({ queryKey: ["notes"] });
      queryClient.invalidateQueries({ queryKey: ["archived"] });
      queryClient.invalidateQueries({ queryKey: ["summary"] });
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
    onSuccess: (data) => {
      toast.success(`Archived ${data.length} item${data.length === 1 ? "" : "s"}`);
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
      queryClient.invalidateQueries({ queryKey: ["epics"] });
      queryClient.invalidateQueries({ queryKey: ["plans"] });
      queryClient.invalidateQueries({ queryKey: ["notes"] });
      queryClient.invalidateQueries({ queryKey: ["archived"] });
      queryClient.invalidateQueries({ queryKey: ["summary"] });
    },
    onError: (err) => {
      toast.error("Failed to archive items", { description: err.message });
    },
  });
}

export function useUnarchiveItem() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => postAction<{ id: string; status: string }>(`/api/items/${id}/unarchive`),
    onSuccess: (data) => {
      toast.success("Restored from archive", { description: data.id });
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
      queryClient.invalidateQueries({ queryKey: ["epics"] });
      queryClient.invalidateQueries({ queryKey: ["plans"] });
      queryClient.invalidateQueries({ queryKey: ["notes"] });
      queryClient.invalidateQueries({ queryKey: ["archived"] });
      queryClient.invalidateQueries({ queryKey: ["summary"] });
    },
    onError: (err) => {
      toast.error("Failed to restore", { description: err.message });
    },
  });
}
