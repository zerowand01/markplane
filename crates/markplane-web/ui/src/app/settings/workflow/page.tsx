"use client";

import dynamic from "next/dynamic";

const WorkflowSection = dynamic(
  () => import("../sections/workflow-section").then((m) => ({ default: m.WorkflowSection })),
  { ssr: false }
);

export default function WorkflowPage() {
  return <WorkflowSection />;
}
