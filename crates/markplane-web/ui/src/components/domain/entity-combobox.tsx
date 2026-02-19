"use client";

import { useState } from "react";
import { Check, ChevronsUpDown, Pencil } from "lucide-react";
import { WikiLinkChip } from "./wiki-link-chip";
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
import { cn } from "@/lib/utils";

interface EntityOption {
  id: string;
  title: string;
}

interface EntityComboboxProps {
  value: string | null;
  options: EntityOption[];
  onSelect: (id: string | null) => void;
  placeholder?: string;
  emptyLabel?: string;
  entityColor?: string;
  /** When true, renders the selected value as a navigable WikiLinkChip with an edit trigger beside it. */
  linkValue?: boolean;
}

export function EntityCombobox({
  value,
  options,
  onSelect,
  placeholder = "Select...",
  emptyLabel = "None",
  entityColor,
  linkValue,
}: EntityComboboxProps) {
  const [open, setOpen] = useState(false);

  const selected = options.find((o) => o.id === value);

  const commandList = (
    <Command>
      <CommandInput placeholder="Search..." />
      <CommandList>
        <CommandEmpty>No results.</CommandEmpty>
        <CommandGroup>
          <CommandItem
            onSelect={() => {
              onSelect(null);
              setOpen(false);
            }}
          >
            <Check
              className={cn(
                "mr-2 size-4",
                !value ? "opacity-100" : "opacity-0"
              )}
            />
            <span className="text-muted-foreground italic">
              {emptyLabel}
            </span>
          </CommandItem>
          {options.map((option) => (
            <CommandItem
              key={option.id}
              value={`${option.id} ${option.title}`}
              onSelect={() => {
                onSelect(option.id);
                setOpen(false);
              }}
            >
              <Check
                className={cn(
                  "mr-2 size-4",
                  value === option.id ? "opacity-100" : "opacity-0"
                )}
              />
              <span className="font-mono text-xs text-muted-foreground mr-1.5">
                {option.id}
              </span>
              <span className="truncate">{option.title}</span>
            </CommandItem>
          ))}
        </CommandGroup>
      </CommandList>
    </Command>
  );

  if (linkValue) {
    return (
      <div className="inline-flex items-center gap-1.5">
        {selected ? (
          <WikiLinkChip id={selected.id} />
        ) : (
          <span className="text-sm text-muted-foreground italic">{placeholder}</span>
        )}
        <Popover open={open} onOpenChange={setOpen}>
          <PopoverTrigger className="cursor-pointer text-muted-foreground hover:text-foreground transition-colors">
            <Pencil className="size-3" />
          </PopoverTrigger>
          <PopoverContent className="w-80 p-0" align="start">
            {commandList}
          </PopoverContent>
        </Popover>
      </div>
    );
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger className="inline-flex items-center gap-1 cursor-pointer text-left">
          {selected ? (
            <span
              className="font-mono text-sm px-1.5 py-0.5 rounded"
              style={
                entityColor
                  ? {
                      backgroundColor: `color-mix(in oklch, ${entityColor} 15%, transparent)`,
                      color: entityColor,
                    }
                  : undefined
              }
            >
              {selected.id}
            </span>
          ) : (
            <span className="text-sm text-muted-foreground italic">{placeholder}</span>
          )}
          <ChevronsUpDown className="size-3 opacity-50" />
      </PopoverTrigger>
      <PopoverContent className="w-80 p-0" align="start">
        {commandList}
      </PopoverContent>
    </Popover>
  );
}
