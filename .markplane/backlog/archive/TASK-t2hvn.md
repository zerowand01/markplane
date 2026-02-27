---
id: TASK-t2hvn
title: Refactor web API to use core update and link methods
status: done
priority: high
type: enhancement
effort: medium
tags:
- web-ui
- core
- refactor
epic: EPIC-c5uem
plan: null
depends_on:
- TASK-m7i6q
- TASK-736rg
blocks: []
assignee: null
position: a0G
related: []
created: 2026-02-20
updated: 2026-02-21
---

# Refactor web API to use core update and link methods

## Description

The web API's update handlers (`update_task`, `update_epic`, `update_plan`, `update_note` in `serve.rs`) implement field updates inline with their own request structs, duplicating logic that now lives in core. After [[TASK-m7i6q]] (core update) and [[TASK-736rg]] (core link) are complete, the web API should be refactored to use the core methods — ensuring all three interfaces (CLI, MCP, web) share the same update and link logic.

## Current State

Each entity type has an `Update*Request` deserialization struct and an inline handler that does read-modify-write:
- `UpdateTaskRequest`: mixes properties (title, status, priority, effort, type, assignee, position, tags) with links (epic, plan, depends_on, blocks) and body
- `UpdateEpicRequest`: properties (title, status, priority, tags, started, target) and body
- `UpdatePlanRequest`: title, status, body only
- `UpdateNoteRequest`: title, status, tags, body only

## Acceptance Criteria

- [ ] `update_task` handler uses `Project::update_task()` for property fields
- [ ] `update_task` handler uses core link methods for relationship fields (epic, plan, depends_on, blocks)
- [ ] `update_epic` handler uses `Project::update_epic()` for property fields
- [ ] `update_epic` handler uses core link methods for depends_on
- [ ] `update_plan` handler uses `Project::update_plan()` for property fields
- [ ] `update_plan` handler uses core link methods for implements, epic
- [ ] `update_note` handler uses `Project::update_note()` for property fields
- [ ] `update_note` handler uses core link methods for related
- [ ] Body updates continue to use `Project::update_body()` or inline handling
- [ ] All existing web UI functionality works unchanged
- [ ] `Update*Request` structs can be simplified or removed if no longer needed

## Implementation Notes

- `diff_vec()` helper added to convert the web UI's replacement semantics (full desired array) to core's add/remove semantics. Lives in serve.rs as a private helper — the adapter layer between the two representations.
- `map_core_error()` exhaustively maps all `MarkplaneError` variants to HTTP status codes (no wildcard catch-all).
- Scalar link fields (epic, plan) use single Add call when switching values — core's `link_items()` handles old-value cleanup internally. Remove is only used for clearing.
- Array link fields (depends_on, blocks) use `diff_vec` to compute per-element Add/Remove calls. Removes execute before adds to avoid transient invalid states.
- `Update*Request` structs kept as-is — they serve a different purpose (JSON deserialization with Option/replacement arrays) than core structs (Patch/add-remove). The handler is the adapter.
- Epic/plan/note update handlers don't handle link fields because those request types never included them — the web UI already routes link changes through `POST /api/link`.
- 3-agent review team verified: no bugs, confirmed snapshot-after-property-update is safe (update_task only modifies property fields, not link fields), and no frontend breakage from error code changes.

## References

- [[TASK-m7i6q]] — core update methods (properties)
- [[TASK-736rg]] — core link methods (relationships)
