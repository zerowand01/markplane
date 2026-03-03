"use client";

import { useState, useCallback } from "react";
import { Plus, X } from "lucide-react";
import { useConfig } from "@/lib/hooks/use-config";
import { useUpdateConfig } from "@/lib/hooks/use-mutations";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { PageTransition } from "@/components/domain/page-transition";
import type { ProjectConfig } from "@/lib/types";

export function GeneralSection() {
  const { data: config, isLoading, dataUpdatedAt } = useConfig();

  if (isLoading || !config) {
    return (
      <div className="space-y-6">
        <Card>
          <CardContent className="p-6">
            <div className="h-32 animate-pulse rounded bg-muted" />
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-6">
            <div className="h-32 animate-pulse rounded bg-muted" />
          </CardContent>
        </Card>
      </div>
    );
  }

  return <GeneralSectionForm key={dataUpdatedAt} config={config} />;
}

function GeneralSectionForm({ config }: { config: ProjectConfig }) {
  const updateConfig = useUpdateConfig();

  // Local state for text fields (save on blur) — initialized from config props
  const [projectName, setProjectName] = useState(config.project.name);
  const [projectDescription, setProjectDescription] = useState(config.project.description);
  const [tokenBudget, setTokenBudget] = useState(String(config.context.token_budget));
  const [recentDays, setRecentDays] = useState(String(config.context.recent_days));
  const [newDocPath, setNewDocPath] = useState("");

  const saveProject = useCallback(
    (name: string, description: string) => {
      if (name === config.project.name && description === config.project.description) return;
      updateConfig.mutate({ project: { name, description } });
    },
    [config, updateConfig],
  );

  const saveContext = useCallback(
    (field: "token_budget" | "recent_days", raw: string) => {
      const num = parseInt(raw, 10);
      if (isNaN(num)) return;
      if (num === config.context[field]) return;
      updateConfig.mutate({ context: { [field]: num } });
    },
    [config, updateConfig],
  );

  const handleAutoGenerateToggle = useCallback(
    (checked: boolean) => {
      updateConfig.mutate({ context: { auto_generate: checked } });
    },
    [updateConfig],
  );

  const handleAddDocPath = useCallback(() => {
    const trimmed = newDocPath.trim();
    if (!trimmed) return;
    if (config.documentation_paths.includes(trimmed)) return;
    setNewDocPath("");
    updateConfig.mutate({
      documentation_paths: [...config.documentation_paths, trimmed],
    });
  }, [config, newDocPath, updateConfig]);

  const handleRemoveDocPath = useCallback(
    (index: number) => {
      updateConfig.mutate({
        documentation_paths: config.documentation_paths.filter((_, i) => i !== index),
      });
    },
    [config, updateConfig],
  );

  return (
    <PageTransition>
      <div className="space-y-6">
        {/* Project */}
        <Card>
          <CardHeader>
            <CardTitle>Project</CardTitle>
            <CardDescription>Basic information about your project</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="project-name">Name</Label>
              <Input
                id="project-name"
                value={projectName}
                onChange={(e) => setProjectName(e.target.value)}
                onBlur={() => saveProject(projectName, projectDescription)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    (e.target as HTMLInputElement).blur();
                  }
                }}
                placeholder="My Project"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="project-description">Description</Label>
              <Textarea
                id="project-description"
                value={projectDescription}
                onChange={(e) => setProjectDescription(e.target.value)}
                onBlur={() => saveProject(projectName, projectDescription)}
                placeholder="A brief description of your project"
                rows={3}
              />
            </div>
          </CardContent>
        </Card>

        {/* Documentation Paths */}
        <Card>
          <CardHeader>
            <CardTitle>Documentation Paths</CardTitle>
            <CardDescription>
              Directories containing project documentation (relative to project root)
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {config.documentation_paths.length > 0 && (
              <div className="space-y-1.5">
                {config.documentation_paths.map((path, index) => (
                  <div
                    key={path}
                    className="flex items-center gap-2 rounded-md border bg-card px-3 py-2"
                  >
                    <span className="flex-1 text-sm font-mono">{path}</span>
                    <button
                      onClick={() => handleRemoveDocPath(index)}
                      className="text-muted-foreground hover:text-destructive"
                      title={`Remove "${path}"`}
                    >
                      <X className="size-4" />
                    </button>
                  </div>
                ))}
              </div>
            )}
            <div className="flex items-center gap-2">
              <Input
                placeholder="docs/"
                value={newDocPath}
                onChange={(e) => setNewDocPath(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    handleAddDocPath();
                  }
                }}
                className="font-mono"
              />
              <Button
                variant="outline"
                size="icon"
                onClick={handleAddDocPath}
                disabled={!newDocPath.trim()}
              >
                <Plus className="size-4" />
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Context Generation */}
        <Card>
          <CardHeader>
            <CardTitle>Context Generation</CardTitle>
            <CardDescription>
              Configure how AI context summaries are generated
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label htmlFor="auto-generate">Auto-generate</Label>
                <p className="text-xs text-muted-foreground">
                  Automatically regenerate context on sync
                </p>
              </div>
              <Switch
                id="auto-generate"
                checked={config.context.auto_generate}
                onCheckedChange={handleAutoGenerateToggle}
              />
            </div>
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="token-budget">Token Budget</Label>
                <Input
                  id="token-budget"
                  type="number"
                  min={1}
                  max={1000000}
                  value={tokenBudget}
                  onChange={(e) => setTokenBudget(e.target.value)}
                  onBlur={() => saveContext("token_budget", tokenBudget)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      (e.target as HTMLInputElement).blur();
                    }
                  }}
                />
                <p className="text-xs text-muted-foreground">
                  Maximum tokens for context summary (1 - 1,000,000)
                </p>
              </div>
              <div className="space-y-2">
                <Label htmlFor="recent-days">Recent Days</Label>
                <Input
                  id="recent-days"
                  type="number"
                  min={1}
                  max={365}
                  value={recentDays}
                  onChange={(e) => setRecentDays(e.target.value)}
                  onBlur={() => saveContext("recent_days", recentDays)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      (e.target as HTMLInputElement).blur();
                    }
                  }}
                />
                <p className="text-xs text-muted-foreground">
                  Days to consider for recent activity (1 - 365)
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </PageTransition>
  );
}
