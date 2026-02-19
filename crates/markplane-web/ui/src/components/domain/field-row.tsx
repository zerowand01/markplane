import type { ReactNode } from "react";

export function FieldRow({
  label,
  editable,
  children,
}: {
  label: string;
  editable?: boolean;
  children: ReactNode;
}) {
  return (
    <div
      className={`flex items-start gap-4 px-1 py-1 rounded ${editable ? "hover:bg-muted/50" : ""}`}
    >
      <span className="text-muted-foreground w-20 shrink-0 text-sm leading-6">
        {label}
      </span>
      <div className="flex-1 min-w-0 text-sm leading-6">{children}</div>
    </div>
  );
}

export function EmptyValue({ children }: { children?: ReactNode }) {
  return (
    <span className="text-sm text-muted-foreground italic">
      {children ?? "None"}
    </span>
  );
}
