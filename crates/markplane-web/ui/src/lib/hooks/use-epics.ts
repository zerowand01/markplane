"use client";

import { useQuery } from "@tanstack/react-query";
import { fetcher, fetchList } from "@/lib/api";
import type { Epic } from "@/lib/types";

export function useEpics() {
  return useQuery({
    queryKey: ["epics"],
    queryFn: () => fetchList<Epic>("/api/epics"),
    select: (result) => result.data,
  });
}

export function useEpic(id: string) {
  return useQuery<Epic>({
    queryKey: ["epics", id],
    queryFn: () => fetcher<Epic>(`/api/epics/${id}`),
    enabled: !!id,
  });
}
