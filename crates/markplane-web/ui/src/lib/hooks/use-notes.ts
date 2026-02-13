"use client";

import { useQuery } from "@tanstack/react-query";
import { fetcher, fetchList } from "@/lib/api";
import type { Note } from "@/lib/types";

export function useNotes() {
  return useQuery({
    queryKey: ["notes"],
    queryFn: () => fetchList<Note>("/api/notes"),
    select: (result) => result.data,
  });
}

export function useNote(id: string) {
  return useQuery<Note>({
    queryKey: ["notes", id],
    queryFn: () => fetcher<Note>(`/api/notes/${id}`),
    enabled: !!id,
  });
}
