"use client";

import { SidebarTrigger, useSidebar } from "@/components/ui/sidebar";

export function MobileHeader() {
  const { isMobile } = useSidebar();

  if (!isMobile) return null;

  return (
    <header className="flex items-center gap-2 px-4 py-3 border-b">
      <SidebarTrigger />
    </header>
  );
}
