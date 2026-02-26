"use client";

import dynamic from "next/dynamic";
import { GraphSkeleton } from "./graph-content";

const GraphContent = dynamic(
  () => import("./graph-content").then((m) => ({ default: m.GraphContent })),
  { ssr: false, loading: () => <GraphSkeleton /> }
);

export default function GraphPage() {
  return <GraphContent />;
}
