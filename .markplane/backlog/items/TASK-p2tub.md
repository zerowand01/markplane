---
id: TASK-p2tub
title: Data consistency and cross-reference integrity
status: backlog
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
updated: 2026-03-02
---

# Data consistency and cross-reference integrity

## Description

Several issues where cross-item relationships can become inconsistent, plus two feature gaps in relationship integrity checking.

**`create_plan` silently creates inconsistent state** (High)
At `serve.rs:1300-1330`, when creating a plan with `task_id`, if `link_items()` fails the plan has `implements: [task_id]` but the task has no `plan` field. The error is only logged to stderr — HTTP 201 is returned. Return an HTTP error or roll back. Validate `task_id` format before writing.

**Workflow update doesn't migrate existing statuses** (Medium)
At `serve.rs:1107-1133`, `PATCH /api/config` allows replacing the task workflow. Existing tasks with old status strings become locked — `validate_task_status()` rejects updates. Validate no active tasks use statuses being removed, or auto-migrate.

**Archive doesn't clean inbound references** (Medium)
At `project.rs:976-993`, `archive_item()` moves the file but doesn't update items that reference it. Active items retain `blocks: [ARCHIVED-ID]` as ghost references. At minimum warn about inbound references.

**Orphan detection misses archived items** (Low)
At `references.rs:140-187`, `find_orphans()` only scans `items/`, not `archive/`. An active item referenced only by an archived item appears as an orphan.

**Duplicate graph edges for reciprocal relationships** (Low)
At `serve.rs:2282-2309`, if A blocks B, both A's `blocks` and B's `depends_on` produce separate edges. Deduplicate by normalizing edge direction.

**No circular dependency detection** (Feature Gap)
No check for cycles in `blocks`/`depends_on` chains. A cycle makes all involved tasks permanently "blocked" in dashboards. Add cycle detection in `link_items()` or `check` command.

**No archive cascade cleanup** (Feature Gap)
No `--cascade` flag when archiving to optionally clean up references in other items.

## Acceptance Criteria

- [ ] `create_plan` returns an error if reciprocal linking fails (not just stderr warning)
- [ ] Workflow update validates or migrates tasks with removed statuses
- [ ] `archive_item` warns about or cleans inbound references
- [ ] `find_orphans` considers archived items as reference sources
- [ ] Graph edges deduplicated for reciprocal relationships
- [ ] Circular dependency detection exists (in `link_items` and/or `check`)
- [ ] All existing tests pass

## References

- Source: Pre-release audit (2026-03-02)
