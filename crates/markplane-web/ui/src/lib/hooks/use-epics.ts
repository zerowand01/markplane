"use client";

import { useQuery } from "@tanstack/react-query";
import { fetcher, fetchList } from "@/lib/api";
import type { Epic } from "@/lib/types";

export function useEpics() {
  return useQuery({
    queryKey: ["epics", "list"],
    queryFn: () => fetchList<Epic>("/api/epics"),
    select: (result) => result.data,
  });
}

export function useEpic(id: string, options?: { enabled?: boolean }) {
  return useQuery<Epic>({
    queryKey: ["epics", "detail", id],
    queryFn: () => fetcher<Epic>(`/api/epics/${id}`),
    enabled: !!id && (options?.enabled ?? true),
  });
}
