"use client";

import { useState, useMemo, useCallback } from "react";
import { useArchivedTasks } from "@/lib/hooks/use-tasks";
import { useArchivedEpics } from "@/lib/hooks/use-epics";
import { useArchivedPlans } from "@/lib/hooks/use-plans";
import { useArchivedNotes } from "@/lib/hooks/use-notes";
import { useSearchParams, useRouter } from "next/navigation";
import { useUnarchiveItem } from "@/lib/hooks/use-mutations";
import { TaskDetailSheet } from "@/components/domain/task-detail-sheet";
import { EpicDetailSheet } from "@/components/domain/epic-detail-sheet";
import { PlanDetailSheet } from "@/components/domain/plan-detail-sheet";
import { NoteDetailSheet } from "@/components/domain/note-detail-sheet";
import { StatusBadge, EpicStatusBadge } from "@/components/domain/status-badge";
import { PriorityIndicator } from "@/components/domain/priority-indicator";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";
import { ArchiveRestore, Search, X } from "lucide-react";
import type { Task, Epic, Plan, Note } from "@/lib/types";

type EntityTab = "all" | "tasks" | "epics" | "plans" | "notes";

interface ArchivedItem {
  id: string;
  title: string;
  type: EntityTab;
  status: string;
  priority?: string;
  updated: string;
}

function toArchivedItems(
  tasks: Task[],
  epics: Epic[],
  plans: Plan[],
  notes: Note[],
): ArchivedItem[] {
  const items: ArchivedItem[] = [
    ...tasks.map((t) => ({
      id: t.id,
      title: t.title,
      type: "tasks" as const,
      status: t.status,
      priority: t.priority,
      updated: t.updated,
    })),
    ...epics.map((e) => ({
      id: e.id,
      title: e.title,
      type: "epics" as const,
      status: e.status,
      priority: e.priority,
      updated: "",
    })),
    ...plans.map((p) => ({
      id: p.id,
      title: p.title,
      type: "plans" as const,
      status: p.status,
      updated: p.updated,
    })),
    ...notes.map((n) => ({
      id: n.id,
      title: n.title,
      type: "notes" as const,
      status: n.status,
      updated: n.updated,
    })),
  ];
  // Sort by updated date descending (most recent first)
  items.sort((a, b) => (b.updated || "").localeCompare(a.updated || ""));
  return items;
}

const TYPE_LABEL: Record<string, string> = {
  tasks: "Task",
  epics: "Epic",
  plans: "Plan",
  notes: "Note",
};

const TYPE_COLOR: Record<string, string> = {
  tasks: "var(--entity-task)",
  epics: "var(--entity-epic)",
  plans: "var(--entity-plan)",
  notes: "var(--entity-note)",
};

const PARAM_KEY: Record<string, string> = {
  tasks: "task",
  epics: "epic",
  plans: "plan",
  notes: "note",
};

