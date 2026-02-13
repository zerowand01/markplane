"use client";

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { PriorityIndicator } from "./priority-indicator";
import { Card, CardContent } from "@/components/ui/card";
import type { Task } from "@/lib/types";

export function TaskCard({
  task,
  onClick,
}: {
  task: Task;
  onClick?: () => void;
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

  return (
    <div ref={setNodeRef} style={style} {...attributes} {...listeners}>
      <Card
        className="hover:border-muted-foreground/30 transition-colors cursor-pointer"
        onClick={onClick}
      >
        <CardContent className="p-3 space-y-2">
          <div className="flex items-center gap-2">
            <PriorityIndicator priority={task.priority} />
            <span className="font-mono text-xs text-muted-foreground">
              {task.id}
            </span>
            {task.effort && (
              <span className="ml-auto text-[10px] font-medium px-1.5 py-0.5 rounded bg-secondary text-secondary-foreground uppercase">
                {task.effort === "xs"
                  ? "XS"
                  : task.effort === "xl"
                    ? "XL"
                    : task.effort.charAt(0).toUpperCase()}
              </span>
            )}
          </div>
          <p className="text-sm font-medium leading-snug line-clamp-2">
            {task.title}
          </p>
          <div className="flex items-center gap-2 flex-wrap">
            {task.epic && (
              <span
                className="text-[10px] font-mono px-1.5 py-0.5 rounded"
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
                className="text-[10px] text-muted-foreground"
              >
                #{tag}
              </span>
            ))}
          </div>
          {task.assignee && (
            <div className="text-xs text-muted-foreground">
              @{task.assignee}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
