"use client";

import { useQuery } from "@tanstack/react-query";
import { fetcher } from "@/lib/api";
import type { ProjectConfig } from "@/lib/types";

export function useConfig() {
  return useQuery({
    queryKey: ["config"],
    queryFn: () => fetcher<ProjectConfig>("/api/config"),
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}
