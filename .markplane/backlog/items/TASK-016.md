---
id: TASK-016
title: Add SSE transport for MCP server
status: backlog
priority: low
type: feature
effort: medium
tags:
- mcp
epic: EPIC-005
plan: null
depends_on: []
blocks: []
assignee: null
position: a6
created: 2026-02-10
updated: 2026-02-10
---

# Add SSE transport for MCP server

## Description

The MCP server currently only supports stdio transport — it reads JSON-RPC from stdin and writes to stdout. This works for desktop clients like Claude Code and Cursor that spawn the server as a subprocess, but it doesn't work for web-based clients or remote connections. The MCP spec defines an SSE (Server-Sent Events) transport where the server listens on an HTTP port and clients connect via SSE for server-to-client messages and HTTP POST for client-to-server messages.

SSE transport is a prerequisite for the web UI (TASK-017) and enables remote MCP access.

## Acceptance Criteria

- [ ] `markplane-mcp --transport sse --port 3001` starts an HTTP server
- [ ] SSE endpoint (`/sse`) streams server-to-client JSON-RPC messages
- [ ] POST endpoint (`/message`) accepts client-to-server JSON-RPC messages
- [ ] Both transports share the same handler logic (no code duplication)
- [ ] CORS headers allow browser-based clients
- [ ] Stdio remains the default transport
- [ ] Health check endpoint (`/health`) for monitoring
- [ ] Integration tests for SSE transport

## Notes

Consider using `axum` or `actix-web` for the HTTP server — both are well-supported in the Rust ecosystem. The MCP spec's SSE transport is documented in the protocol specification. The key architectural decision is whether to make the HTTP server async (tokio) while keeping the core library sync. The handler dispatch logic in `main.rs` should be extracted into a transport-agnostic function that both stdio and SSE can call.

## References

- MCP transport specification: https://modelcontextprotocol.io/specification/2025-11-05/basic/transports
