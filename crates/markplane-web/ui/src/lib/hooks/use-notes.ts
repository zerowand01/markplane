"use client";

import { useQuery } from "@tanstack/react-query";
import { fetcher, fetchList } from "@/lib/api";
import type { Note } from "@/lib/types";

export function useNotes() {
  return useQuery({
    queryKey: ["notes", "list"],
    queryFn: () => fetchList<Note>("/api/notes"),
    select: (result) => result.data,
  });
}

export function useNote(id: string, options?: { enabled?: boolean }) {
  return useQuery<Note>({
    queryKey: ["notes", "detail", id],
    queryFn: () => fetcher<Note>(`/api/notes/${id}`),
    enabled: !!id && (options?.enabled ?? true),
  });
}
