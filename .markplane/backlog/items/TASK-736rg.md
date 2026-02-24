---
id: TASK-736rg
title: Extend link command to support cross-type entity linking
status: done
priority: high
type: enhancement
effort: medium
tags:
- cli
- mcp
- linking
epic: EPIC-6zdf4
plan: null
depends_on: []
blocks:
- TASK-t2hvn
assignee: null
position: a0
created: 2026-02-19
updated: 2026-02-21
---

# Extend link command to support cross-type entity linking

## Description

The `link` command and `markplane_link` MCP tool currently only support Task-to-Task linking (`blocks`/`depends_on`). All relationship fields — references to other tracked entities by ID — should be managed through the link system, not through the update command.

## Architectural Decision

Frontmatter fields split into **properties** (local to the item) and **relationships** (links to other entities). The `update` command/tool owns properties; the `link` command/tool owns all relationships. This separation exists because link mutations involve referential integrity, potential reciprocal writes, and cross-entity side effects.

**All relationship fields in the data model:**

| Field | Entity types | Cardinality | Reciprocal |
|-------|-------------|-------------|------------|
| `epic` | Task, Plan | single | no (derived via scan) |
| `plan` | Task | single | `plan.implements` |
| `depends_on` | Task, Epic | array | `blocks` (Task only) |
| `blocks` | Task | array | `depends_on` |
| `implements` | Plan | array | `task.plan` |
| `related` | Note | array | no |

## Gaps

**Currently supported:** Task `blocks`/`depends_on` (with reciprocal management)

**Missing:**
- Epic-to-Epic `depends_on`
- Task/Plan `epic` (set/clear a "belongs to" single-valued link)
- Task `plan` / Plan `implements` (bidirectional, set/clear + add/remove)
- Note `related` (add/remove)
- Unlinking (remove any relationship)

**Web UI:** Plan field on task detail sheets and implements field on plan detail sheets are read-only because the API doesn't handle bidirectional consistency. Extending the link tool would unblock making these editable.

## Acceptance Criteria

- [ ] `markplane link` supports all relationship types listed above
- [ ] Single-valued links (`epic`, `plan`) support set and clear semantics
- [ ] Array links support add and remove
- [ ] Bidirectional links (`plan`/`implements`, `blocks`/`depends_on`) update both sides atomically
- [ ] `markplane_link` MCP tool updated with same capabilities
- [ ] Works across all entity types (Task, Epic, Plan, Note)
- [ ] Web API supports link management (unblocks UI editing)

## Notes

- The `plan` field on Task is singular (one plan per task) while `implements` on Plan is plural — linking a second plan to a task would need to handle this constraint
- `epic` has no reciprocal field — epics discover their tasks via scanning, not via a stored field
- This task is the relationship counterpart to the update expansion task which handles properties only
