import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  trailingSlash: true,
  images: {
    unoptimized: true,
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
