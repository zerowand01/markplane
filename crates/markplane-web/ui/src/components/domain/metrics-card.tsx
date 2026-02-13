import { Card, CardContent } from "@/components/ui/card";

interface MetricsCardProps {
  label: string;
  value: number | string;
  accent?: "default" | "warning" | "muted";
}

export function MetricsCard({ label, value, accent = "default" }: MetricsCardProps) {
  return (
    <Card
      className={
        accent === "warning"
          ? "border-l-2 border-l-status-blocked"
          : undefined
      }
    >
      <CardContent className="p-4">
        <div className="text-2xl font-bold tracking-tight">{value}</div>
        <div className="text-xs text-muted-foreground mt-1">{label}</div>
      </CardContent>
    </Card>
  );
}
