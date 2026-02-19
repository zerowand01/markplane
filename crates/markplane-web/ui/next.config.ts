import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
  // In dev mode, the WebSocket must connect directly to the Rust server
  // since Next.js rewrites don't proxy WebSocket upgrades.
  env: {
    NEXT_PUBLIC_WS_URL:
      process.env.NODE_ENV === "development"
        ? "ws://localhost:4200/ws"
        : undefined,
  },
  // Rewrites only apply during `next dev` (ignored in static export).
  // Proxies API calls to the Rust server so the UI works in dev mode.
  async rewrites() {
    return [
      {
        source: "/api/:path*",
        destination: "http://localhost:4200/api/:path*",
      },
      {
        source: "/ws",
        destination: "http://localhost:4200/ws",
      },
    ];
  },
};

export default nextConfig;
