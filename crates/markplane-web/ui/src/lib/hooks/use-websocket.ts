"use client";

import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { WsEvent } from "@/lib/types";

export function useWebSocket() {
  const queryClient = useQueryClient();
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout>>(null);

  useEffect(() => {
    function connect() {
      const url =
        process.env.NEXT_PUBLIC_WS_URL ??
        `${window.location.protocol === "https:" ? "wss:" : "ws:"}//${window.location.host}/ws`;
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        // Clear reconnect timer on successful connection
        if (reconnectTimer.current) {
          clearTimeout(reconnectTimer.current);
          reconnectTimer.current = null;
        }
      };

      ws.onmessage = (event) => {
        try {
          const msg: WsEvent = JSON.parse(event.data);

          switch (msg.type) {
            case "file_changed": {
              const queryPrefix = msg.entity + "s"; // "task" → "tasks"
              queryClient.invalidateQueries({ queryKey: [queryPrefix] });
              queryClient.invalidateQueries({ queryKey: ["summary"] });
              queryClient.invalidateQueries({ queryKey: ["graph"] });
              break;
            }
            case "config_changed":
              queryClient.invalidateQueries({ queryKey: ["summary"] });
              break;
            case "sync_complete":
              queryClient.invalidateQueries();
              break;
            case "connected":
              break;
          }
        } catch {
          // Ignore malformed messages
        }
      };

      ws.onclose = () => {
        wsRef.current = null;
        // Reconnect after 2 seconds
        reconnectTimer.current = setTimeout(connect, 2000);
      };

      ws.onerror = () => {
        ws.close();
      };
    }

    connect();

    return () => {
      if (reconnectTimer.current) {
        clearTimeout(reconnectTimer.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [queryClient]);
}
