"use client";

import dynamic from "next/dynamic";

const TaskTypesSection = dynamic(
  () => import("../sections/task-types-section").then((m) => ({ default: m.TaskTypesSection })),
  { ssr: false }
);

export default function TaskTypesPage() {
  return <TaskTypesSection />;
}
