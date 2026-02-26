"use client";

import dynamic from "next/dynamic";
import { RoadmapSkeleton } from "./roadmap-content";

const RoadmapContent = dynamic(
  () => import("./roadmap-content").then((m) => ({ default: m.RoadmapContent })),
  { ssr: false, loading: () => <RoadmapSkeleton /> }
);

export default function RoadmapPage() {
  return (
    <div className="p-4 md:p-6">
      <RoadmapContent />
    </div>
  );
}
