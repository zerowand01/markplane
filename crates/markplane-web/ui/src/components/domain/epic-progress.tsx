import { Progress } from "@/components/ui/progress";
import type { Epic } from "@/lib/types";

export function EpicProgress({
  epic,
  showHeader = true,
}: {
  epic: Epic;
  showHeader?: boolean;
}) {
  const percent = Math.round(epic.progress * 100);

  return (
    <div className="space-y-1.5">
      {showHeader && (
        <div className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-2">
            <span
              className="font-mono text-xs"
              style={{ color: "var(--entity-epic)" }}
            >
              {epic.id}
            </span>
            <span className="font-medium">{epic.title}</span>
          </div>
        </div>
      )}
      <div className="flex items-center gap-2">
        <Progress value={percent} className="h-1.5 flex-1" />
        <span className="text-xs text-muted-foreground shrink-0">
          {epic.done_count}/{epic.task_count} tasks &middot; {percent}%
        </span>
      </div>
    </div>
  );
}
