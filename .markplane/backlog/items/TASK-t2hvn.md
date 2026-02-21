---
id: TASK-t2hvn
title: Refactor web API to use core update and link methods
status: backlog
priority: high
type: enhancement
effort: medium
tags:
- web-ui
- core
- refactor
epic: null
plan: null
depends_on:
- TASK-m7i6q
- TASK-736rg
blocks: []
assignee: null
position: a0G
created: 2026-02-20
updated: 2026-02-20
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

## Notes

- This is intentionally deferred until both dependencies land — doing it in one sweep avoids an awkward partial refactor where properties use core but links are still inline
- The web API uses replacement semantics for arrays (tags, depends_on, blocks) because the UI round-trips the full state. The core update uses add/remove semantics. The handler will need to diff current vs requested to produce the right add/remove calls, or core could also expose a replacement-style API for the web use case

## References

- [[TASK-m7i6q]] — core update methods (properties)
- [[TASK-736rg]] — core link methods (relationships)
