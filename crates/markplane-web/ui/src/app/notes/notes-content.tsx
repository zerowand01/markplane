"use client";

import { useState } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { useNotes } from "@/lib/hooks/use-notes";
import { NoteDetailSheet } from "@/components/domain/note-detail-sheet";
import { CreateDialog } from "@/components/domain/create-dialog";
import { Button } from "@/components/ui/button";

import { Skeleton } from "@/components/ui/skeleton";
import { NOTE_TYPE_CONFIG } from "@/lib/constants";
import { useConfig } from "@/lib/hooks/use-config";
import { GenericStatusBadge } from "@/components/domain/status-badge";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";
import { Plus } from "lucide-react";

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

export function NotesContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const { data, isLoading, error, refetch } = useNotes();
  const { data: config } = useConfig();
  const [createOpen, setCreateOpen] = useState(false);

  const selectedNoteId = searchParams.get("note");

  const notes = data ?? [];

  // Build ordered type list from config, plus any types found in data but not in config
  const configTypes = config?.note_types ?? ["research", "analysis", "idea", "decision", "meeting"];
  const dataTypes = [...new Set(notes.map((n) => n.type))];
  const orderedTypes = [...configTypes, ...dataTypes.filter((t) => !configTypes.includes(t))];

  const grouped: Record<string, typeof notes> = {};
  for (const t of orderedTypes) {
    const items = notes.filter((n) => n.type === t);
    if (items.length > 0) {
      grouped[t] = items;
    }
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
        <p className="text-muted-foreground">Failed to load notes.</p>
        <p className="text-sm text-destructive">{error.message}</p>
        <Button variant="outline" size="sm" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  return (
    <PageTransition>
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold">Notes</h1>
        <Button
          variant="outline"
          className="text-xs gap-1 cursor-pointer"
          style={{
            color: "var(--entity-note)",
            borderColor: "var(--entity-note)",
            backgroundColor: "color-mix(in oklch, var(--entity-note) 8%, transparent)",
          }}
          onClick={() => setCreateOpen(true)}
        >
          <Plus className="size-3.5" /> New Note
        </Button>
      </div>

      {isLoading ? (
        <div className="space-y-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <Skeleton key={i} className="h-20 w-full" />
          ))}
        </div>
      ) : notes.length === 0 ? (
        <EmptyState
          title="No notes yet"
          description="Capture ideas, research, and decisions with markplane note &quot;title&quot;"
        />
      ) : (
        Object.entries(grouped).map(
          ([type, items]) => (
              <div key={type} className="space-y-3">
                <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
                  {NOTE_TYPE_CONFIG[type]?.label ?? capitalize(type)} ({items.length})
                </h2>
                <div className="space-y-2">
                  {items.map((note) => {
                    const isDraft = note.status === "draft";
                    return (
                      <div
                        key={note.id}
                        className={`rounded-md border px-3 py-2 cursor-pointer transition-colors hover:border-muted-foreground/30 ${
                          isDraft
                            ? "bg-amber-50 border-amber-200/60 dark:bg-amber-950/30 dark:border-amber-800/40"
                            : "bg-card"
                        }`}
                        onClick={() => {
                          const params = new URLSearchParams(searchParams);
                          params.set("note", note.id);
                          router.push(`/notes?${params.toString()}`);
                        }}
                      >
                        <div className="flex items-center gap-2">
                          <span
                            className="font-mono text-sm shrink-0"
                            style={{ color: "var(--entity-note)" }}
                          >
                            {note.id}
                          </span>
                          {isDraft && <GenericStatusBadge status="draft" />}
                          <span className="text-base font-medium truncate flex-1">
                            {note.title}
                          </span>
                          {note.tags.length > 0 && (
                            <div className="flex gap-1 shrink-0">
                              {note.tags.map((tag) => (
                                <span
                                  key={tag}
                                  className="text-xs text-muted-foreground"
                                >
                                  #{tag}
                                </span>
                              ))}
                            </div>
                          )}
                          <span className="text-xs text-muted-foreground whitespace-nowrap shrink-0">
                            {note.updated}
                          </span>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )
        )
      )}

      <NoteDetailSheet
        noteId={selectedNoteId}
        open={!!selectedNoteId}
        onOpenChange={(open) => {
          if (!open) {
            const params = new URLSearchParams(searchParams);
            params.delete("note");
            router.push(`/notes?${params.toString()}`);
          }
        }}
      />

      <CreateDialog
        kind="note"
        open={createOpen}
        onOpenChange={setCreateOpen}
        onCreated={(id) => {
          const params = new URLSearchParams(searchParams);
          params.set("note", id);
          router.push(`/notes?${params.toString()}`);
        }}
      />
    </div>
    </PageTransition>
  );
}

export function NotesSkeleton() {
  return (
    <div className="space-y-3">
      {Array.from({ length: 3 }).map((_, i) => (
        <Skeleton key={i} className="h-20 w-full" />
      ))}
    </div>
  );
}
