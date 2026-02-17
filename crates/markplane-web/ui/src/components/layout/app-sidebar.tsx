"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  CheckSquare,
  FileText,
  Lightbulb,
  GitBranch,
  Map,
  Search,
  Sun,
  Moon,
} from "lucide-react";
import { useTheme } from "next-themes";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { Button } from "@/components/ui/button";
import { MarkplaneLogo } from "@/components/ui/markplane-logo";

const ICON_MAP = {
  LayoutDashboard,
  CheckSquare,
  FileText,
  Lightbulb,
  GitBranch,
  Map,
} as const;

const mainNav = [
  { href: "/dashboard", label: "Dashboard", icon: "LayoutDashboard" as const },
  { href: "/roadmap", label: "Roadmap", icon: "Map" as const },
  { href: "/backlog", label: "Backlog", icon: "CheckSquare" as const },
  { href: "/plans", label: "Plans", icon: "FileText" as const },
  { href: "/notes", label: "Notes", icon: "Lightbulb" as const },
  { href: "/graph", label: "Graph", icon: "GitBranch" as const },
];

export function AppSidebar() {
  const pathname = usePathname();
  const { theme, setTheme } = useTheme();

  return (
    <Sidebar>
      <SidebarHeader className="px-4 py-3">
        <div className="flex items-center justify-between">
          <Link href="/dashboard" className="flex items-center gap-2">
            <MarkplaneLogo className="size-6 text-primary" />
            <span className="text-lg font-semibold tracking-tight font-mono">Markplane</span>
          </Link>
          <button
            onClick={() => window.dispatchEvent(new Event("open-command-palette"))}
            className="size-8 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
            title="Search (⌘K)"
          >
            <Search className="size-4" />
          </button>
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {mainNav.map((item) => {
                const Icon = ICON_MAP[item.icon];
                return (
                  <SidebarMenuItem key={item.href}>
                    <SidebarMenuButton asChild isActive={pathname.startsWith(item.href)} className="h-10 text-base">
                      <Link href={item.href}>
                        <Icon className="size-5" />
                        <span>{item.label}</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="px-3 pb-3">
        <Button
          variant="ghost"
          size="sm"
          className="w-full justify-start gap-2"
          onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
        >
          <Sun className="size-5 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
          <Moon className="absolute size-5 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
          <span className="dark:hidden">Light</span>
          <span className="hidden dark:inline">Dark</span>
        </Button>
      </SidebarFooter>
    </Sidebar>
  );
}
