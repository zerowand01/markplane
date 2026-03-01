"use client";

import dynamic from "next/dynamic";

const NoteTypesSection = dynamic(
  () => import("../sections/note-types-section").then((m) => ({ default: m.NoteTypesSection })),
  { ssr: false }
);

export default function NoteTypesPage() {
  return <NoteTypesSection />;
}
