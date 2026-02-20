"use client";

import { useState } from "react";
import { useNote } from "@/lib/hooks/use-notes";
import { useUpdateNote, useArchiveItem } from "@/lib/hooks/use-mutations";
import { MarkdownRenderer } from "./markdown-renderer";
import { MarkdownEditor } from "./markdown-editor";
import { WikiLinkChip } from "./wiki-link-chip";
import { InlineEdit } from "./inline-edit";
import { TagEditor } from "./tag-editor";
import { FieldRow, EmptyValue } from "./field-row";
import {
  Sheet,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { ResizableSheetContent } from "./resizable-sheet-content";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { Pencil, Archive } from "lucide-react";
import { Button } from "@/components/ui/button";
import { NOTE_STATUS_CONFIG } from "@/lib/constants";
import type { NoteStatus } from "@/lib/types";

const ALL_NOTE_STATUSES: NoteStatus[] = ["draft", "active", "archived"];

export function NoteDetailSheet({
  noteId,
  open,
  onOpenChange,
}: {
  noteId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [isEditingBody, setIsEditingBody] = useState(false);
  const { data: note, isLoading } = useNote(noteId || "", {
    enabled: !isEditingBody,
  });
  const updateNote = useUpdateNote();
  const archiveItem = useArchiveItem();

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
                <div className="flex-1" />
                {note.status === "archived" && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 px-2 gap-1 text-muted-foreground hover:text-foreground"
                    onClick={() => {
                      archiveItem.mutate(note.id);
                      onOpenChange(false);
                    }}
                    disabled={archiveItem.isPending}
                  >
                    <Archive className="size-3.5" />
                    <span className="text-xs">Archive</span>
                  </Button>
                )}
              </div>
              <SheetTitle className="text-left text-xl">
                <InlineEdit
                  value={note.title}
                  onSave={(title) =>
                    updateNote.mutate({ id: note.id, title })
                  }
                />
              </SheetTitle>
            </SheetHeader>

            <div className="space-y-4 px-4 pb-6">
              <div className="space-y-0.5 text-sm">
                <FieldRow label="Status" editable>
                  <DropdownMenu>
                    <DropdownMenuTrigger className="cursor-pointer">
                      <span
                        className="inline-flex items-center gap-1.5"
                        style={{ color: `var(--status-${note.status})` }}
                      >
                        {(() => {
                          const Icon = NOTE_STATUS_CONFIG[note.status]?.icon;
                          return Icon ? <Icon className="size-3.5 text-current" /> : null;
                        })()}
                        <span className="text-sm">{NOTE_STATUS_CONFIG[note.status]?.label}</span>
                      </span>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      {ALL_NOTE_STATUSES.map((s) => (
                        <DropdownMenuItem
                          key={s}
                          onClick={() =>
                            updateNote.mutate({ id: note.id, status: s })
                          }
                        >
                          {(() => {
                            const Icon = NOTE_STATUS_CONFIG[s]?.icon;
                            return Icon ? (
                              <Icon
                                className="mr-2 size-4 text-current"
                                style={{ color: `var(--status-${s})` }}
                              />
                            ) : null;
                          })()}
                          {NOTE_STATUS_CONFIG[s]?.label}
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuContent>
                  </DropdownMenu>
                </FieldRow>

                <FieldRow label="Type">
                  <span className="text-sm uppercase">{note.type}</span>
                </FieldRow>

                <FieldRow label="Tags" editable>
                  <TagEditor
                    tags={note.tags}
                    onSave={(tags) =>
                      updateNote.mutate({ id: note.id, tags })
                    }
                  />
                </FieldRow>

                <FieldRow label="Related">
                  {note.related.length > 0 ? (
                    <div className="flex flex-wrap gap-1.5">
                      {note.related.map((id) => (
                        <WikiLinkChip key={id} id={id} />
                      ))}
                    </div>
                  ) : (
                    <EmptyValue />
                  )}
                </FieldRow>

                <FieldRow label="Created">
                  <span className="text-muted-foreground">{note.created}</span>
                </FieldRow>

                <FieldRow label="Updated">
                  <span className="text-muted-foreground">{note.updated}</span>
                </FieldRow>
              </div>

              <Separator />

              {isEditingBody ? (
                <MarkdownEditor
                  content={note.body}
                  onSave={(body) => {
                    updateNote.mutate({ id: note.id, body });
                    setIsEditingBody(false);
                  }}
                  onCancel={() => setIsEditingBody(false)}
                  isLoading={updateNote.isPending}
                />
              ) : (
                <div className="group relative">
                  <button
                    type="button"
                    onClick={() => setIsEditingBody(true)}
                    className="sticky top-11 float-right ml-2 p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-accent transition-opacity cursor-pointer z-10"
                  >
                    <Pencil className="size-4 text-primary/50 group-hover:text-primary" />
                  </button>
                  {note.body.trim() ? (
                    <MarkdownRenderer content={note.body} />
                  ) : (
                    <button
                      type="button"
                      onClick={() => setIsEditingBody(true)}
                      className="text-sm text-muted-foreground italic hover:text-foreground cursor-pointer"
                    >
                      Click to add content...
                    </button>
                  )}
                </div>
              )}
            </div>
          </>
        )}
      </ResizableSheetContent>
    </Sheet>
  );
}
