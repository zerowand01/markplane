"use client";

import { Suspense, useState, useMemo } from "react";
import { useArchivedTasks } from "@/lib/hooks/use-tasks";
import { useArchivedEpics } from "@/lib/hooks/use-epics";
import { useArchivedPlans } from "@/lib/hooks/use-plans";
import { useArchivedNotes } from "@/lib/hooks/use-notes";
import { useUnarchiveItem } from "@/lib/hooks/use-mutations";
import { StatusBadge, EpicStatusBadge } from "@/components/domain/status-badge";
import { PriorityIndicator } from "@/components/domain/priority-indicator";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";
import { ArchiveRestore } from "lucide-react";
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

function ArchiveContent() {
  const [tab, setTab] = useState<EntityTab>("all");

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

  const filteredItems = useMemo(
    () => (tab === "all" ? allItems : allItems.filter((i) => i.type === tab)),
    [allItems, tab],
  );

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
    return (
      <div className="space-y-3">
        <Skeleton className="h-8 w-64" />
        {Array.from({ length: 5 }).map((_, i) => (
          <Skeleton key={i} className="h-14 w-full" />
        ))}
      </div>
    );
  }

  return (
    <PageTransition>
      <div className="space-y-4">
        {/* Header */}
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-semibold">Archive</h1>
          <span className="text-sm text-muted-foreground">
            ({counts.all} {counts.all === 1 ? "item" : "items"})
          </span>
        </div>

        {/* Tabs */}
        <div className="flex gap-4 border-b">
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
        </div>

        {/* Items */}
        {filteredItems.length === 0 ? (
          <EmptyState
            title="Archive is empty"
            description="Completed items will appear here when archived"
          />
        ) : (
          <div className="space-y-1">
            {filteredItems.map((item) => (
              <div
                key={item.id}
                className="group/row flex items-center gap-3 rounded-md border bg-card px-3 py-2.5 transition-colors hover:border-muted-foreground/30"
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
                  <EpicStatusBadge status={item.status as "planned" | "active" | "done"} />
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
                  onClick={() => unarchive.mutate(item.id)}
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
    </PageTransition>
  );
}

export default function ArchivePage() {
  return (
    <div className="p-4 md:p-6">
      <Suspense
        fallback={
          <div className="space-y-3">
            <Skeleton className="h-8 w-64" />
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-14 w-full" />
            ))}
          </div>
        }
      >
        <ArchiveContent />
      </Suspense>
    </div>
  );
}
