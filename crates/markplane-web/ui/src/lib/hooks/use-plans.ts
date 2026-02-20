"use client";

import { useQuery } from "@tanstack/react-query";
import { fetcher, fetchList } from "@/lib/api";
import type { Plan } from "@/lib/types";

export function usePlans() {
  return useQuery({
    queryKey: ["plans", "list"],
    queryFn: () => fetchList<Plan>("/api/plans"),
    select: (result) => result.data,
  });
}

export function usePlan(id: string, options?: { enabled?: boolean }) {
  return useQuery<Plan>({
    queryKey: ["plans", "detail", id],
    queryFn: () => fetcher<Plan>(`/api/plans/${id}`),
    enabled: !!id && (options?.enabled ?? true),
  });
}

export function useArchivedPlans() {
  return useQuery({
    queryKey: ["archived", "plans"],
    queryFn: () => fetchList<Plan>("/api/plans?archived=true"),
    select: (result) => result.data,
  });
}
