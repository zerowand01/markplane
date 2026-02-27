"use client";

import { useState } from "react";
import { Plus } from "lucide-react";
import { WikiLinkChip } from "./wiki-link-chip";
import { EmptyValue } from "./field-row";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

interface EntityRefEditorProps {
  ids: string[];
  options: { id: string; title: string }[];
  onAdd: (id: string) => void;
  onRemove: (id: string) => void;
  emptyText?: string;
}

export function EntityRefEditor({
  ids,
  options,
  onAdd,
  onRemove,
  emptyText = "None",
}: EntityRefEditorProps) {
  const [open, setOpen] = useState(false);

  const available = options.filter((o) => !ids.includes(o.id));

  return (
    <div className="flex flex-wrap items-center gap-1.5">
      {ids.length > 0 ? (
        ids.map((id) => (
          <WikiLinkChip key={id} id={id} onRemove={() => onRemove(id)} />
        ))
      ) : (
        <EmptyValue>{emptyText}</EmptyValue>
      )}
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger className="inline-flex items-center justify-center size-5 rounded text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer">
          <Plus className="size-3.5" />
        </PopoverTrigger>
        <PopoverContent className="w-80 p-0" align="start">
          <Command>
            <CommandInput placeholder="Search..." />
            <CommandList>
              <CommandEmpty>No results.</CommandEmpty>
              <CommandGroup>
                {available.map((option) => (
                  <CommandItem
                    key={option.id}
                    value={`${option.id} ${option.title}`}
                    onSelect={() => {
                      onAdd(option.id);
                      setOpen(false);
                    }}
                  >
                    <span className="font-mono text-xs text-muted-foreground mr-1.5 shrink-0">
                      {option.id}
                    </span>
                    <span className="truncate min-w-0">{option.title}</span>
                  </CommandItem>
                ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    </div>
  );
}
