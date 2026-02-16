"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import { SheetContent } from "@/components/ui/sheet";
import { cn } from "@/lib/utils";

const STORAGE_KEY = "markplane-sheet-width";
const MIN_WIDTH = 480;
const MAX_WIDTH = 960;
const DEFAULT_WIDTH = 680;

export function ResizableSheetContent({
  children,
  className,
  ...props
}: React.ComponentProps<typeof SheetContent>) {
  const [width, setWidth] = useState(DEFAULT_WIDTH);
  const widthRef = useRef(DEFAULT_WIDTH);

  // Load persisted width on mount
  useEffect(() => {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = parseInt(stored, 10);
      if (!isNaN(parsed)) {
        const clamped = Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, parsed));
        setWidth(clamped);
        widthRef.current = clamped;
      }
    }
  }, []);

  const handlePointerDown = useCallback((e: React.PointerEvent) => {
    e.preventDefault();
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";

    const handlePointerMove = (ev: PointerEvent) => {
      const newWidth = Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, window.innerWidth - ev.clientX));
      widthRef.current = newWidth;
      setWidth(newWidth);
    };

    const handlePointerUp = () => {
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      localStorage.setItem(STORAGE_KEY, String(widthRef.current));
      document.removeEventListener("pointermove", handlePointerMove);
      document.removeEventListener("pointerup", handlePointerUp);
    };

    document.addEventListener("pointermove", handlePointerMove);
    document.addEventListener("pointerup", handlePointerUp);
  }, []);

  return (
    <SheetContent
      className={cn("overflow-hidden sm:max-w-none bg-card", className)}
      style={{ width: `min(${width}px, 75vw)` }}
      {...props}
    >
      {/* Resize handle — outside scroll container so it spans full height */}
      <div
        aria-hidden
        className="absolute left-0 top-0 bottom-0 w-1.5 cursor-col-resize z-10"
        onPointerDown={handlePointerDown}
      >
        <div className="absolute inset-y-0 left-0 w-1.5 bg-primary/0 hover:bg-primary/15 active:bg-primary/30 transition-colors" />
      </div>
      {/* Scrollable content */}
      <div className="overflow-y-auto flex-1 min-h-0 flex flex-col gap-4">
        {children}
      </div>
    </SheetContent>
  );
}
