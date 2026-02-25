import { Badge } from "@/components/ui/badge";
import { STATUS_CONFIG, EPIC_STATUS_CONFIG, PLAN_STATUS_CONFIG, NOTE_STATUS_CONFIG } from "@/lib/constants";
import type { TaskStatus, EpicStatus } from "@/lib/types";

export function StatusBadge({ status }: { status: TaskStatus }) {
  const config = STATUS_CONFIG[status];
  const Icon = config.icon;
  return (
    <Badge
      variant="outline"
      className="gap-1 border-transparent font-medium transition-colors duration-300"
      style={{
        backgroundColor: `color-mix(in oklch, var(--status-${status}) 15%, transparent)`,
        color: `var(--status-${status})`,
      }}
    >
      <Icon className="size-3.5 text-current" />
      {config.label}
    </Badge>
  );
}

export function EpicStatusBadge({ status }: { status: EpicStatus }) {
  const config = EPIC_STATUS_CONFIG[status];
  const Icon = config.icon;
  return (
    <Badge
      variant="outline"
      className="gap-1 border-transparent font-medium transition-colors duration-300"
      style={{
        backgroundColor: `color-mix(in oklch, var(--status-${status}) 15%, transparent)`,
        color: `var(--status-${status})`,
      }}
    >
      <Icon className="size-3.5 text-current" />
      {config.label}
    </Badge>
  );
}

const ALL_STATUS_CONFIGS: Record<string, { label: string; icon: import("lucide-react").LucideIcon }> = {
  ...STATUS_CONFIG,
  ...EPIC_STATUS_CONFIG,
  ...PLAN_STATUS_CONFIG,
  ...NOTE_STATUS_CONFIG,
};

export function GenericStatusBadge({ status }: { status: string }) {
  const config = ALL_STATUS_CONFIGS[status];
  if (!config) {
    return (
      <Badge variant="outline" className="gap-1 border-transparent font-medium">
        {status}
      </Badge>
    );
  }
  const Icon = config.icon;
  return (
    <Badge
      variant="outline"
      className="gap-1 border-transparent font-medium transition-colors duration-300"
      style={{
        backgroundColor: `color-mix(in oklch, var(--status-${status}) 15%, transparent)`,
        color: `var(--status-${status})`,
      }}
    >
      <Icon className="size-3.5 text-current" />
      {config.label}
    </Badge>
  );
}
