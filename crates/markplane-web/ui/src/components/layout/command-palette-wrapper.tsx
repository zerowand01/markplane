"use client";

import dynamic from "next/dynamic";

const CommandPalette = dynamic(
  () =>
    import("@/components/layout/command-palette").then(
      (mod) => mod.CommandPalette
    ),
  { ssr: false }
);

export function CommandPaletteWrapper() {
  return <CommandPalette />;
}
