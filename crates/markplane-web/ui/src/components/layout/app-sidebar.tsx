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
  Archive,
  BookOpen,
  PanelLeft,
  Search,
  Settings,
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
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
  SidebarSeparator,
  useSidebar,
} from "@/components/ui/sidebar";
import { MarkplaneLogo } from "@/components/ui/markplane-logo";

const ICON_MAP = {
  LayoutDashboard,
  CheckSquare,
  FileText,
  Lightbulb,
  GitBranch,
  Map,
  Archive,
} as const;

const mainNav = [
  { href: "/dashboard", label: "Dashboard", icon: "LayoutDashboard" as const },
  { href: "/roadmap", label: "Roadmap", icon: "Map" as const },
  { href: "/backlog", label: "Backlog", icon: "CheckSquare" as const },
  { href: "/plans", label: "Plans", icon: "FileText" as const },
  { href: "/notes", label: "Notes", icon: "Lightbulb" as const },
  { href: "/graph", label: "Graph", icon: "GitBranch" as const },
  { href: "/archive", label: "Archive", icon: "Archive" as const },
];

export function AppSidebar() {
  const pathname = usePathname();
  const { theme, setTheme } = useTheme();
  const { isMobile, toggleSidebar } = useSidebar();

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader className="p-2 pt-3">
        <Link
          href="/dashboard"
          className="flex items-center gap-2 overflow-hidden rounded-md p-1"
        >
          <MarkplaneLogo className="size-6 text-primary shrink-0" />
          <span className="text-lg font-semibold tracking-tight font-mono whitespace-nowrap">Markplane</span>
        </Link>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup className="pb-0">
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton
                  tooltip="Search (⌘K)"
                  onClick={() => window.dispatchEvent(new Event("open-command-palette"))}
                  className="text-base"
                >
                  <Search className="size-5" />
                  <span>Search</span>
                </SidebarMenuButton>
                <SidebarMenuBadge className="text-muted-foreground/60 text-xs">⌘K</SidebarMenuBadge>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarSeparator />
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {mainNav.map((item) => {
                const Icon = ICON_MAP[item.icon];
                return (
                  <SidebarMenuItem key={item.href}>
                    <SidebarMenuButton asChild isActive={pathname.startsWith(item.href)} tooltip={item.label} className="text-base">
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

      <SidebarFooter className="p-2">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton asChild isActive={pathname.startsWith("/docs")} tooltip="Docs" className="text-base">
              <Link href="/docs">
                <BookOpen className="size-5" />
                <span>Docs</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton asChild isActive={pathname.startsWith("/settings")} tooltip="Settings" className="text-base">
              <Link href="/settings">
                <Settings className="size-5" />
                <span>Settings</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
        <SidebarSeparator className="mx-0" />
        <div className="flex items-center justify-between group-data-[collapsible=icon]:flex-col group-data-[collapsible=icon]:gap-1">
          <button
            onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            className="size-8 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
            title="Toggle theme"
          >
            <Sun className="size-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
            <Moon className="absolute size-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
          </button>
          {!isMobile && (
            <button
              onClick={toggleSidebar}
              className="size-8 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
              title="Toggle sidebar (⌘B)"
            >
              <PanelLeft className="size-4" />
            </button>
          )}
        </div>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
