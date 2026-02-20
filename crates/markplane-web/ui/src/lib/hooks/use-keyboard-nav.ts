"use client";

import { useEffect, useRef, useCallback } from "react";
import { useRouter } from "next/navigation";
import type { AppRouterInstance } from "next/dist/shared/lib/app-router-context.shared-runtime";

type ChordMap = Record<string, string>;

const CHORD_NAV: ChordMap = {
  d: "/dashboard",
  b: "/backlog",
  p: "/plans",
  n: "/notes",
  r: "/roadmap",
  g: "/graph",
  a: "/archive",
};

function isInputFocused(): boolean {
  const el = document.activeElement;
  if (!el) return false;
  const tag = el.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
  if ((el as HTMLElement).isContentEditable) return true;
  // cmdk input
  if (el.getAttribute("role") === "combobox") return true;
  return false;
}

export function useKeyboardNav() {
  const router = useRouter();
  const routerRef = useRef<AppRouterInstance>(router);
  routerRef.current = router;

  const chordPending = useRef(false);
  const chordTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const resetChord = useCallback(() => {
    chordPending.current = false;
    if (chordTimer.current) {
      clearTimeout(chordTimer.current);
      chordTimer.current = null;
    }
  }, []);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      // Skip when typing in inputs or when modifiers are held (except Escape)
      if (e.key !== "Escape" && (isInputFocused() || e.metaKey || e.ctrlKey || e.altKey)) {
        return;
      }

      // Escape: close sheets/dialogs/command palette
      if (e.key === "Escape") {
        resetChord();
        return; // Let Radix UI handle the actual closing
      }

      // ? opens command palette (Cmd+K)
      if (e.key === "?") {
        e.preventDefault();
        window.dispatchEvent(new KeyboardEvent("keydown", { key: "k", metaKey: true }));
        return;
      }

      // Chord: g then <letter> for navigation
      if (chordPending.current) {
        resetChord();
        const dest = CHORD_NAV[e.key];
        if (dest) {
          e.preventDefault();
          routerRef.current.push(dest);
        }
        return;
      }

      if (e.key === "g") {
        e.preventDefault();
        chordPending.current = true;
        // Auto-cancel chord after 1.5s
        chordTimer.current = setTimeout(resetChord, 1500);
        return;
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      resetChord();
    };
  }, [resetChord]);
}
