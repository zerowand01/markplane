import { Circle } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { STATUS_CONFIG, EPIC_STATUS_CONFIG, PLAN_STATUS_CONFIG, NOTE_STATUS_CONFIG, CATEGORY_CONFIG, categoryOf } from "@/lib/constants";
import { useConfig } from "@/lib/hooks/use-config";
import type { EpicStatus } from "@/lib/types";

export function StatusBadge({ status }: { status: string }) {
  const { data: config } = useConfig();
  const workflow = config?.workflows.task;

  // Try known status config first, then fall back to category icon
  const known = STATUS_CONFIG[status];
  const category = workflow ? categoryOf(workflow, status) : undefined;
  const catConfig = category ? CATEGORY_CONFIG[category] : undefined;
  const Icon = known?.icon ?? catConfig?.icon ?? Circle;
  const label = known?.label ?? status.split("-").map(w => w.length > 0 ? w[0].toUpperCase() + w.slice(1) : "").join(" ");
  // Use category CSS variable as fallback for custom statuses
  const cssVar = `var(--status-${status}, ${category ? `var(--status-category-${category})` : "var(--muted-foreground)"})`;

  return (
    <Badge
      variant="outline"
      className="gap-1 border-transparent font-medium transition-colors duration-300"
      style={{
        backgroundColor: `color-mix(in oklch, ${cssVar} 15%, transparent)`,
        color: cssVar,
      }}
    >
      <Icon className="size-3.5 text-current" />
      {label}
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
