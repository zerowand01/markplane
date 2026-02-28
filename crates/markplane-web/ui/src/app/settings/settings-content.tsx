"use client";

import { useState, useCallback } from "react";
import {
  DndContext,
  closestCenter,
  PointerSensor,
  KeyboardSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import type { DragEndEvent } from "@dnd-kit/core";
import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
  arrayMove,
  sortableKeyboardCoordinates,
  defaultAnimateLayoutChanges,
} from "@dnd-kit/sortable";
import type { AnimateLayoutChanges } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVertical, Plus, X } from "lucide-react";
import { useConfig } from "@/lib/hooks/use-config";
import { useUpdateConfig } from "@/lib/hooks/use-mutations";
import { CATEGORY_CONFIG } from "@/lib/constants";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { PageTransition } from "@/components/domain/page-transition";
import type { StatusCategory, TaskWorkflow } from "@/lib/types";

// Don't animate the item that was just dropped — prevents the "jump back" effect
const skipDropAnimation: AnimateLayoutChanges = (args) => {
  if (args.wasDragging) return false;
  return defaultAnimateLayoutChanges(args);
};

function SortableItem({
  id,
  onRemove,
  canRemove,
}: {
  id: string;
  onRemove: () => void;
  canRemove: boolean;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id, animateLayoutChanges: skipDropAnimation });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="flex items-center gap-2 rounded-md border bg-card px-3 py-2"
    >
      <button
        {...attributes}
        {...listeners}
        className="cursor-grab text-muted-foreground hover:text-foreground touch-none"
        tabIndex={-1}
      >
        <GripVertical className="size-4" />
      </button>
      <span className="flex-1 text-sm">{id}</span>
      <button
        onClick={onRemove}
        disabled={!canRemove}
        className="text-muted-foreground hover:text-destructive disabled:opacity-30 disabled:cursor-not-allowed"
        title={canRemove ? `Remove "${id}"` : "Cannot remove the last status"}
      >
        <X className="size-4" />
      </button>
    </div>
  );
}

function TypeListEditor({
  title,
  description,
  items,
  onUpdate,
  isPending,
}: {
  title: string;
  description: string;
  items: string[];
  onUpdate: (items: string[]) => void;
  isPending: boolean;
}) {
  const [newValue, setNewValue] = useState("");
  const [error, setError] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  const handleAdd = useCallback(() => {
    const trimmed = newValue.trim().toLowerCase();
    if (!trimmed) {
      setError("Type name cannot be empty");
      return;
    }
    if (items.includes(trimmed)) {
      setError(`"${trimmed}" already exists`);
      return;
    }
    setError(null);
    setNewValue("");
    onUpdate([...items, trimmed]);
  }, [newValue, items, onUpdate]);

  const handleRemove = useCallback(
    (index: number) => {
      const next = items.filter((_, i) => i !== index);
      onUpdate(next);
    },
    [items, onUpdate],
  );

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event;
      if (over && active.id !== over.id) {
        const oldIndex = items.indexOf(active.id as string);
        const newIndex = items.indexOf(over.id as string);
        onUpdate(arrayMove(items, oldIndex, newIndex));
      }
    },
    [items, onUpdate],
  );

  return (
    <Card>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragEnd={handleDragEnd}
        >
          <SortableContext items={items} strategy={verticalListSortingStrategy}>
            <div className="space-y-1.5">
              {items.map((item, index) => (
                <SortableItem
                  key={item}
                  id={item}
                  onRemove={() => handleRemove(index)}
                  canRemove={items.length > 1}
                />
              ))}
            </div>
          </SortableContext>
        </DndContext>

        <div className="flex items-start gap-2 pt-1">
          <div className="flex-1 space-y-1">
            <Input
              placeholder="Add new type..."
              value={newValue}
              onChange={(e) => {
                setNewValue(e.target.value);
                setError(null);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  handleAdd();
                }
              }}
              disabled={isPending}
            />
            {error && <p className="text-xs text-destructive">{error}</p>}
          </div>
          <Button
            variant="outline"
            size="icon"
            onClick={handleAdd}
            disabled={isPending || !newValue.trim()}
          >
            <Plus className="size-4" />
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

// ── Workflow Editor ──────────────────────────────────────────────────────

const CATEGORY_ORDER: StatusCategory[] = [
  "draft", "backlog", "planned", "active", "completed", "cancelled",
];

const CATEGORY_DESCRIPTIONS: Record<StatusCategory, string> = {
  draft: "Unrefined ideas, not yet ready for work",
  backlog: "Ready for work but not yet scheduled",
  planned: "Scheduled for upcoming work",
  active: "Currently being worked on",
  completed: "Successfully finished",
  cancelled: "Abandoned or no longer relevant",
};

