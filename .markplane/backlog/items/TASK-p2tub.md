---
id: TASK-p2tub
title: Data consistency and cross-reference integrity
status: planned
priority: high
type: bug
effort: large
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- data-integrity
- references
- pre-release
position: a3t
created: 2026-03-02
updated: 2026-03-03
---

# Data consistency and cross-reference integrity

## Description

Several issues where cross-item relationships can become inconsistent, plus two feature gaps in relationship integrity checking.

**`create_plan` silently creates inconsistent state** (High)
At `serve.rs:1333-1369`, when creating a plan with `task_id`, `create_plan()` writes the plan file with `implements: [task_id]` at line 1342, then calls `link_items()` at lines 1348-1357 to set `task.plan`. If `link_items()` fails, the error is only logged via `eprintln!` — HTTP 201 is returned with the plan containing `implements: [task_id]` but the task having no `plan` field. Additionally, `task_id` format is not validated before `create_plan()` writes the file — an invalid ID results in a plan with a bogus `implements` entry. Return an HTTP error or roll back. Validate `task_id` format before writing.

**Workflow update doesn't migrate existing statuses** (Medium)
At `serve.rs:1172-1198`, `PATCH /api/config` allows replacing the task workflow wholesale at line 1197. Existing tasks with old status strings become locked — `validate_task_status()` (`project.rs:342`) rejects updates. Validate no active tasks use statuses being removed, or auto-migrate.

**Archive doesn't clean inbound references** (Medium)
At `project.rs:983-1000`, `archive_item()` moves the file via `fs::rename` but doesn't update items that reference it. Active items retain `blocks: [ARCHIVED-ID]` as ghost references. At minimum warn about inbound references.

**Orphan detection misses archived items** (Low)
At `references.rs:140-187`, `find_orphans()` uses `scan_pattern()` which resolves to `items/*.md` only — `archive/` is never scanned. An active item referenced only by an archived item falsely appears as an orphan.

**Duplicate graph edges for reciprocal relationships** (Low — UX, not data integrity)
At `serve.rs:2281-2294`, if A blocks B, both A's `blocks` and B's `depends_on` produce separate edges with different relation labels for the same underlying relationship. React Flow renders both as distinct visual edges between the same node pair. Deduplicate by normalizing edge direction.

**No circular dependency detection** (Feature Gap)
`link_items()` in `links.rs:156-327` has a self-link check but no transitive cycle detection. Creating A blocks B then B blocks A both succeed, producing a cycle that makes all involved tasks permanently "blocked" in dashboards. Add cycle detection in `link_items()` or `check` command.

**No archive cascade cleanup** (Feature Gap)
No `--cascade` flag when archiving to optionally clean up references in other items (related to archive inbound refs issue above).

## Acceptance Criteria

- [ ] `create_plan` validates `task_id` format before writing the plan file
- [ ] `create_plan` returns an error if reciprocal linking fails (not just stderr warning)
- [ ] Workflow update validates or migrates tasks with removed statuses
- [ ] `archive_item` warns about or cleans inbound references
- [ ] `find_orphans` considers archived items as reference sources
- [ ] Graph edges deduplicated for reciprocal relationships
- [ ] Circular dependency detection exists (in `link_items` and/or `check`)
- [ ] All existing tests pass

## References

- Source: Pre-release audit (2026-03-02)
