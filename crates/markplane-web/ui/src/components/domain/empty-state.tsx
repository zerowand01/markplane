import { Card, CardContent } from "@/components/ui/card";
import { Plane } from "lucide-react";

export function EmptyState({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children?: React.ReactNode;
}) {
  return (
    <Card className="border-dashed">
      <CardContent className="flex flex-col items-center justify-center gap-3 py-12">
        <Plane className="size-8 text-muted-foreground/40" />
        <div className="text-center space-y-1">
          <p className="text-sm font-medium text-muted-foreground">{title}</p>
          {description && (
            <p className="text-xs text-muted-foreground/70 max-w-sm">{description}</p>
          )}
        </div>
        {children}
      </CardContent>
    </Card>
  );
}
