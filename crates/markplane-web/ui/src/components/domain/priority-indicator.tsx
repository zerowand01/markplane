import { PRIORITY_CONFIG } from "@/lib/constants";
import type { Priority } from "@/lib/types";

export function PriorityIndicator({
  priority,
  showLabel = false,
}: {
  priority: Priority;
  showLabel?: boolean;
}) {
  const config = PRIORITY_CONFIG[priority];
  return (
    <span
      className="inline-flex items-center gap-1 text-xs"
      style={{ color: `var(--priority-${priority})` }}
    >
      <span className={priority === "critical" ? "animate-pulse" : ""}>
        {config.icon}
      </span>
      {showLabel && <span>{config.label}</span>}
    </span>
  );
}
