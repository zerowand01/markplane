---
id: TASK-2tags
title: Add concurrency safety for long-running MCP sessions
status: cancelled
priority: medium
type: enhancement
effort: large
tags:
- core
- mcp
- reliability
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
assignee: null
position: a7
related: []
created: 2026-02-19
updated: 2026-02-23
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

## Cancellation Reasoning

This task was written around the old `fs2` file-locking architecture, which was removed when the ID system switched from sequential counters to random 5-char alphanumeric IDs. The premise — "currently only `next_id()` uses file locking (fs2)" — is no longer accurate; there is zero file locking in the codebase today.

The remaining theoretical risks were re-evaluated:

1. **Atomic writes (temp + rename)** don't address the actual problem. The main risk is lost updates from read-modify-write cycles (two processes read, modify different fields, last writer wins). Atomic writes only protect against partial reads during the write syscall — a microsecond window for sub-1KB markdown files.
2. **Advisory file locking** adds a dependency and complexity for a rare scenario (CLI and MCP writing the *same* file at the *same* instant). The consequence of the worst case is a lost field update on a git-tracked markdown file — trivially noticed via `git diff` and easily recovered.
3. **Position collisions** produce cosmetic ordering issues only, already fixable with `markplane sync --normalize`.

New file creation already uses `File::create_new()` (O_CREAT | O_EXCL), which is atomic and prevents duplicate IDs. The file-based, git-tracked design provides inherent recoverability that makes additional concurrency machinery over-engineering.
