"use client";

import { useQuery } from "@tanstack/react-query";
import { fetcher, fetchList } from "@/lib/api";
import type { DocMeta, DocContent } from "@/lib/types";

export function useDocs() {
  return useQuery({
    queryKey: ["docs", "list"],
    queryFn: () => fetchList<DocMeta>("/api/docs"),
    select: (result) => result.data,
  });
}

export function useDoc(slug: string, options?: { enabled?: boolean }) {
  return useQuery<DocContent>({
    queryKey: ["docs", "detail", slug],
    queryFn: () => fetcher<DocContent>(`/api/docs/${slug}`),
    enabled: !!slug && (options?.enabled ?? true),
  });
}

export function useAllDocContents() {
  const { data: docs } = useDocs();

  return useQuery({
    queryKey: ["docs", "all-content"],
    queryFn: async () => {
      if (!docs || docs.length === 0) return [];
      return Promise.all(
        docs.map((doc) => fetcher<DocContent>(`/api/docs/${doc.slug}`))
      );
    },
    enabled: !!docs && docs.length > 0,
    staleTime: 60_000,
  });
}
