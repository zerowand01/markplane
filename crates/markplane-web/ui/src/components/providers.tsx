"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider } from "next-themes";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useState } from "react";
import { useWebSocket } from "@/lib/hooks/use-websocket";
import { useKeyboardNav } from "@/lib/hooks/use-keyboard-nav";

function WebSocketProvider({ children }: { children: React.ReactNode }) {
  useWebSocket();
  return <>{children}</>;
}

function KeyboardNavProvider({ children }: { children: React.ReactNode }) {
  useKeyboardNav();
  return <>{children}</>;
}

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 30_000,
            gcTime: 5 * 60_000,
            refetchOnWindowFocus: false,
            retry: 1,
          },
        },
      })
  );

  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider attribute="class" defaultTheme="dark" enableSystem>
        <TooltipProvider>
          <WebSocketProvider>
            <KeyboardNavProvider>{children}</KeyboardNavProvider>
          </WebSocketProvider>
        </TooltipProvider>
      </ThemeProvider>
    </QueryClientProvider>
  );
}