function CategoryBucket({
  category,
  statuses,
  allStatuses,
  onUpdate,
  isPending,
}: {
  category: StatusCategory;
  statuses: string[];
  allStatuses: Set<string>;
  onUpdate: (statuses: string[]) => void;
  isPending: boolean;
}) {
  const [newValue, setNewValue] = useState("");
  const [error, setError] = useState<string | null>(null);
  const catConfig = CATEGORY_CONFIG[category];
  const Icon = catConfig.icon;

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  const handleAdd = useCallback(() => {
    const trimmed = newValue.trim().toLowerCase();
    if (!trimmed) {
      setError("Status name cannot be empty");
      return;
    }
    if (allStatuses.has(trimmed)) {
      setError(`"${trimmed}" already exists`);
      return;
    }
    setError(null);
    setNewValue("");
    onUpdate([...statuses, trimmed]);
  }, [newValue, statuses, allStatuses, onUpdate]);

  const handleRemove = useCallback(
    (index: number) => {
      onUpdate(statuses.filter((_, i) => i !== index));
    },
    [statuses, onUpdate],
  );

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event;
      if (over && active.id !== over.id) {
        const oldIndex = statuses.indexOf(active.id as string);
        const newIndex = statuses.indexOf(over.id as string);
        onUpdate(arrayMove(statuses, oldIndex, newIndex));
      }
    },
    [statuses, onUpdate],
  );

  return (
    <div className="rounded-lg border p-3 space-y-2">
      <div className="flex items-center gap-2">
        <Icon className="size-4" style={{ color: `var(--status-category-${category})` }} />
        <span className="text-sm font-medium">{catConfig.label}</span>
        <span className="text-xs text-muted-foreground">{CATEGORY_DESCRIPTIONS[category]}</span>
      </div>

      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
      >
        <SortableContext items={statuses} strategy={verticalListSortingStrategy}>
          <div className="space-y-1">
            {statuses.map((status, index) => (
              <SortableItem
                key={status}
                id={status}
                onRemove={() => handleRemove(index)}
                canRemove={statuses.length > 1}
              />
            ))}
          </div>
        </SortableContext>
      </DndContext>

      <div className="flex items-start gap-2">
        <div className="flex-1 space-y-1">
          <Input
            placeholder="Add status..."
            value={newValue}
            onChange={(e) => {
              setNewValue(e.target.value);
              setError(null);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                handleAdd();
              }
            }}
            disabled={isPending}
            className="h-8 text-sm"
          />
          {error && <p className="text-xs text-destructive">{error}</p>}
        </div>
        <Button
          variant="outline"
          size="icon"
          className="size-8"
          onClick={handleAdd}
          disabled={isPending || !newValue.trim()}
        >
          <Plus className="size-3.5" />
        </Button>
      </div>
    </div>
  );
}

function WorkflowEditor({
  workflow,
  onUpdate,
  isPending,
}: {
  workflow: TaskWorkflow;
  onUpdate: (workflow: TaskWorkflow) => void;
  isPending: boolean;
}) {
  // Collect all status strings across categories for duplicate detection
  const allStatuses = new Set(
    CATEGORY_ORDER.flatMap((cat) => workflow[cat] ?? [])
  );

  const handleCategoryUpdate = useCallback(
    (category: StatusCategory, statuses: string[]) => {
      onUpdate({ ...workflow, [category]: statuses });
    },
    [workflow, onUpdate],
  );

  return (
    <Card className="md:col-span-2">
      <CardHeader>
        <CardTitle>Task Workflow</CardTitle>
        <CardDescription>
          Configure which status strings map to each category. Categories are fixed; statuses within them are yours to define.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid gap-3 md:grid-cols-2">
          {CATEGORY_ORDER.map((category) => (
            <CategoryBucket
              key={category}
              category={category}
              statuses={workflow[category] ?? []}
              allStatuses={allStatuses}
              onUpdate={(statuses) => handleCategoryUpdate(category, statuses)}
              isPending={isPending}
            />
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

// ── Main Settings ────────────────────────────────────────────────────────

export function SettingsContent() {
  const { data: config, isLoading } = useConfig();
  const updateConfig = useUpdateConfig();

  if (isLoading || !config) {
    return (
      <div className="space-y-6">
        <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
        <div className="grid gap-6 md:grid-cols-2">
          <Card><CardContent className="p-6"><div className="h-48 animate-pulse rounded bg-muted" /></CardContent></Card>
          <Card><CardContent className="p-6"><div className="h-48 animate-pulse rounded bg-muted" /></CardContent></Card>
        </div>
      </div>
    );
  }

  return (
    <PageTransition>
      <div className="space-y-6">
        <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
        <div className="grid gap-6 md:grid-cols-2">
          <TypeListEditor
            title="Task Types"
            description="First item is the default for new tasks"
            items={config.task_types}
            onUpdate={(task_types) => updateConfig.mutate({ task_types })}
            isPending={updateConfig.isPending}
          />
          <TypeListEditor
            title="Note Types"
            description="First item is the default for new notes"
            items={config.note_types}
            onUpdate={(note_types) => updateConfig.mutate({ note_types })}
            isPending={updateConfig.isPending}
          />
          <WorkflowEditor
            workflow={config.workflows.task}
            onUpdate={(task) =>
              updateConfig.mutate({ workflows: { task } })
            }
            isPending={updateConfig.isPending}
          />
        </div>
      </div>
    </PageTransition>
  );
}
