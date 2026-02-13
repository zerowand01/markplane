"use client";

import { useQuery } from "@tanstack/react-query";
import { fetcher, fetchList } from "@/lib/api";
import type { Plan } from "@/lib/types";

export function usePlans() {
  return useQuery({
    queryKey: ["plans"],
    queryFn: () => fetchList<Plan>("/api/plans"),
    select: (result) => result.data,
  });
}

export function usePlan(id: string) {
  return useQuery<Plan>({
    queryKey: ["plans", id],
    queryFn: () => fetcher<Plan>(`/api/plans/${id}`),
    enabled: !!id,
  });
}
