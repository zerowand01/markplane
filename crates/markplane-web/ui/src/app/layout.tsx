import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { Providers } from "@/components/providers";
import { AppSidebar } from "@/components/layout/app-sidebar";
import { CommandPaletteWrapper } from "@/components/layout/command-palette-wrapper";
import { GlobalCreateDialog } from "@/components/layout/global-create-dialog";
import { ErrorBoundary } from "@/components/domain/error-boundary";
import { MobileHeader } from "@/components/layout/mobile-header";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { Toaster } from "@/components/ui/sonner";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Markplane",
  description: "AI-native project management",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${geistSans.variable} ${geistMono.variable} font-sans antialiased`}
      >
        <Providers>
          <SidebarProvider>
            <AppSidebar />
            <SidebarInset>
              <MobileHeader />
              <main className="flex-1">
                <ErrorBoundary>{children}</ErrorBoundary>
              </main>
            </SidebarInset>
            <CommandPaletteWrapper />
            <GlobalCreateDialog />
          </SidebarProvider>
          <Toaster position="bottom-right" />
        </Providers>
      </body>
    </html>
  );
}
