---
id: TASK-ksw5c
title: "Debounced auto-sync in server mode after API mutations and file changes"
status: draft
priority: someday
type: enhancement
effort: medium
tags: ["sync", "serve", "architecture"]
epic: null
plan: null
depends_on: []
blocks: []
assignee: null
position: a2
created: 2026-02-20
updated: 2026-02-20
---

# Debounced auto-sync in server mode after API mutations and file changes

## Description

Add a `SyncScheduler` to `markplane serve` that automatically regenerates derived files (INDEX.md, .context/) after API mutations and file changes, using debounced timers to avoid excessive I/O.

Currently not needed because:
- The web UI reads source files directly via REST API, not derived files
- INDEX.md and .context/ are gitignored, so GitHub browsing doesn't rely on them being fresh
- MCP runs as a separate process with its own `markplane_sync` tool
- Startup sync on `serve`, `mcp`, and `init` covers the common case

## When This Becomes Needed

- If the web UI adds a dashboard or view that reads .context/ summaries for project-level insights
- If a future integration (e.g., GitHub Actions, CI) needs fresh derived files after every change
- If simultaneous `markplane serve` + `markplane mcp` usage needs derived files kept in sync without manual `markplane sync` calls
- If INDEX.md rendering is added to the web UI (e.g., a rendered markdown navigation view)

## Proposed Design

- `SyncTrigger` enum: `ApiMutation` (2s debounce) / `FileChange` (10s debounce)
- `SyncScheduler` task: listens on `mpsc::Receiver<SyncTrigger>`, maintains a deadline
  - Each `ApiMutation` resets deadline to `now + 2s`
  - Each `FileChange` sets deadline to `now + 10s` only if no shorter deadline pending
  - When deadline expires: run `project.sync_all()`, broadcast `{"type":"sync_completed"}` via WebSocket
- Store `SyncScheduler` (containing the `mpsc::Sender`) in `AppState`
- API mutation handlers send `SyncTrigger::ApiMutation` after successful writes
- File watcher sends `SyncTrigger::FileChange` for source file modifications
- Filter INDEX.md and .context/ changes in file watcher (derived files, skip them)

## References

- Originated from [[TASK-skq58]] Step 7d (deferred as unnecessary for current architecture)
