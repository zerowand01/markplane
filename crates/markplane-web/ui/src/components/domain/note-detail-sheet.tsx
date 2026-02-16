"use client";

import { useNote } from "@/lib/hooks/use-notes";
import { MarkdownRenderer } from "./markdown-renderer";
import { WikiLinkChip } from "./wiki-link-chip";
import {
  Sheet,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { ResizableSheetContent } from "./resizable-sheet-content";
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
      <ResizableSheetContent>
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
                  className="font-mono text-sm"
                  style={{ color: "var(--entity-note)" }}
                >
                  {note.id}
                </span>
                <span className="text-xs px-2 py-0.5 rounded bg-secondary text-secondary-foreground uppercase">
                  {note.type}
                </span>
              </div>
              <SheetTitle className="text-left text-xl">
                {note.title}
              </SheetTitle>
            </SheetHeader>

            <div className="space-y-4 px-4 pb-6">
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <span className="text-sm text-muted-foreground block mb-1">
                    Status
                  </span>
                  <span
                    className="inline-flex items-center gap-1 text-sm px-2 py-0.5 rounded"
                    style={{
                      backgroundColor: `color-mix(in oklch, var(--status-${note.status}) 15%, transparent)`,
                      color: `var(--status-${note.status})`,
                    }}
                  >
                    {(() => {
                      const Icon = NOTE_STATUS_CONFIG[note.status]?.icon;
                      return Icon ? <Icon className="size-3.5 text-current" /> : null;
                    })()}
                    <span>{NOTE_STATUS_CONFIG[note.status]?.label}</span>
                  </span>
                </div>
              </div>

              {note.tags.length > 0 && (
                <div>
                  <span className="text-sm text-muted-foreground block mb-1">
                    Tags
                  </span>
                  <div className="flex flex-wrap gap-1.5">
                    {note.tags.map((tag) => (
                      <span
                        key={tag}
                        className="text-sm text-muted-foreground bg-secondary px-2 py-0.5 rounded"
                      >
                        #{tag}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              {note.related.length > 0 && (
                <div>
                  <span className="text-sm text-muted-foreground block mb-1">
                    Related
                  </span>
                  <div className="flex flex-wrap gap-1.5">
                    {note.related.map((id) => (
                      <WikiLinkChip key={id} id={id} />
                    ))}
                  </div>
                </div>
              )}

              <div className="flex gap-4 text-sm text-muted-foreground">
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
      </ResizableSheetContent>
    </Sheet>
  );
}
