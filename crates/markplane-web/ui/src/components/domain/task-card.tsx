"use client";

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { Archive } from "lucide-react";
import { PriorityIndicator } from "./priority-indicator";
import type { Task } from "@/lib/types";

export function TaskCard({
  task,
  onClick,
  onArchive,
}: {
  task: Task;
  onClick?: () => void;
  onArchive?: (id: string) => void;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: task.id, data: { task } });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const hasMetadata = task.epic || task.tags.length > 0;

  return (
    <div ref={setNodeRef} style={style} {...attributes} {...listeners}>
      <div
        className="rounded-lg border bg-card p-3 space-y-1 hover:border-muted-foreground/30 transition-colors cursor-pointer"
        onClick={onClick}
      >
        <div className="flex items-center gap-2">
          <PriorityIndicator priority={task.priority} />
          <span className="font-mono text-sm text-muted-foreground shrink-0">
            {task.id}
          </span>
          {onArchive && (
            <button
              title="Archive"
              className="size-6 flex items-center justify-center rounded opacity-0 group-hover/card:opacity-100 transition-opacity text-muted-foreground hover:text-primary hover:bg-primary/10 cursor-pointer"
              onClick={(e) => { e.stopPropagation(); onArchive(task.id); }}
              onPointerDown={(e) => e.stopPropagation()}
            >
              <Archive className="size-3.5" />
            </button>
          )}
          {task.effort && (
            <span className="ml-auto text-sm font-medium px-2 py-0.5 rounded bg-secondary text-secondary-foreground uppercase">
              {task.effort === "xs"
                ? "XS"
                : task.effort === "xl"
                  ? "XL"
                  : task.effort.charAt(0).toUpperCase()}
            </span>
          )}
        </div>
        <p className="text-base font-medium leading-snug line-clamp-2">
          {task.title}
        </p>
        {hasMetadata && (
          <div className="flex items-center gap-2 flex-wrap">
            {task.epic && (
              <span
                className="text-sm font-mono px-2 py-0.5 rounded"
                style={{
                  backgroundColor:
                    "color-mix(in oklch, var(--entity-epic) 15%, transparent)",
                  color: "var(--entity-epic)",
                }}
              >
                {task.epic}
              </span>
            )}
            {task.tags.map((tag) => (
              <span
                key={tag}
                className="text-sm text-muted-foreground"
              >
                #{tag}
              </span>
            ))}
          </div>
        )}
        {task.assignee && (
          <div className="text-xs text-muted-foreground">
            @{task.assignee}
          </div>
        )}
      </div>
    </div>
  );
}
