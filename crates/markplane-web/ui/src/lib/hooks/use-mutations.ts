"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { patchJson, postJson, deleteAction } from "@/lib/api";
import { toast } from "sonner";
import type { Task, Epic, Plan, Note, CreateTaskRequest, UpdateTaskRequest, UpdateEpicRequest, UpdatePlanRequest, UpdateNoteRequest } from "@/lib/types";

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
      }>({ queryKey: ["tasks"] });

      // Optimistically update every cached task list that contains this task
      queryClient.setQueriesData<{ data: Task[]; total: number }>(
        { queryKey: ["tasks"] },
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
      const previousTask = queryClient.getQueryData<Task>(["tasks", id]);
      if (previousTask) {
        queryClient.setQueryData<Task>(["tasks", id], {
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
        queryClient.setQueryData(["tasks", context.id], context.previousTask);
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

export function useUpdateEpic() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, ...body }: UpdateEpicRequest & { id: string }) =>
      patchJson<Epic>(`/api/epics/${id}`, body),
    onMutate: async ({ id, ...updates }) => {
      await queryClient.cancelQueries({ queryKey: ["epics"] });
      const previousEpic = queryClient.getQueryData<Epic>(["epics", id]);
      if (previousEpic) {
        queryClient.setQueryData<Epic>(["epics", id], { ...previousEpic, ...updates });
      }
      return { previousEpic, id };
    },
    onSuccess: (_data, variables) => {
      const field = variables.status ? "Status" : variables.priority ? "Priority" : "Epic";
      toast.success(`${field} updated`);
    },
    onError: (err, _vars, context) => {
      toast.error("Failed to update epic", { description: err.message });
      if (context?.previousEpic) {
        queryClient.setQueryData(["epics", context.id], context.previousEpic);
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
      const previousPlan = queryClient.getQueryData<Plan>(["plans", id]);
      if (previousPlan) {
        queryClient.setQueryData<Plan>(["plans", id], { ...previousPlan, ...updates });
      }
      return { previousPlan, id };
    },
    onSuccess: () => {
      toast.success("Status updated");
    },
    onError: (err, _vars, context) => {
      toast.error("Failed to update plan", { description: err.message });
      if (context?.previousPlan) {
        queryClient.setQueryData(["plans", context.id], context.previousPlan);
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
      const previousNote = queryClient.getQueryData<Note>(["notes", id]);
      if (previousNote) {
        queryClient.setQueryData<Note>(["notes", id], { ...previousNote, ...updates });
      }
      return { previousNote, id };
    },
    onSuccess: () => {
      toast.success("Status updated");
    },
    onError: (err, _vars, context) => {
      toast.error("Failed to update note", { description: err.message });
      if (context?.previousNote) {
        queryClient.setQueryData(["notes", context.id], context.previousNote);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["notes"] });
    },
  });
}

export function useDeleteTask() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => deleteAction<Task>(`/api/tasks/${id}`),
    onSuccess: (data) => {
      toast.success("Task archived", { description: `${data.id}: ${data.title}` });
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
      queryClient.invalidateQueries({ queryKey: ["summary"] });
    },
    onError: (err) => {
      toast.error("Failed to archive task", { description: err.message });
    },
  });
}
