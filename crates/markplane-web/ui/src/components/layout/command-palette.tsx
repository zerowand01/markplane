"use client";

import { useEffect, useState, useCallback, useRef, type ReactNode } from "react";
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
import { useSearch } from "@/lib/hooks/use-search";
import { postAction } from "@/lib/api";
import { useQueryClient } from "@tanstack/react-query";
import { PREFIX_CONFIG } from "@/lib/constants";
import { StatusBadge } from "@/components/domain/status-badge";
import { PriorityIndicator } from "@/components/domain/priority-indicator";
import type { SearchResult, Priority, TaskStatus } from "@/lib/types";

function highlightMatch(text: string, query: string): ReactNode {
  if (!query) return text;
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const regex = new RegExp(`(${escaped})`, "gi");
  const parts = text.split(regex);
  return parts.map((part, i) =>
    regex.test(part) ? (
      <mark key={i} className="bg-primary/20 text-foreground rounded px-0.5">
        {part}
      </mark>
    ) : (
      part
    )
  );
}

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const router = useRouter();
  const queryClient = useQueryClient();

  const { data: tasksData } = useTasks();
  const { data: epicsData } = useEpics();
  const { data: plansData } = usePlans();
  const { data: notesData } = useNotes();
  const { data: searchData, isLoading: isSearching } = useSearch(debouncedQuery);

  const tasks = tasksData ?? [];
  const epics = epicsData ?? [];
  const plans = plansData ?? [];
  const notes = notesData ?? [];
  const searchResults = searchData?.data ?? [];

  const isSearchMode = debouncedQuery.length >= 2;

  // Debounce the query for server-side search
  useEffect(() => {
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => setDebouncedQuery(query), 300);
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [query]);

  // Reset query when closing
  useEffect(() => {
    if (!open) {
      setQuery("");
      setDebouncedQuery("");
    }
  }, [open]);

  // Cmd+K listener
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

  // Custom event listener for sidebar search trigger
  useEffect(() => {
    const handler = () => setOpen(true);
    window.addEventListener("open-command-palette", handler);
    return () => window.removeEventListener("open-command-palette", handler);
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

  function resultRoute(result: SearchResult) {
    const prefix = result.id.split("-")[0];
    const config = PREFIX_CONFIG[prefix];
    if (!config) return "/backlog";
    return `${config.route}?${result.entity_type === "task" ? "task" : result.entity_type}=${result.id}`;
  }

  return (
    <CommandDialog
      open={open}
      onOpenChange={setOpen}
      shouldFilter={!isSearchMode}
      className="sm:max-w-3xl"
    >
      <CommandInput
        placeholder="Search items, commands, or navigate..."
        value={query}
        onValueChange={setQuery}
      />
      <CommandList>
        {isSearchMode ? (
          <>
            {isSearching ? (
              <div className="py-6 text-center text-sm text-muted-foreground">
                Searching...
              </div>
            ) : searchResults.length === 0 ? (
              <CommandEmpty>No results found.</CommandEmpty>
            ) : (
              <CommandGroup
                heading={`${searchResults.length} result${searchResults.length !== 1 ? "s" : ""}`}
              >
                {searchResults.map((result) => {
                  const prefix = result.id.split("-")[0];
                  const entityColor =
                    PREFIX_CONFIG[prefix]?.color ?? "var(--entity-task)";
                  return (
                    <CommandItem
                      key={result.id}
                      value={result.id}
                      onSelect={() => navigate(resultRoute(result))}
                      className="!py-2.5"
                    >
                      <span
                        className="mr-2 font-mono text-xs shrink-0"
                        style={{ color: entityColor }}
                      >
                        {result.id}
                      </span>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="truncate font-medium">
                            {highlightMatch(result.title, debouncedQuery)}
                          </span>
                        </div>
                        {result.snippet && (
                          <p className="text-xs text-muted-foreground line-clamp-2 mt-0.5">
                            {highlightMatch(result.snippet, debouncedQuery)}
                          </p>
                        )}
                      </div>
                      <div className="flex items-center gap-2 ml-2 shrink-0">
                        {result.priority && (
                          <PriorityIndicator
                            priority={result.priority as Priority}
                          />
                        )}
                        <StatusBadge status={result.status as TaskStatus} />
                      </div>
                    </CommandItem>
                  );
                })}
              </CommandGroup>
            )}
          </>
        ) : (
          <>
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
                    onSelect={() => navigate(`/roadmap?epic=${epic.id}`)}
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
                <span className="ml-auto text-xs text-muted-foreground">
                  g d
                </span>
              </CommandItem>
              <CommandItem onSelect={() => navigate("/backlog")}>
                Go to Backlog
                <span className="ml-auto text-xs text-muted-foreground">
                  g b
                </span>
              </CommandItem>
              <CommandItem onSelect={() => navigate("/roadmap")}>
                Go to Roadmap
                <span className="ml-auto text-xs text-muted-foreground">
                  g r
                </span>
              </CommandItem>
              <CommandItem onSelect={() => navigate("/plans")}>
                Go to Plans
                <span className="ml-auto text-xs text-muted-foreground">
                  g p
                </span>
              </CommandItem>
              <CommandItem onSelect={() => navigate("/notes")}>
                Go to Notes
                <span className="ml-auto text-xs text-muted-foreground">
                  g n
                </span>
              </CommandItem>
              <CommandItem onSelect={() => navigate("/graph")}>
                Go to Graph
                <span className="ml-auto text-xs text-muted-foreground">
                  g g
                </span>
              </CommandItem>
            </CommandGroup>

            <CommandSeparator />

            <CommandGroup heading="Actions">
              <CommandItem onSelect={() => triggerSync()}>
                Sync project
                <span className="ml-auto text-xs text-muted-foreground">
                  ⌘S
                </span>
              </CommandItem>
            </CommandGroup>
          </>
        )}
      </CommandList>
    </CommandDialog>
  );
}
