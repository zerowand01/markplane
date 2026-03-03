"use client";

import { useEffect, useState, useCallback, useRef, type ReactNode } from "react";
import { useRouter } from "next/navigation";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/components/ui/command";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useTasks } from "@/lib/hooks/use-tasks";
import { useEpics } from "@/lib/hooks/use-epics";
import { usePlans } from "@/lib/hooks/use-plans";
import { useNotes } from "@/lib/hooks/use-notes";
import { useSearch } from "@/lib/hooks/use-search";
import { postAction } from "@/lib/api";
import { useQueryClient } from "@tanstack/react-query";
import { PREFIX_CONFIG } from "@/lib/constants";
import { GenericStatusBadge } from "@/components/domain/status-badge";
import { PriorityIndicator } from "@/components/domain/priority-indicator";
import { Archive } from "lucide-react";
import type { SearchResult, Priority } from "@/lib/types";

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

const NAVIGATION_ITEMS = [
  { label: "Go to Dashboard", path: "/dashboard", shortcut: "g d" },
  { label: "Go to Backlog", path: "/backlog", shortcut: "g b" },
  { label: "Go to Roadmap", path: "/roadmap", shortcut: "g r" },
  { label: "Go to Plans", path: "/plans", shortcut: "g p" },
  { label: "Go to Notes", path: "/notes", shortcut: "g n" },
  { label: "Go to Graph", path: "/graph", shortcut: "g g" },
  { label: "Go to Docs", path: "/docs", shortcut: "g ?" },
];

const CREATE_ITEMS = [
  { label: "New Task", kind: "task" },
  { label: "New Epic", kind: "epic" },
  { label: "New Note", kind: "note" },
  { label: "New Plan", kind: "plan" },
];

export function CommandPalette() {
  const [open, setOpen] = useState(false);

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

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogHeader className="sr-only">
        <DialogTitle>Command Palette</DialogTitle>
        <DialogDescription>Search for a command to run...</DialogDescription>
      </DialogHeader>
      <DialogContent className="overflow-hidden p-0 sm:max-w-3xl" showCloseButton={false}>
        {open && <CommandPaletteContent onClose={() => setOpen(false)} />}
      </DialogContent>
    </Dialog>
  );
}

