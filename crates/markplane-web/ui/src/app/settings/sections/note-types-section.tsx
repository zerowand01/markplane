"use client";

import { useConfig } from "@/lib/hooks/use-config";
import { useUpdateConfig } from "@/lib/hooks/use-mutations";
import { Card, CardContent } from "@/components/ui/card";
import { PageTransition } from "@/components/domain/page-transition";
import { TypeListEditor } from "./type-list-editor";

export function NoteTypesSection() {
  const { data: config, isLoading } = useConfig();
  const updateConfig = useUpdateConfig();

  if (isLoading || !config) {
    return (
      <Card>
        <CardContent className="p-6">
          <div className="h-48 animate-pulse rounded bg-muted" />
        </CardContent>
      </Card>
    );
  }

  return (
    <PageTransition>
      <TypeListEditor
        title="Note Types"
        description="First item is the default for new notes"
        items={config.note_types}
        onUpdate={(note_types) => updateConfig.mutate({ note_types })}
        isPending={updateConfig.isPending}
      />
    </PageTransition>
  );
}
