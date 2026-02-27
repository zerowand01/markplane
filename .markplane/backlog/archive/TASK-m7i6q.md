---
id: TASK-m7i6q
title: Expand update command/tool to support all property fields
status: done
priority: high
type: enhancement
effort: large
tags:
- cli
- mcp
- core
epic: EPIC-6zdf4
plan: null
depends_on: []
blocks:
- TASK-t2hvn
assignee: null
position: Zz
related: []
created: 2026-02-20
updated: 2026-02-21
---

# Expand update command/tool to support all property fields

## Description

The `markplane_update` MCP tool only exposes `status`, `priority`, `assignee`, `position`. Property fields like `effort`, `type`, `tags`, `title`, and Epic date fields (`started`/`target`) can't be set via MCP or CLI. There is no unified CLI `update` command — field changes are scattered across specialized commands (`assign`, `tag`, `status`). Update logic should live in core with typed per-entity update structs.

## Architectural Decision

Frontmatter fields are categorized as **properties** (local to the item) vs **relationships** (links to other entities). This task handles properties only. Relationship fields (`epic`, `plan`, `depends_on`, `blocks`, `implements`, `related`) are handled by [[TASK-736rg]].

**Property fields by entity type:**

| Field | Task | Epic | Plan | Note |
|-------|------|------|------|------|
| `title` | Y | Y | Y | Y |
| `status` | Y | Y | Y | Y |
| `priority` | Y | Y | - | - |
| `effort` | Y | - | - | - |
| `type` | Y | - | - | Y |
| `assignee` | Y | - | - | - |
| `position` | Y | - | - | - |
| `tags` (add/remove) | Y | Y | - | Y |
| `started` | - | Y | - | - |
| `target` | - | Y | - | - |

## Acceptance Criteria

- [ ] Core: `Patch<T>` enum (`Unchanged`/`Clear`/`Set`) for clearable fields
- [ ] Core: Per-type update structs (`TaskUpdate`, `EpicUpdate`, `PlanUpdate`, `NoteUpdate`) with only valid fields
- [ ] Core: Generic `UpdateFields` struct + `Project::update_item()` dispatch by ID prefix
- [ ] Core: Field applicability validation (e.g., `effort` on Plan → error)
- [ ] Core: `apply_tag_changes()` helper for add/remove with dedup
- [ ] MCP: `markplane_update` tool schema expanded with all property fields
- [ ] MCP: `handle_update()` rewritten to use core `update_item()`
- [ ] CLI: New `markplane update` command with flags for all property fields
- [ ] CLI: `--clear-*` flags for clearable optional fields (assignee, position, started, target)
- [ ] Remove `assign` and `tag` CLI commands (subsumed by `update --assignee` and `--add-tag`/`--remove-tag`)
- [ ] Keep `status`, `start`, `done` CLI commands (workflow shortcuts, not pure field setters)
- [ ] Unit tests for all entity types, type mismatches, tag operations, clear operations
- [ ] MCP + CLI integration tests

## Notes

- Uses `Patch<T>` enum instead of `Option<Option<T>>` for readability
- Existing `update_status()` and `update_body()` convenience methods remain unchanged
- Web API refactoring deferred until after both core update and core link ([[TASK-736rg]]) exist — single clean sweep
- Tags use add/remove semantics (not replacement) since MCP/CLI callers may not know current state

## References

- [[TASK-736rg]] — companion task for relationship/link field management
