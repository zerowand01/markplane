---
id: TASK-7jei5
title: Add Streamable HTTP transport for MCP server
status: backlog
priority: someday
type: feature
effort: medium
tags:
- mcp
epic: EPIC-z8tdz
plan: null
depends_on: []
blocks: []
assignee: null
position: a6
created: 2026-02-10
updated: 2026-02-23
---

# Add Streamable HTTP transport for MCP server

## Description

The MCP server currently only supports stdio transport — it reads JSON-RPC from stdin and writes to stdout. This works for desktop clients like Claude Code and Cursor that spawn the server as a subprocess, but doesn't support web-based clients or remote connections.

The MCP spec (2025-03-26) defines **Streamable HTTP** as the recommended remote transport, replacing the now-deprecated SSE transport. Streamable HTTP uses a single bidirectional POST endpoint with Content-Type negotiation (`application/json` or `text/event-stream`), which is simpler and more secure than the old SSE + POST split.

## Assessment

**Current merit: Low.** Markplane is a local-first tool. The web UI already provides full HTTP access to all project data via REST + WebSocket — it doesn't use MCP. Stdio works for all current desktop MCP clients. The only scenario where Streamable HTTP adds value is if a **remote MCP client** (e.g., Claude.ai web, a hosted AI agent) needs to connect to a local or remote Markplane instance via the MCP protocol specifically.

**When to implement:**

- **Remote MCP clients become mainstream.** If Claude.ai, ChatGPT, or similar web-based tools start connecting to user-hosted MCP servers via Streamable HTTP, this enables Markplane to participate.
- **Multi-machine workflows.** If users want to run `markplane mcp` on a server and connect from multiple clients remotely.
- **MCP becomes the primary API.** If the MCP tool interface proves more useful than the REST API for integrations, consolidating on MCP + Streamable HTTP could simplify the architecture.

**Why keep in backlog:** The transport abstraction is straightforward — handler logic in `tools.rs` and `resources.rs` is already transport-independent. The web server already uses axum + tokio, so the HTTP infrastructure exists. When demand appears, this is a bounded piece of work.

## Acceptance Criteria

- [ ] `markplane mcp --transport http --port 3001` starts an HTTP server
- [ ] Single POST endpoint handles bidirectional JSON-RPC per Streamable HTTP spec
- [ ] Content-Type negotiation: `application/json` for single responses, `text/event-stream` for streaming
- [ ] Both transports share the same handler logic (no code duplication)
- [ ] CORS headers allow browser-based clients
- [ ] Stdio remains the default transport
- [ ] Health check endpoint (`/health`) for monitoring
- [ ] Integration tests for Streamable HTTP transport

## Implementation Approach

Use axum (already a dependency) for the HTTP server. Extract `handle_request()` from the stdio loop into a transport-agnostic dispatcher. The stdio and HTTP transports become thin wrappers that read requests, call the dispatcher, and write responses. The core library stays synchronous — use `tokio::task::spawn_blocking` for disk I/O in the async context.

## References

- MCP Streamable HTTP transport spec: https://modelcontextprotocol.io/specification/2025-03-26/basic/transports
- Background on SSE deprecation: https://blog.fka.dev/blog/2025-06-06-why-mcp-deprecated-sse-and-go-with-streamable-http/
