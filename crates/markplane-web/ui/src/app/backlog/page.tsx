"use client";

import dynamic from "next/dynamic";
import { BacklogSkeleton } from "./backlog-content";

const BacklogContent = dynamic(
  () => import("./backlog-content").then((m) => ({ default: m.BacklogContent })),
  { ssr: false, loading: () => <BacklogSkeleton /> }
);

export default function BacklogPage() {
  return (
    <div className="p-4 md:p-6">
      <BacklogContent />
    </div>
  );
}
