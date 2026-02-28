---
id: TASK-v5b6b
title: Add file watching for real-time MCP updates
status: backlog
priority: someday
type: feature
effort: medium
epic: EPIC-z8tdz
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- mcp
position: a1V
created: 2026-02-10
updated: 2026-02-23
---

# Add file watching for real-time MCP updates

## Description

The MCP server currently reads `.markplane/` files on every request — there's no caching or change detection. The MCP spec supports `notifications/resources/updated` messages that servers can send to clients when resources change. By watching the filesystem for changes, the MCP server could proactively notify clients that project state has changed, enabling real-time dashboards and reducing stale data.

This becomes important when users edit markdown files directly (in their editor) while an MCP client is connected — the client should know the data changed without polling.

## Acceptance Criteria

- [ ] MCP server watches `.markplane/` directory for file changes using `notify` crate
- [ ] File changes trigger `notifications/resources/updated` for affected resources
- [ ] Debouncing prevents notification floods during rapid edits (e.g., 500ms debounce)
- [ ] Watches are scoped to relevant directories (backlog, roadmap, plans, notes, .context)
- [ ] Server handles watch errors gracefully (e.g., too many watchers on Linux)
- [ ] File watching is optional and can be disabled via config or flag
- [ ] Integration test verifies notification delivery after file modification

## Assessment

**Current merit: Low.** The MCP spec defines `notifications/resources/updated` but today's MCP clients (Claude Code, Cursor) don't visibly act on server-initiated notifications. Since `markplane-core` has no caching layer — it reads from disk on every request — queries already return fresh data. The "stale data" problem this solves doesn't exist for typical AI assistant usage.

**When to implement:**

- **MCP clients start consuming resource notifications.** Once Claude Code, Cursor, or other clients use `notifications/resources/updated` to refresh state, this becomes an immediate quick win.
- **A caching layer is added to core.** If `markplane-core` starts caching data in memory for performance, notifications become essential for cache invalidation. This task shifts from "nice to have" to "required."
- **A real-time MCP dashboard exists.** If any client builds a live view that subscribes to resource updates, file watching is the enabler.

**Why keep in backlog:** The feature is well-scoped, technically trivial (~100 lines), and the implementation pattern is already proven in the web server (`serve.rs` file watcher + `notify-debouncer-mini`). All dependencies are in `Cargo.toml`. It's a zero-cost option — ready to pick up whenever the demand side catches up.

## Implementation Approach

**Recommended: Queue-based (not async refactor).** Spawn a file watcher thread reusing the web server's `run_file_watcher()` pattern. Collect notifications in a `crossbeam::channel`, flush them between JSON-RPC responses in the existing sync MCP loop. No changes to `markplane-core` required.

## Notes

The `notify` crate provides cross-platform file watching. Use `notify::RecommendedWatcher` with debouncing. The main complexity is mapping file paths back to MCP resource URIs (e.g., `.markplane/backlog/items/TASK-eduur.md` → `markplane://task/TASK-eduur`). File watching requires a background thread since the MCP server's main loop reads stdin synchronously.
