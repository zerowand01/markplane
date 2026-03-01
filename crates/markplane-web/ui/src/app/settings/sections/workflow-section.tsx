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
  verticalListSortingStrategy,
  arrayMove,
  sortableKeyboardCoordinates,
} from "@dnd-kit/sortable";
import { Plus } from "lucide-react";
import { useConfig } from "@/lib/hooks/use-config";
import { useUpdateConfig } from "@/lib/hooks/use-mutations";
import { CATEGORY_CONFIG } from "@/lib/constants";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { PageTransition } from "@/components/domain/page-transition";
import { SortableItem } from "./type-list-editor";
import type { StatusCategory, TaskWorkflow } from "@/lib/types";

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
    <Card>
      <CardHeader>
        <CardTitle>Task Workflow</CardTitle>
        <CardDescription>
          Configure which status strings map to each category. Categories are fixed; statuses within them are yours to define.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
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

export function WorkflowSection() {
  const { data: config, isLoading } = useConfig();
  const updateConfig = useUpdateConfig();

  if (isLoading || !config) {
    return (
      <Card>
        <CardContent className="p-6">
          <div className="h-96 animate-pulse rounded bg-muted" />
        </CardContent>
      </Card>
    );
  }

  return (
    <PageTransition>
      <WorkflowEditor
        workflow={config.workflows.task}
        onUpdate={(task) => updateConfig.mutate({ workflows: { task } })}
        isPending={updateConfig.isPending}
      />
    </PageTransition>
  );
}
