---
id: TASK-xvx5f
title: 'Rationalize MCP tool surface: polymorphic add, query-all-types, drop stale'
status: done
priority: high
type: enhancement
effort: medium
epic: EPIC-a5vs9
plan: null
depends_on:
- TASK-s6vjt
blocks:
- TASK-45fmi
related: []
assignee: null
tags:
- mcp
position: a0d
created: 2026-02-21
updated: 2026-02-24
---

# Rationalize MCP tool surface: polymorphic add, query-all-types, drop stale

## Description

### 1. Make `markplane_add` polymorphic

Add a `kind` parameter (`task`, `epic`, `note`) defaulting to `task` for backward compatibility. Dispatch to the appropriate core method:

- `task` (default): `create_task()` — accepts `type`, `priority`, `effort`, `epic`, `tags`
- `epic`: `create_epic()` — accepts `priority`
- `note`: `create_note()` — accepts `note_type`, `tags`

Type-specific params ignored for inapplicable kinds (same pattern as `markplane_update`). Update the tool description from "Create a new task" to "Create a new item".

`markplane_plan` stays separate — it's a creation + linking workflow, not just item creation.

### 2. Make `markplane_query` polymorphic

Add a `kind` parameter (`tasks`, `epics`, `plans`, `notes`) defaulting to `tasks`. Dispatch to the corresponding `list_*` method in core. Add `updated` to the JSON output for all kinds so staleness queries are self-serve.

### 3. Remove `markplane_stale`

Drop from `list_tools()` and `call_tool()` dispatch. Delete `handle_stale()`. CLI `stale` command stays. Staleness is covered by `markplane_query` (with `updated` in output) and `markplane_context --focus active-work`.

## Acceptance Criteria

- [ ] `markplane_add` creates epics and notes via `kind` param
- [ ] `markplane_query` lists epics, plans, and notes via `kind` param
- [ ] `markplane_query` output includes `updated` date
- [ ] `markplane_stale` removed from MCP (CLI kept)
- [ ] MCP integration tests updated
- [ ] Tool count: 17 → 16

## Notes

- Core already has `create_epic()`, `create_note()` — this is purely MCP wiring
- CLI keeps separate `add`, `epic`, `note` commands (CLI ergonomics differ from MCP)
- `markplane_check --orphans` is another CLI-vs-MCP gap but low priority, not in scope

## References
