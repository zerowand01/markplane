"use client";

import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { AlertTriangle } from "lucide-react";
import { useDataVersionMismatch } from "@/lib/data-version";

/**
 * Persistent, non-dismissable banner shown while the server refuses requests
 * because the project's data version does not match the running markplane
 * binary. The server's message says which side is behind. The banner clears
 * itself once a request succeeds again, and at that moment refetches every
 * query so pages that failed during the mismatch recover without a reload.
 */
export function DataVersionBanner() {
  const message = useDataVersionMismatch();
  const queryClient = useQueryClient();
  const wasMismatched = useRef(false);

  useEffect(() => {
    if (message) {
      wasMismatched.current = true;
    } else if (wasMismatched.current) {
      wasMismatched.current = false;
      queryClient.invalidateQueries();
    }
  }, [message, queryClient]);

  if (!message) return null;

  return (
    <div
      role="alert"
      className="flex items-start gap-3 border-b border-destructive/40 bg-destructive/10 px-4 py-3 text-sm"
    >
      <AlertTriangle className="mt-0.5 size-4 shrink-0 text-destructive" aria-hidden="true" />
      <div className="min-w-0 space-y-0.5">
        <p className="font-medium text-destructive">
          markplane cannot use this project&apos;s data
        </p>
        <p className="text-muted-foreground break-words">
          {message} Restart <code className="font-mono text-xs">markplane serve</code> after
          fixing it.
        </p>
      </div>
    </div>
  );
}