function CommandPaletteContent({ onClose }: { onClose: () => void }) {
  const [query, setQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [includeArchived, setIncludeArchived] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const router = useRouter();
  const queryClient = useQueryClient();

  const { data: tasksData } = useTasks();
  const { data: epicsData } = useEpics();
  const { data: plansData } = usePlans();
  const { data: notesData } = useNotes();
  const { data: searchData, isLoading: isSearching } = useSearch(debouncedQuery, { includeArchived });

  const tasks = tasksData ?? [];
  const epics = epicsData ?? [];
  const plans = plansData ?? [];
  const notes = notesData ?? [];
  const searchResults = searchData?.data ?? [];

  const isSearchMode = debouncedQuery.length >= 2;

  // Client-side filtering for action items during search mode.
  const queryLower = query.toLowerCase();
  const filteredNavItems = isSearchMode
    ? NAVIGATION_ITEMS.filter((item) => item.label.toLowerCase().includes(queryLower))
    : NAVIGATION_ITEMS;
  const filteredCreateItems = isSearchMode
    ? CREATE_ITEMS.filter((item) => item.label.toLowerCase().includes(queryLower))
    : CREATE_ITEMS;
  const showSyncAction = !isSearchMode || "sync project".includes(queryLower);
  const hasActionMatches = filteredNavItems.length > 0 || filteredCreateItems.length > 0 || showSyncAction;
  // Show separator above action groups only when there's content above them
  const showActionSeparator = !isSearchMode || isSearching || searchResults.length > 0;

  // Debounce the query for server-side search
  useEffect(() => {
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => setDebouncedQuery(query), 300);
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [query]);

  const navigate = useCallback(
    (path: string) => {
      onClose();
      router.push(path);
    },
    [router, onClose]
  );

  const triggerSync = useCallback(async () => {
    onClose();
    await postAction("/api/sync");
    queryClient.invalidateQueries();
  }, [queryClient, onClose]);

  const createItem = useCallback((kind: string) => {
    onClose();
    window.dispatchEvent(new CustomEvent("create-item", { detail: { kind } }));
  }, [onClose]);

  function resultRoute(result: SearchResult) {
    const prefix = result.id.split("-")[0];
    const config = PREFIX_CONFIG[prefix];
    if (!config) return "/backlog";
    return `${config.route}?${result.entity_type === "task" ? "task" : result.entity_type}=${result.id}`;
  }

  return (
    <Command shouldFilter={!isSearchMode} className="[&_[cmdk-group-heading]]:text-muted-foreground **:data-[slot=command-input-wrapper]:h-12 [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group]]:px-2 [&_[cmdk-group]:not([hidden])_~[cmdk-group]]:pt-0 [&_[cmdk-input-wrapper]_svg]:h-5 [&_[cmdk-input-wrapper]_svg]:w-5 [&_[cmdk-input]]:h-12 [&_[cmdk-item]]:px-2 [&_[cmdk-item]]:py-3 [&_[cmdk-item]_svg]:h-5 [&_[cmdk-item]_svg]:w-5">
      <div className="relative">
        <CommandInput
          placeholder="Search items, commands, or navigate..."
          value={query}
          onValueChange={setQuery}
        />
        {isSearchMode && (
          <button
            type="button"
            onClick={() => setIncludeArchived((v) => !v)}
            className={`absolute right-12 top-1/2 -translate-y-1/2 inline-flex items-center gap-1.5 text-xs px-2 py-1 rounded-md transition-colors ${
              includeArchived
                ? "bg-primary/10 text-primary font-medium"
                : "text-muted-foreground hover:text-foreground hover:bg-muted"
            }`}
          >
            <Archive className="size-3.5" />
            Include archived
          </button>
        )}
      </div>
      <CommandList>
        {isSearchMode ? (
          <>
            {isSearching ? (
              <div className="py-6 text-center text-sm text-muted-foreground">
                Searching...
              </div>
            ) : searchResults.length > 0 ? (
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
                        {result.archived && (
                          <span className="text-[10px] px-1.5 py-0.5 rounded bg-muted text-muted-foreground font-medium">
                            Archived
                          </span>
                        )}
                        {result.priority && (
                          <PriorityIndicator
                            priority={result.priority as Priority}
                          />
                        )}
                        <GenericStatusBadge status={result.status} />
                      </div>
                    </CommandItem>
                  );
                })}
              </CommandGroup>
            ) : !hasActionMatches ? (
              <div className="py-6 text-center text-sm text-muted-foreground">
                No results found.
              </div>
            ) : null}
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
          </>
        )}

        {filteredNavItems.length > 0 && (
          <>
            {showActionSeparator && <CommandSeparator />}
            <CommandGroup heading="Navigation">
              {filteredNavItems.map((item) => (
                <CommandItem key={item.path} onSelect={() => navigate(item.path)}>
                  {item.label}
                  <span className="ml-auto text-xs text-muted-foreground">
                    {item.shortcut}
                  </span>
                </CommandItem>
              ))}
            </CommandGroup>
          </>
        )}

        {filteredCreateItems.length > 0 && (
          <>
            {(showActionSeparator || filteredNavItems.length > 0) && <CommandSeparator />}
            <CommandGroup heading="Create">
              {filteredCreateItems.map((item) => (
                <CommandItem key={item.kind} onSelect={() => createItem(item.kind)}>
                  {item.label}
                </CommandItem>
              ))}
            </CommandGroup>
          </>
        )}

        {showSyncAction && (
          <>
            {(showActionSeparator || filteredNavItems.length > 0 || filteredCreateItems.length > 0) && <CommandSeparator />}
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
    </Command>
  );
}
