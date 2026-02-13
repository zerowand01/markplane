import { Progress } from "@/components/ui/progress";
import type { Epic } from "@/lib/types";

export function EpicProgress({ epic }: { epic: Epic }) {
  const percent = Math.round(epic.progress * 100);

  return (
    <div className="space-y-1.5">
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
        <span className="text-xs text-muted-foreground">
          {epic.done_count}/{epic.task_count} tasks &middot; {percent}%
        </span>
      </div>
      <Progress value={percent} className="h-1.5" />
    </div>
  );
}
