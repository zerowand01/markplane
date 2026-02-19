"use client";

import { useState } from "react";
import { X, Plus } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

interface TagEditorProps {
  tags: string[];
  onSave: (tags: string[]) => void;
}

export function TagEditor({ tags, onSave }: TagEditorProps) {
  const [open, setOpen] = useState(false);
  const [newTag, setNewTag] = useState("");

  const handleRemove = (tag: string) => {
    onSave(tags.filter((t) => t !== tag));
  };

  const handleAdd = () => {
    const trimmed = newTag.trim().toLowerCase();
    if (trimmed && !tags.includes(trimmed)) {
      onSave([...tags, trimmed]);
    }
    setNewTag("");
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAdd();
    }
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          type="button"
          className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground cursor-pointer transition-colors"
        >
          {tags.length > 0 ? (
            <span className="flex flex-wrap gap-1.5">
              {tags.map((tag) => (
                <span
                  key={tag}
                  className="text-sm text-muted-foreground bg-secondary px-2 py-0.5 rounded"
                >
                  #{tag}
                </span>
              ))}
            </span>
          ) : (
            <span className="text-sm text-muted-foreground italic">
              None
            </span>
          )}
        </button>
      </PopoverTrigger>
        <PopoverContent className="w-64 p-3" align="start">
          <div className="space-y-2">
            {tags.length > 0 && (
              <div className="flex flex-wrap gap-1.5">
                {tags.map((tag) => (
                  <span
                    key={tag}
                    className="inline-flex items-center gap-1 text-sm bg-secondary px-2 py-0.5 rounded"
                  >
                    #{tag}
                    <button
                      type="button"
                      onClick={() => handleRemove(tag)}
                      className="hover:text-destructive cursor-pointer"
                    >
                      <X className="size-3" />
                    </button>
                  </span>
                ))}
              </div>
            )}
            <div className="flex gap-1.5">
              <Input
                value={newTag}
                onChange={(e) => setNewTag(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="New tag..."
                className="h-7 text-sm"
              />
              <Button
                size="xs"
                variant="ghost"
                onClick={handleAdd}
                disabled={!newTag.trim()}
              >
                <Plus className="size-3" />
              </Button>
            </div>
          </div>
        </PopoverContent>
    </Popover>
  );
}
