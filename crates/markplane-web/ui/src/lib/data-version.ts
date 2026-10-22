import { useSyncExternalStore } from "react";

/**
 * Error code the server returns (HTTP 503) on every API request while
 * `.markplane/config.yaml` is at a data version this binary does not support.
 * See docs/file-format.md#version.
 */
export const DATA_VERSION_MISMATCH = "data_version_mismatch";

// Tiny module-level store: the mismatch is a property of the server, not of
// any one query, so every page reads the same value without a provider.
let message: string | null = null;
const listeners = new Set<() => void>();

function emit() {
  for (const listener of listeners) listener();
}

/** Record (or clear, with `null`) the server's data version mismatch message. */
export function setDataVersionMismatch(next: string | null) {
  if (next === message) return;
  message = next;
  emit();
}

export function getDataVersionMismatch(): string | null {
  return message;
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

function getServerSnapshot(): string | null {
  return null;
}

/** The current mismatch message, or `null` when the server is healthy. */
export function useDataVersionMismatch(): string | null {
  return useSyncExternalStore(subscribe, getDataVersionMismatch, getServerSnapshot);
}
