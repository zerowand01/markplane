import Link from "next/link";
import { X } from "lucide-react";

const PREFIX_CONFIG: Record<string, { route: string; param: string; cssVar: string }> = {
  TASK: { route: "/backlog", param: "task", cssVar: "--entity-task" },
  EPIC: { route: "/roadmap", param: "epic", cssVar: "--entity-epic" },
  PLAN: { route: "/plans", param: "plan", cssVar: "--entity-plan" },
  NOTE: { route: "/notes", param: "note", cssVar: "--entity-note" },
};

export function WikiLinkChip({
  id,
  onRemove,
}: {
  id: string;
  onRemove?: () => void;
}) {
  const prefix = id.split("-")[0];
  const config = PREFIX_CONFIG[prefix] || PREFIX_CONFIG.TASK;
  const href = `${config.route}?${config.param}=${id}`;

  const chipStyle = {
    backgroundColor: `color-mix(in oklch, var(${config.cssVar}) 15%, transparent)`,
    color: `var(${config.cssVar})`,
  };

  if (onRemove) {
    return (
      <span
        className="inline-flex items-center gap-1 font-mono text-xs px-1.5 py-0.5 rounded"
        style={chipStyle}
      >
        <Link
          href={href}
          className="no-underline hover:opacity-80 transition-opacity"
          style={{ color: "inherit" }}
        >
          {id}
        </Link>
        <button
          type="button"
          onClick={onRemove}
          className="opacity-40 hover:opacity-100 hover:text-destructive transition-opacity cursor-pointer"
        >
          <X className="size-3" />
        </button>
      </span>
    );
  }

  return (
    <Link
      href={href}
      className="inline-flex items-center font-mono text-xs px-1.5 py-0.5 rounded no-underline hover:opacity-80 transition-opacity"
      style={chipStyle}
    >
      {id}
    </Link>
  );
}
