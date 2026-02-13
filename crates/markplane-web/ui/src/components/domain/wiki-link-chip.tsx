import Link from "next/link";

const PREFIX_CONFIG: Record<string, { route: string; param: string; cssVar: string }> = {
  TASK: { route: "/backlog", param: "task", cssVar: "--entity-task" },
  EPIC: { route: "/epics", param: "epic", cssVar: "--entity-epic" },
  PLAN: { route: "/plans", param: "plan", cssVar: "--entity-plan" },
  NOTE: { route: "/notes", param: "note", cssVar: "--entity-note" },
};

export function WikiLinkChip({ id }: { id: string }) {
  const prefix = id.split("-")[0];
  const config = PREFIX_CONFIG[prefix] || PREFIX_CONFIG.TASK;

  return (
    <Link
      href={`${config.route}?${config.param}=${id}`}
      className="inline-flex items-center font-mono text-xs px-1.5 py-0.5 rounded no-underline hover:opacity-80 transition-opacity"
      style={{
        backgroundColor: `color-mix(in oklch, var(${config.cssVar}) 15%, transparent)`,
        color: `var(${config.cssVar})`,
      }}
    >
      {id}
    </Link>
  );
}
