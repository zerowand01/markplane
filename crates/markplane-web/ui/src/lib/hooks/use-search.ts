import { useQuery } from "@tanstack/react-query";
import { fetchList } from "@/lib/api";
import type { SearchResult } from "@/lib/types";

export function useSearch(query: string) {
  return useQuery({
    queryKey: ["search", query],
    queryFn: () =>
      fetchList<SearchResult>(`/api/search?q=${encodeURIComponent(query)}`),
    enabled: query.length >= 2,
    staleTime: 10_000,
  });
}
