"use client";

import { useEffect, useState, useCallback } from "react";
import { useRouter } from "next/navigation";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/components/ui/command";
import { useTasks } from "@/lib/hooks/use-tasks";
import { useEpics } from "@/lib/hooks/use-epics";
import { usePlans } from "@/lib/hooks/use-plans";
import { useNotes } from "@/lib/hooks/use-notes";
import { postAction } from "@/lib/api";
import { useQueryClient } from "@tanstack/react-query";

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const router = useRouter();
  const queryClient = useQueryClient();

  const { data: tasksData } = useTasks();
  const { data: epicsData } = useEpics();
  const { data: plansData } = usePlans();
  const { data: notesData } = useNotes();

  const tasks = tasksData ?? [];
  const epics = epicsData ?? [];
  const plans = plansData ?? [];
  const notes = notesData ?? [];

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen((prev) => !prev);
      }
    };
    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, []);

  const navigate = useCallback(
    (path: string) => {
      setOpen(false);
      router.push(path);
    },
    [router]
  );

  const triggerSync = useCallback(async () => {
    setOpen(false);
    await postAction("/api/sync");
    queryClient.invalidateQueries();
  }, [queryClient]);

  return (
    <CommandDialog open={open} onOpenChange={setOpen}>
      <CommandInput placeholder="Search items, commands, or navigate..." />
      <CommandList>
        <CommandEmpty>No results found.</CommandEmpty>

        {tasks.length > 0 && (
          <CommandGroup heading="Tasks">
            {tasks.slice(0, 8).map((task) => (
              <CommandItem
                key={task.id}
                value={`${task.id} ${task.title}`}
                onSelect={() => navigate(`/backlog?task=${task.id}`)}
              >
                <span
                  className="mr-2 font-mono text-xs"
                  style={{ color: "var(--entity-task)" }}
                >
                  {task.id}
                </span>
                <span className="flex-1 truncate">{task.title}</span>
                <span className="text-xs text-muted-foreground ml-2">
                  {task.status}
                </span>
              </CommandItem>
            ))}
          </CommandGroup>
        )}

        {epics.length > 0 && (
          <CommandGroup heading="Epics">
            {epics.map((epic) => (
              <CommandItem
                key={epic.id}
                value={`${epic.id} ${epic.title}`}
                onSelect={() => navigate(`/epics?epic=${epic.id}`)}
              >
                <span
                  className="mr-2 font-mono text-xs"
                  style={{ color: "var(--entity-epic)" }}
                >
                  {epic.id}
                </span>
                <span className="flex-1 truncate">{epic.title}</span>
                <span className="text-xs text-muted-foreground ml-2">
                  {epic.status}
                </span>
              </CommandItem>
            ))}
          </CommandGroup>
        )}

        {plans.length > 0 && (
          <CommandGroup heading="Plans">
            {plans.map((plan) => (
              <CommandItem
                key={plan.id}
                value={`${plan.id} ${plan.title}`}
                onSelect={() => navigate(`/plans?plan=${plan.id}`)}
              >
                <span
                  className="mr-2 font-mono text-xs"
                  style={{ color: "var(--entity-plan)" }}
                >
                  {plan.id}
                </span>
                <span className="flex-1 truncate">{plan.title}</span>
                <span className="text-xs text-muted-foreground ml-2">
                  {plan.status}
                </span>
              </CommandItem>
            ))}
          </CommandGroup>
        )}

        {notes.length > 0 && (
          <CommandGroup heading="Notes">
            {notes.map((note) => (
              <CommandItem
                key={note.id}
                value={`${note.id} ${note.title}`}
                onSelect={() => navigate(`/notes?note=${note.id}`)}
              >
                <span
                  className="mr-2 font-mono text-xs"
                  style={{ color: "var(--entity-note)" }}
                >
                  {note.id}
                </span>
                <span className="flex-1 truncate">{note.title}</span>
                <span className="text-xs text-muted-foreground ml-2">
                  {note.status}
                </span>
              </CommandItem>
            ))}
          </CommandGroup>
        )}

        <CommandSeparator />

        <CommandGroup heading="Navigation">
          <CommandItem onSelect={() => navigate("/dashboard")}>
            Go to Dashboard
            <span className="ml-auto text-xs text-muted-foreground">g d</span>
          </CommandItem>
          <CommandItem onSelect={() => navigate("/backlog")}>
            Go to Backlog
            <span className="ml-auto text-xs text-muted-foreground">g b</span>
          </CommandItem>
          <CommandItem onSelect={() => navigate("/epics")}>
            Go to Epics
            <span className="ml-auto text-xs text-muted-foreground">g e</span>
          </CommandItem>
          <CommandItem onSelect={() => navigate("/plans")}>
            Go to Plans
            <span className="ml-auto text-xs text-muted-foreground">g p</span>
          </CommandItem>
          <CommandItem onSelect={() => navigate("/notes")}>
            Go to Notes
            <span className="ml-auto text-xs text-muted-foreground">g n</span>
          </CommandItem>
          <CommandItem onSelect={() => navigate("/graph")}>
            Go to Dependencies
            <span className="ml-auto text-xs text-muted-foreground">g g</span>
          </CommandItem>
          <CommandItem onSelect={() => navigate("/search")}>
            Go to Search
            <span className="ml-auto text-xs text-muted-foreground">g s</span>
          </CommandItem>
        </CommandGroup>

        <CommandSeparator />

        <CommandGroup heading="Actions">
          <CommandItem onSelect={() => triggerSync()}>
            Sync project
            <span className="ml-auto text-xs text-muted-foreground">⌘S</span>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  );
}
