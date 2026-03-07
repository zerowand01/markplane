---
id: TASK-p2tub
title: Data consistency and cross-reference integrity
status: done
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

- [x] `create_plan` validates `task_id` format before writing the plan file
- [x] `create_plan` returns an error if reciprocal linking fails (not just stderr warning)
- [x] Workflow update validates or migrates tasks with removed statuses
- [x] `archive_item` warns about or cleans inbound references
- [x] `find_orphans` considers archived items as reference sources
- [x] Graph edges deduplicated for reciprocal relationships
- [x] Circular dependency detection exists (in `link_items` and/or `check`)
- [x] All existing tests pass

## Implementation Notes

**Cycle detection** — `has_path()` BFS in `links.rs` prevents new cycles at link time; `detect_cycles()` iterative DFS in `references.rs` reports existing cycles via `check`. Both `check` CLI and MCP `markplane_check` report cycles.

**create_plan validation** — Validates task_id format (must be TASK prefix), verifies task exists (404), and rolls back the plan file if `link_items()` fails.

**Workflow validation** — `PATCH /api/config` returns 409 Conflict with affected task IDs if workflow changes would strand active tasks using removed statuses.

**Archive cleanup** — `archive_item()` calls `find_inbound_references()` before moving the file, then does best-effort removal of inbound refs (blocks, depends_on, related, plan, epic) from active items.

**Orphan detection** — `find_orphans()` now scans `archive/*.md` files for references (but doesn't treat archived items themselves as orphan candidates).

**Graph dedup** — Skips `depends_on` edges (covered by `blocks`) and `task.plan` edges (covered by `plan.implements`).

**Files changed**: `links.rs`, `references.rs`, `project.rs`, `lib.rs`, `serve.rs`, `check.rs`, `mcp/tools.rs`, plus docs updates to `cli-reference.md`, `mcp-setup.md`, `architecture.md`, `web-ui-guide.md`. 441 tests passing, clippy clean.

## References

- Source: Pre-release audit (2026-03-02)
