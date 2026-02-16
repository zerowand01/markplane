"use client";

import { useState, useCallback, useRef } from "react";
import { useRouter } from "next/navigation";
import { useSearch } from "@/lib/hooks/use-search";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { StatusBadge } from "@/components/domain/status-badge";
import { PriorityIndicator } from "@/components/domain/priority-indicator";
import { PREFIX_CONFIG } from "@/lib/constants";
import { PageTransition } from "@/components/domain/page-transition";
import { EmptyState } from "@/components/domain/empty-state";
import type { Priority, TaskStatus, SearchResult } from "@/lib/types";

function useDebounce(delay: number) {
  const [value, setValue] = useState("");
  const [debouncedValue, setDebouncedValue] = useState("");
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const onChange = useCallback(
    (newValue: string) => {
      setValue(newValue);
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => setDebouncedValue(newValue), delay);
    },
    [delay]
  );

  return { value, debouncedValue, onChange };
}

function highlightMatch(text: string, query: string) {
  if (!query) return text;
  const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")})`, "gi");
  const parts = text.split(regex);
  return parts.map((part, i) =>
    regex.test(part) ? (
      <mark key={i} className="bg-primary/20 text-foreground rounded px-0.5">
        {part}
      </mark>
    ) : (
      part
    )
  );
}

function resultRoute(result: SearchResult) {
  const prefix = result.id.split("-")[0];
  const config = PREFIX_CONFIG[prefix];
  if (!config) return "/backlog";
  return `${config.route}?${result.entity_type === "task" ? "task" : result.entity_type}=${result.id}`;
}

export default function SearchPage() {
  const router = useRouter();
  const { value, debouncedValue, onChange } = useDebounce(300);
  const { data, isLoading } = useSearch(debouncedValue);

  const results = data?.data ?? [];

  return (
    <PageTransition>
    <div className="p-4 md:p-6 space-y-6 max-w-3xl">
      <Input
        type="search"
        placeholder="Search by title or content..."
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="text-base h-11"
        autoFocus
      />

      {debouncedValue.length >= 2 && (
        <div className="space-y-3">
          {isLoading ? (
            Array.from({ length: 3 }).map((_, i) => (
              <Skeleton key={i} className="h-24 w-full" />
            ))
          ) : results.length === 0 ? (
            <EmptyState
              title="No matches found"
              description="Try different keywords or broaden your search"
            />
          ) : (
            <>
              <p className="text-sm text-muted-foreground">
                {results.length} result{results.length !== 1 ? "s" : ""} for
                &ldquo;{debouncedValue}&rdquo;
              </p>
              {results.map((result) => {
                const prefix = result.id.split("-")[0];
                const entityColor =
                  PREFIX_CONFIG[prefix]?.color ?? "var(--entity-task)";

                return (
                  <Card
                    key={result.id}
                    className="cursor-pointer hover:border-ring transition-colors"
                    onClick={() => router.push(resultRoute(result))}
                  >
                    <CardContent className="p-4">
                      <div className="flex items-center gap-2 mb-1">
                        <span
                          className="font-mono text-xs"
                          style={{ color: entityColor }}
                        >
                          {result.id}
                        </span>
                        <span className="font-medium text-sm flex-1">
                          {highlightMatch(result.title, debouncedValue)}
                        </span>
                      </div>
                      <div className="flex items-center gap-2 mt-1">
                        {result.priority && (
                          <PriorityIndicator
                            priority={result.priority as Priority}
                            showLabel
                          />
                        )}
                        <StatusBadge status={result.status as TaskStatus} />
                      </div>
                      {result.snippet && (
                        <p className="text-xs text-muted-foreground mt-2 line-clamp-2">
                          {highlightMatch(result.snippet, debouncedValue)}
                        </p>
                      )}
                    </CardContent>
                  </Card>
                );
              })}
            </>
          )}
        </div>
      )}
    </div>
    </PageTransition>
  );
}
