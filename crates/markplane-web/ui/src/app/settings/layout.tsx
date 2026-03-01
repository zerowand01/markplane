"use client";

import type { ReactNode } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Settings, List, FileText, GitBranch } from "lucide-react";
import { cn } from "@/lib/utils";

const NAV_ITEMS = [
  { href: "/settings/general", label: "General", icon: Settings },
  { href: "/settings/task-types", label: "Task Types", icon: List },
  { href: "/settings/note-types", label: "Note Types", icon: FileText },
  { href: "/settings/workflow", label: "Task Workflow", icon: GitBranch },
];

export default function SettingsLayout({ children }: { children: ReactNode }) {
  const pathname = usePathname();
  const isActive = (href: string) =>
    pathname === href || pathname === `${href}/`;

  return (
    <div className="p-4 md:p-6">
      <div className="flex flex-col md:flex-row gap-6">
        {/* Mobile: horizontal scrollable tabs */}
        <nav className="md:hidden sticky top-0 z-10 -mx-4 bg-background px-4 py-2 flex gap-1 overflow-x-auto border-b">
          {NAV_ITEMS.map(({ href, label, icon: Icon }) => (
            <Link
              key={href}
              href={href}
              className={cn(
                "flex items-center gap-1.5 rounded-md px-3 py-2 text-sm font-medium whitespace-nowrap transition-colors",
                isActive(href)
                  ? "bg-accent font-medium text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent hover:text-foreground"
              )}
            >
              <Icon className="size-4" />
              {label}
            </Link>
          ))}
        </nav>
        {/* Desktop: vertical sidebar — sticky so content scrolls independently */}
        <nav className="hidden md:flex md:w-52 md:shrink-0 flex-col gap-1 sticky top-6 self-start">
          {NAV_ITEMS.map(({ href, label, icon: Icon }) => (
            <Link
              key={href}
              href={href}
              className={cn(
                "flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                isActive(href)
                  ? "bg-accent font-medium text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
              )}
            >
              <Icon className="size-4" />
              {label}
            </Link>
          ))}
        </nav>
        <div className="flex-1 min-w-0">{children}</div>
      </div>
    </div>
  );
}