export function ArchiveContent() {
  const searchParams = useSearchParams();
  const router = useRouter();

  const [tab, setTab] = useState<EntityTab>("all");
  const [searchQuery, setSearchQuery] = useState("");

  const selectedTaskId = searchParams.get("task");
  const selectedEpicId = searchParams.get("epic");
  const selectedPlanId = searchParams.get("plan");
  const selectedNoteId = searchParams.get("note");
  const [secondaryTaskId, setSecondaryTaskId] = useState<string | null>(null);

  const openItem = useCallback(
    (id: string, type: string) => {
      const params = new URLSearchParams(searchParams.toString());
      for (const key of Object.values(PARAM_KEY)) {
        params.delete(key);
      }
      const paramKey = PARAM_KEY[type];
      if (paramKey) params.set(paramKey, id);
      router.replace(`?${params.toString()}`, { scroll: false });
    },
    [router, searchParams],
  );

  const closeItem = useCallback(() => {
    const params = new URLSearchParams(searchParams.toString());
    for (const key of Object.values(PARAM_KEY)) {
      params.delete(key);
    }
    const qs = params.toString();
    router.replace(qs ? `?${qs}` : "/archive/", { scroll: false });
  }, [router, searchParams]);

  const { data: tasks = [], isLoading: loadingTasks } = useArchivedTasks();
  const { data: epics = [], isLoading: loadingEpics } = useArchivedEpics();
  const { data: plans = [], isLoading: loadingPlans } = useArchivedPlans();
  const { data: notes = [], isLoading: loadingNotes } = useArchivedNotes();
  const unarchive = useUnarchiveItem();

  const isLoading = loadingTasks || loadingEpics || loadingPlans || loadingNotes;

  const allItems = useMemo(
    () => toArchivedItems(tasks, epics, plans, notes),
    [tasks, epics, plans, notes],
  );

  const filteredItems = useMemo(() => {
    let items = tab === "all" ? allItems : allItems.filter((i) => i.type === tab);
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      items = items.filter(
        (i) => i.title.toLowerCase().includes(q) || i.id.toLowerCase().includes(q),
      );
    }
    return items;
  }, [allItems, tab, searchQuery]);

  const counts = useMemo(
    () => ({
      all: allItems.length,
      tasks: tasks.length,
      epics: epics.length,
      plans: plans.length,
      notes: notes.length,
    }),
    [allItems, tasks, epics, plans, notes],
  );

  const tabs: { key: EntityTab; label: string }[] = [
    { key: "all", label: "All" },
    { key: "tasks", label: "Tasks" },
    { key: "epics", label: "Epics" },
    { key: "plans", label: "Plans" },
    { key: "notes", label: "Notes" },
  ];

  if (isLoading) {
    return <ArchiveSkeleton />;
  }

  return (
    <PageTransition>
      <div className="space-y-4">
        {/* Header */}
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-semibold">Archive</h1>
          <span className="text-sm text-muted-foreground">
            ({filteredItems.length} {filteredItems.length === 1 ? "item" : "items"})
          </span>
        </div>

        {/* Tabs + Search */}
        <div className="flex items-center gap-4 border-b">
          {tabs.map((t) => (
            <button
              key={t.key}
              className={`text-sm pb-2 -mb-px transition-colors ${
                tab === t.key
                  ? "text-primary border-b-2 border-primary font-semibold"
                  : "text-muted-foreground hover:text-foreground"
              }`}
              onClick={() => setTab(t.key)}
            >
              {t.label}
              {counts[t.key] > 0 && (
                <span className="ml-1.5 text-xs text-muted-foreground">
                  {counts[t.key]}
                </span>
              )}
            </button>
          ))}
          <div className="relative ml-auto w-56 shrink-0 mb-1.5">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
            <input
              type="text"
              placeholder="Filter by title or ID..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full rounded-md border bg-background pl-8 pr-8 py-1 text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            />
            {searchQuery && (
              <button
                type="button"
                onClick={() => setSearchQuery("")}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                <X className="size-3.5" />
              </button>
            )}
          </div>
        </div>

        {/* Items */}
        {filteredItems.length === 0 ? (
          allItems.length === 0 ? (
            <EmptyState
              title="Archive is empty"
              description="Completed items will appear here when archived"
            />
          ) : (
            <EmptyState
              title="No matches"
              description="No archived items match your search"
            />
          )
        ) : (
          <div className="space-y-1">
            {filteredItems.map((item) => (
              <div
                key={item.id}
                className="group/row flex items-center gap-3 rounded-md border bg-card px-3 py-2.5 transition-colors hover:border-muted-foreground/30 cursor-pointer"
                role="button"
                tabIndex={0}
                onClick={() => openItem(item.id, item.type)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    openItem(item.id, item.type);
                  }
                }}
              >
                {/* Type badge */}
                <span
                  className="text-xs font-mono px-1.5 py-0.5 rounded shrink-0"
                  style={{
                    backgroundColor: `color-mix(in oklch, ${TYPE_COLOR[item.type]} 15%, transparent)`,
                    color: TYPE_COLOR[item.type],
                  }}
                >
                  {TYPE_LABEL[item.type]}
                </span>

                {/* ID */}
                <span className="font-mono text-sm text-muted-foreground w-24 shrink-0">
                  {item.id}
                </span>

                {/* Title */}
                <span className="text-base font-medium truncate flex-1">
                  {item.title}
                </span>

                {/* Status */}
                {item.type === "epics" ? (
                  <EpicStatusBadge status={item.status as "now" | "next" | "later" | "done"} />
                ) : item.type === "tasks" ? (
                  <StatusBadge status={item.status as "done" | "cancelled" | "draft" | "backlog" | "planned" | "in-progress"} />
                ) : (
                  <span
                    className="text-xs px-2 py-0.5 rounded shrink-0"
                    style={{
                      backgroundColor: `color-mix(in oklch, var(--status-${item.status}) 15%, transparent)`,
                      color: `var(--status-${item.status})`,
                    }}
                  >
                    {item.status}
                  </span>
                )}

                {/* Priority */}
                {item.priority && (
                  <PriorityIndicator priority={item.priority as "critical" | "high" | "medium" | "low" | "someday"} />
                )}

                {/* Updated date */}
                {item.updated && (
                  <span className="text-xs text-muted-foreground w-24 text-right shrink-0 hidden md:block">
                    {item.updated}
                  </span>
                )}

                {/* Restore button */}
                <Button
                  variant="ghost"
                  size="sm"
                  className="opacity-0 group-hover/row:opacity-100 transition-opacity shrink-0 h-7 px-2 gap-1"
                  onClick={(e) => {
                    e.stopPropagation();
                    unarchive.mutate(item.id);
                  }}
                  disabled={unarchive.isPending}
                >
                  <ArchiveRestore className="size-3.5" />
                  <span className="text-xs">Restore</span>
                </Button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Detail sheets */}
      <TaskDetailSheet
        taskId={selectedTaskId}
        open={!!selectedTaskId}
        onOpenChange={(open) => {
          if (!open) closeItem();
        }}
        archived
      />
      <EpicDetailSheet
        epicId={selectedEpicId}
        open={!!selectedEpicId}
        onOpenChange={(open) => {
          if (!open) closeItem();
        }}
        onTaskClick={(id) => setSecondaryTaskId(id)}
        archived
      />
      <PlanDetailSheet
        planId={selectedPlanId}
        open={!!selectedPlanId}
        onOpenChange={(open) => {
          if (!open) closeItem();
        }}
        archived
      />
      <NoteDetailSheet
        noteId={selectedNoteId}
        open={!!selectedNoteId}
        onOpenChange={(open) => {
          if (!open) closeItem();
        }}
        archived
      />

      {/* Secondary task sheet for linked tasks from epic detail */}
      <TaskDetailSheet
        taskId={secondaryTaskId}
        open={!!secondaryTaskId}
        onOpenChange={(open) => {
          if (!open) setSecondaryTaskId(null);
        }}
      />
    </PageTransition>
  );
}

export function ArchiveSkeleton() {
  return (
    <div className="space-y-3">
      <Skeleton className="h-8 w-64" />
      {Array.from({ length: 5 }).map((_, i) => (
        <Skeleton key={i} className="h-14 w-full" />
      ))}
    </div>
  );
}
