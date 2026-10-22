"use client";

import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { WsEvent } from "@/lib/types";
import { getDataVersionMismatch } from "@/lib/data-version";

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
        // The /ws route sits behind the same data version gate as the API, so
        // a successful connection while the banner is up means the server was
        // restarted with an upgraded binary. Refetch so the banner clears and
        // the pages fill in without a manual reload.
        if (getDataVersionMismatch()) {
          queryClient.invalidateQueries();
        }
      };

      ws.onmessage = (event) => {
        try {
          const msg: WsEvent = JSON.parse(event.data);

          // While the data version banner is up, every query that ran is in
          // error state with retries off. Any change on the server (such as
          // config.yaml being fixed) is a reason to refetch all of them, not
          // just the keys this event names, so the banner can clear.
          if (getDataVersionMismatch()) {
            queryClient.invalidateQueries();
            return;
          }

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
              queryClient.invalidateQueries({ queryKey: ["config"] });
              break;
            case "sync_complete":
              queryClient.invalidateQueries();
              break;
            case "doc_changed":
              queryClient.invalidateQueries({ queryKey: ["docs"] });
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
        // Reconnect after 2 seconds; back off while the server is refusing
        // connections over a data version mismatch.
        const delay = getDataVersionMismatch() ? 5000 : 2000;
        reconnectTimer.current = setTimeout(connect, delay);
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
