"use client";

import dynamic from "next/dynamic";

const SettingsContent = dynamic(
  () => import("./settings-content").then((m) => ({ default: m.SettingsContent })),
  { ssr: false }
);

export default function SettingsPage() {
  return (
    <div className="p-4 md:p-6">
      <SettingsContent />
    </div>
  );
}
