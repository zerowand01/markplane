---
id: TASK-jejpq
title: Wrap sync I/O in spawn_blocking for async handlers
status: backlog
priority: medium
type: enhancement
effort: medium
epic: null
plan: null
depends_on:
- TASK-ns57e
blocks: []
related: []
assignee: null
tags:
- web-server
- async
- pre-release
position: a1
created: 2026-03-02
updated: 2026-03-02
---

# Wrap sync I/O in spawn_blocking for async handlers

## Description

**Blocking synchronous I/O in async web handlers** (Medium)

Every API handler in `crates/markplane-cli/src/commands/serve.rs` performs synchronous `fs::read_to_string()` and `fs::write()` directly on tokio worker threads. Under concurrent requests, this blocks the async runtime. No `spawn_blocking()` usage exists anywhere in the codebase.

For a localhost single-user tool, tokio's default thread pool handles this adequately. This becomes a problem at higher concurrency or if the server is exposed beyond localhost.

This is a cross-cutting change that wraps existing I/O calls — mechanical but touches many functions. Should be done after [[TASK-ns57e]] (write path safety) so that `spawn_blocking` wraps the final atomic write implementation.

## Acceptance Criteria

- [ ] All file I/O in `serve.rs` handlers wrapped in `tokio::task::spawn_blocking()`
- [ ] No synchronous `fs::` calls on tokio worker threads
- [ ] All existing tests pass
- [ ] Web server remains functional under concurrent requests

## References

- Source: Pre-release audit (2026-03-02)
- Depends on: [[TASK-ns57e]] (write path safety)
