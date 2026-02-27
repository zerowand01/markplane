---
id: TASK-qgxin
title: Remove update_body() and fold body into typed update methods
status: done
priority: medium
type: enhancement
effort: small
tags:
- cli
- core
- web
epic: null
plan: null
depends_on: []
blocks:
- TASK-xwfhp
assignee: null
position: Zw
related: []
created: 2026-02-26
updated: 2026-02-26
---

# Remove update_body() and fold body into typed update methods

## Description

`update_body()` on `Project` is a standalone method that reads an item, replaces `doc.body`, stamps `updated`, and writes it back. Its only callers are the 4 Web API PATCH handlers in `serve.rs`, where it runs as a separate read-modify-write cycle *after* the typed update method (`update_task()` etc.) has already done its own read-modify-write on the same file. This means a PATCH with both property and body changes does two full serialize-deserialize round-trips on the same file, with both independently setting `updated` to today.

The body is just another field on `MarkplaneDocument<T>`. It should be updated in the same pass as properties.

## Acceptance Criteria

- [ ] `body: Option<String>` field added to `TaskUpdate`, `EpicUpdate`, `PlanUpdate`, `NoteUpdate`
- [ ] Typed update methods (`update_task`, `update_epic`, `update_plan`, `update_note`) set `doc.body` when `body` is `Some`
- [ ] `update_body()` method removed from `Project`
- [ ] `serve.rs` PATCH handlers pass body through the typed update struct instead of calling `update_body()` separately
- [ ] `update_body` unit tests migrated to test body via the typed update methods
- [ ] CLI `markplane update` and MCP `markplane_update` unaffected (they don't touch body)
- [ ] All tests pass, clippy clean

## Notes

- `update_status()` is a separate convenience method used by CLI/MCP for quick status transitions — it's fine as-is and out of scope.
- `link_items()` is inherently multi-file (touches both sides of a relationship) so it can't be folded into a single pass. Out of scope.
- The initial read in each PATCH handler (for diffing tags/links against current state) is also separate and necessary — it provides the baseline for computing diffs before any mutations happen.
