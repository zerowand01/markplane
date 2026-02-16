"use client";

import { Suspense } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { useNotes } from "@/lib/hooks/use-notes";
import { NoteDetailSheet } from "@/components/domain/note-detail-sheet";
import { Button } from "@/components/ui/button";

import { Skeleton } from "@/components/ui/skeleton";
import { NOTE_STATUS_CONFIG } from "@/lib/constants";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";

function NotesContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const { data, isLoading, error, refetch } = useNotes();

  const selectedNoteId = searchParams.get("note");

  const notes = data ?? [];

  const grouped = {
    active: notes.filter((n) => n.status === "active"),
    draft: notes.filter((n) => n.status === "draft"),
    archived: notes.filter((n) => n.status === "archived"),
  };

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
      <div>
        <h1 className="text-2xl font-bold">Notes</h1>
        <p className="text-sm text-muted-foreground mt-1">
          {notes.length} research notes, ideas & decisions
        </p>
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
          ([status, items]) =>
            items.length > 0 && (
              <div key={status} className="space-y-3">
                <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
                  {NOTE_STATUS_CONFIG[status]?.label ?? status} ({items.length})
                </h2>
                <div className="space-y-2">
                  {items.map((note) => (
                    <div
                      key={note.id}
                      className="rounded-lg border bg-card p-4 cursor-pointer hover:border-ring transition-colors"
                      onClick={() => {
                        const params = new URLSearchParams(searchParams);
                        params.set("note", note.id);
                        router.push(`/notes?${params.toString()}`);
                      }}
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2 mb-1">
                            <span
                              className="font-mono text-sm"
                              style={{ color: "var(--entity-note)" }}
                            >
                              {note.id}
                            </span>
                            <span className="text-xs px-2 py-0.5 rounded bg-secondary text-secondary-foreground uppercase">
                              {note.type}
                            </span>
                            <span
                              className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded"
                              style={{
                                backgroundColor: `color-mix(in oklch, var(--status-${note.status}) 15%, transparent)`,
                                color: `var(--status-${note.status})`,
                              }}
                            >
                              {NOTE_STATUS_CONFIG[note.status]?.icon}{" "}
                              {NOTE_STATUS_CONFIG[note.status]?.label}
                            </span>
                          </div>
                          <h3 className="text-base font-medium">
                            {note.title}
                          </h3>
                          {note.tags.length > 0 && (
                            <div className="flex flex-wrap gap-1 mt-1.5">
                              {note.tags.map((tag) => (
                                <span
                                  key={tag}
                                  className="text-sm text-muted-foreground"
                                >
                                  #{tag}
                                </span>
                              ))}
                            </div>
                          )}
                        </div>
                        <span className="text-xs text-muted-foreground whitespace-nowrap">
                          {note.updated}
                        </span>
                      </div>
                    </div>
                  ))}
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
    </div>
    </PageTransition>
  );
}

export default function NotesPage() {
  return (
    <Suspense
      fallback={
        <div className="space-y-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <Skeleton key={i} className="h-20 w-full" />
          ))}
        </div>
      }
    >
      <NotesContent />
    </Suspense>
  );
}
