"use client";

import dynamic from "next/dynamic";
import { DashboardSkeleton } from "./dashboard-content";

const DashboardContent = dynamic(
  () => import("./dashboard-content").then((m) => ({ default: m.DashboardContent })),
  { ssr: false, loading: () => <div className="p-4 md:p-6"><DashboardSkeleton /></div> }
);

export default function DashboardPage() {
  return <DashboardContent />;
}
