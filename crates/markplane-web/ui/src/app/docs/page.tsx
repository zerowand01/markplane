"use client";

import dynamic from "next/dynamic";
import { DocsSkeleton } from "./docs-content";

const DocsContent = dynamic(
  () => import("./docs-content").then((m) => ({ default: m.DocsContent })),
  { ssr: false, loading: () => <DocsSkeleton /> }
);

export default function DocsPage() {
  return <DocsContent />;
}
