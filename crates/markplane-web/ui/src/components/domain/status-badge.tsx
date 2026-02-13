import { Badge } from "@/components/ui/badge";
import { STATUS_CONFIG, EPIC_STATUS_CONFIG } from "@/lib/constants";
import type { TaskStatus, EpicStatus } from "@/lib/types";

export function StatusBadge({ status }: { status: TaskStatus }) {
  const config = STATUS_CONFIG[status];
  return (
    <Badge
      variant="outline"
      className="gap-1 border-transparent font-medium transition-colors duration-300"
      style={{
        backgroundColor: `color-mix(in oklch, var(--status-${status}) 15%, transparent)`,
        color: `var(--status-${status})`,
      }}
    >
      <span>{config.icon}</span>
      {config.label}
    </Badge>
  );
}

export function EpicStatusBadge({ status }: { status: EpicStatus }) {
  const config = EPIC_STATUS_CONFIG[status];
  return (
    <Badge
      variant="outline"
      className="gap-1 border-transparent font-medium transition-colors duration-300"
      style={{
        backgroundColor: `color-mix(in oklch, var(--status-${status}) 15%, transparent)`,
        color: `var(--status-${status})`,
      }}
    >
      <span>{config.icon}</span>
      {config.label}
    </Badge>
  );
}
