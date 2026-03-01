"use client";

import dynamic from "next/dynamic";

const GeneralSection = dynamic(
  () => import("../sections/general-section").then((m) => ({ default: m.GeneralSection })),
  { ssr: false }
);

export default function GeneralPage() {
  return <GeneralSection />;
}
