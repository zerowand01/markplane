"use client";

import dynamic from "next/dynamic";
import { NotesSkeleton } from "./notes-content";

const NotesContent = dynamic(
  () => import("./notes-content").then((m) => ({ default: m.NotesContent })),
  { ssr: false, loading: () => <NotesSkeleton /> }
);

export default function NotesPage() {
  return (
    <div className="p-4 md:p-6">
      <NotesContent />
    </div>
  );
}
