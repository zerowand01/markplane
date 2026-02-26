"use client";

import dynamic from "next/dynamic";
import { PlansSkeleton } from "./plans-content";

const PlansContent = dynamic(
  () => import("./plans-content").then((m) => ({ default: m.PlansContent })),
  { ssr: false, loading: () => <PlansSkeleton /> }
);

export default function PlansPage() {
  return (
    <div className="p-4 md:p-6">
      <PlansContent />
    </div>
  );
}
