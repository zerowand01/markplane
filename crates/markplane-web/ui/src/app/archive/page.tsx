"use client";

import dynamic from "next/dynamic";
import { ArchiveSkeleton } from "./archive-content";

const ArchiveContent = dynamic(
  () => import("./archive-content").then((m) => ({ default: m.ArchiveContent })),
  { ssr: false, loading: () => <ArchiveSkeleton /> }
);

export default function ArchivePage() {
  return (
    <div className="p-4 md:p-6">
      <ArchiveContent />
    </div>
  );
}
