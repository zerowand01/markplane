import { useQuery } from "@tanstack/react-query";
import { fetcher } from "@/lib/api";
import type { GraphData } from "@/lib/types";

export function useGraph(focusId?: string) {
  const path = focusId ? `/api/graph/${focusId}` : "/api/graph";
  return useQuery({
    queryKey: ["graph", focusId ?? "all"],
    queryFn: () => fetcher<GraphData>(path),
    staleTime: 30_000,
  });
}
