"use client";

import { useNote } from "@/lib/hooks/use-notes";
import { MarkdownRenderer } from "./markdown-renderer";
import { WikiLinkChip } from "./wiki-link-chip";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { NOTE_STATUS_CONFIG } from "@/lib/constants";

export function NoteDetailSheet({
  noteId,
  open,
  onOpenChange,
}: {
  noteId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const { data: note, isLoading } = useNote(noteId || "");

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="sm:max-w-[540px] overflow-y-auto">
        {isLoading || !note ? (
          <SheetHeader>
            <SheetTitle>
              <Skeleton className="h-6 w-48" />
            </SheetTitle>
            <div className="space-y-4 pt-4">
              <Skeleton className="h-4 w-32" />
              <Skeleton className="h-40 w-full" />
            </div>
          </SheetHeader>
        ) : (
          <>
            <SheetHeader>
              <div className="flex items-center gap-2">
                <span
                  className="font-mono text-xs"
                  style={{ color: "var(--entity-note)" }}
                >
                  {note.id}
                </span>
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-secondary text-secondary-foreground uppercase">
                  {note.type}
                </span>
              </div>
              <SheetTitle className="text-left text-lg">
                {note.title}
              </SheetTitle>
            </SheetHeader>

            <div className="space-y-4 pt-4">
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <span className="text-xs text-muted-foreground block mb-1">
                    Status
                  </span>
                  <span
                    className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded"
                    style={{
                      backgroundColor: `color-mix(in oklch, var(--status-${note.status}) 15%, transparent)`,
                      color: `var(--status-${note.status})`,
                    }}
                  >
                    <span>{NOTE_STATUS_CONFIG[note.status]?.icon}</span>
                    <span>{NOTE_STATUS_CONFIG[note.status]?.label}</span>
                  </span>
                </div>
              </div>

              {note.tags.length > 0 && (
                <div>
                  <span className="text-xs text-muted-foreground block mb-1">
                    Tags
                  </span>
                  <div className="flex flex-wrap gap-1.5">
                    {note.tags.map((tag) => (
                      <span
                        key={tag}
                        className="text-xs text-muted-foreground bg-secondary px-1.5 py-0.5 rounded"
                      >
                        #{tag}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              {note.related.length > 0 && (
                <div>
                  <span className="text-xs text-muted-foreground block mb-1">
                    Related
                  </span>
                  <div className="flex flex-wrap gap-1.5">
                    {note.related.map((id) => (
                      <WikiLinkChip key={id} id={id} />
                    ))}
                  </div>
                </div>
              )}

              <div className="flex gap-4 text-xs text-muted-foreground">
                <span>Created {note.created}</span>
                <span>Updated {note.updated}</span>
              </div>

              <Separator />

              {note.body.trim() ? (
                <MarkdownRenderer content={note.body} />
              ) : (
                <p className="text-sm text-muted-foreground italic">
                  No content.
                </p>
              )}
            </div>
          </>
        )}
      </SheetContent>
    </Sheet>
  );
}
