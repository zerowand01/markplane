---
id: TASK-027
title: Add concurrency safety for long-running MCP sessions
status: backlog
priority: medium
type: enhancement
effort: large
tags:
- core
- mcp
- reliability
epic: null
plan: null
depends_on: []
blocks: []
assignee: null
created: 2026-02-19
updated: 2026-02-19
---

# Add concurrency safety for long-running MCP sessions

## Description

The MCP server runs as a long-lived subprocess (minutes to days) while users concurrently run CLI commands against the same `.markplane/` files. Currently only `next_id()` uses file locking (fs2). All other read/write operations are unprotected, creating several race conditions.

## Risks Identified

1. **Non-atomic file writes** — `fs::write()` is not atomic. A CLI `sync` rewriting INDEX.md while MCP reads it could yield partial content.
2. **No locking beyond next_id()** — Concurrent `update_status` or `write_item` from CLI and MCP on the same task file will race (last write wins, silently).
3. **Position collision** — `append_position()` counts tasks by priority without locking. Two concurrent `add` operations with the same priority can produce duplicate position keys.

## Acceptance Criteria

- [ ] Atomic file writes (write to temp file + rename) for all item, config, INDEX.md, and .context/ writes
- [ ] Advisory file locking for write operations beyond just `next_id()`
- [ ] `append_position()` covered by the same lock scope as `next_id()` in `create_task()`
- [ ] No data loss when CLI and MCP write to the same file concurrently

## Notes

- These risks exist today with two separate binaries — they are not introduced by the CLI/MCP merge (TASK-027 context).
- Atomic writes via temp file + `fs::rename()` are the simplest improvement and cover most cases.
- Consider whether optimistic concurrency (check mtime before write) is worth the complexity.
