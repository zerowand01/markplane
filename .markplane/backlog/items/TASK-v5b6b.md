---
id: TASK-v5b6b
title: Add file watching for real-time MCP updates
status: backlog
priority: low
type: feature
effort: medium
tags:
- mcp
epic: EPIC-z8tdz
plan: null
depends_on: []
blocks: []
assignee: null
position: a5
created: 2026-02-10
updated: 2026-02-10
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

## Notes

The `notify` crate (already mentioned in CLAUDE.md as a planned dependency) provides cross-platform file watching. Use `notify::RecommendedWatcher` with debouncing. The main complexity is mapping file paths back to MCP resource URIs (e.g., `.markplane/backlog/items/TASK-eduur.md` → `markplane://task/TASK-eduur`). File watching requires an async runtime or a background thread since the MCP server's main loop reads stdin synchronously.
