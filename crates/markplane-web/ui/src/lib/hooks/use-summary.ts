"use client";

import { useQuery } from "@tanstack/react-query";
import { fetcher } from "@/lib/api";
import type { ProjectSummary } from "@/lib/types";

export function useSummary() {
  return useQuery<ProjectSummary>({
    queryKey: ["summary"],
    queryFn: () => fetcher<ProjectSummary>("/api/summary"),
  });
}
