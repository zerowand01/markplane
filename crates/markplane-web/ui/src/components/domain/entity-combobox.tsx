"use client";

import { useState } from "react";
import { Check, ChevronsUpDown } from "lucide-react";
import { Button } from "@/components/ui/button";
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
}

export function EntityCombobox({
  value,
  options,
  onSelect,
  placeholder = "Select...",
  emptyLabel = "None",
  entityColor,
}: EntityComboboxProps) {
  const [open, setOpen] = useState(false);

  const selected = options.find((o) => o.id === value);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          className="h-auto px-2 py-0.5 font-normal justify-start"
        >
          {selected ? (
            <span
              className="font-mono text-sm"
              style={
                entityColor
                  ? {
                      backgroundColor: `color-mix(in oklch, ${entityColor} 15%, transparent)`,
                      color: entityColor,
                      padding: "1px 6px",
                      borderRadius: "4px",
                    }
                  : undefined
              }
            >
              {selected.id}
            </span>
          ) : (
            <span className="text-muted-foreground italic">{placeholder}</span>
          )}
          <ChevronsUpDown className="ml-1 size-3 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-64 p-0" align="start">
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
      </PopoverContent>
    </Popover>
  );
}
