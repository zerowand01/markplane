import { PRIORITY_CONFIG } from "@/lib/constants";
import type { Priority } from "@/lib/types";

const BARS = [
  { x: 0, y: 9, height: 3 },
  { x: 4, y: 6, height: 6 },
  { x: 8, y: 3, height: 9 },
  { x: 12, y: 0, height: 12 },
];

const FILLED_COUNT: Record<Priority, number> = {
  critical: 4,
  high: 3,
  medium: 2,
  low: 1,
  someday: 0,
};

export function PriorityIndicator({
  priority,
  showLabel = false,
}: {
  priority: Priority;
  showLabel?: boolean;
}) {
  const filled = FILLED_COUNT[priority];
  return (
    <span
      className="inline-flex items-center gap-1.5"
      style={{ color: `var(--priority-${priority})` }}
    >
      <svg
        width="14"
        height="12"
        viewBox="0 0 14 12"
        fill="currentColor"
        className={priority === "critical" ? "animate-pulse" : ""}
      >
        {BARS.map((bar, i) => (
          <rect
            key={i}
            x={bar.x}
            y={bar.y}
            width={2}
            height={bar.height}
            rx={0.5}
            opacity={i < filled ? 1 : 0.15}
          />
        ))}
      </svg>
      {showLabel && <span className="text-xs">{PRIORITY_CONFIG[priority].label}</span>}
    </span>
  );
}
