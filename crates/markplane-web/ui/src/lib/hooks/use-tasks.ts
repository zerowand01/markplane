"use client";

import { useQuery } from "@tanstack/react-query";
import { fetcher, fetchList } from "@/lib/api";
import type { Task, TaskStatus, Priority } from "@/lib/types";

interface TaskFilter {
  status?: TaskStatus[];
  priority?: Priority[];
  epic?: string;
  tags?: string[];
  assignee?: string;
}

export function useTasks(filter?: TaskFilter) {
  const params = new URLSearchParams();
  if (filter?.status) params.set("status", filter.status.join(","));
  if (filter?.priority) params.set("priority", filter.priority.join(","));
  if (filter?.epic) params.set("epic", filter.epic);
  if (filter?.tags) params.set("tags", filter.tags.join(","));
  if (filter?.assignee) params.set("assignee", filter.assignee);

  const query = params.toString();
  return useQuery({
    queryKey: ["tasks", "list", filter],
    queryFn: () => fetchList<Task>(`/api/tasks${query ? `?${query}` : ""}`),
    select: (result) => result.data,
  });
}

export function useTask(id: string, options?: { enabled?: boolean }) {
  return useQuery<Task>({
    queryKey: ["tasks", "detail", id],
    queryFn: () => fetcher<Task>(`/api/tasks/${id}`),
    enabled: !!id && (options?.enabled ?? true),
  });
}
