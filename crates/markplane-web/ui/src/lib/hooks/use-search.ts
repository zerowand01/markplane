import { useQuery } from "@tanstack/react-query";
import { fetchList } from "@/lib/api";
import type { SearchResult } from "@/lib/types";

export function useSearch(query: string, options?: { includeArchived?: boolean }) {
  const includeArchived = options?.includeArchived ?? false;
  const params = new URLSearchParams({ q: query });
  if (includeArchived) params.set("include_archived", "true");
  return useQuery({
    queryKey: ["search", query, includeArchived],
    queryFn: () =>
      fetchList<SearchResult>(`/api/search?${params}`),
    enabled: query.length >= 2,
    staleTime: 10_000,
  });
}
