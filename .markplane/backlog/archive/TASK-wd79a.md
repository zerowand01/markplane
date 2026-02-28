---
id: TASK-wd79a
title: Sort "Recently Done" section by completion date in backlog INDEX
status: done
priority: low
type: bug
effort: xs
epic: EPIC-6zdf4
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- sync
- index
position: Zz
created: 2026-02-12
updated: 2026-02-25
---

# Sort "Recently Done" section by completion date in backlog INDEX

## Description

The "Recently Done" section in `backlog/INDEX.md` inherits the `list_tasks()` sort order (priority → position → updated → ID), but semantically should be sorted by completion date. Most other sections are already correctly sorted — `list_tasks()` provides deterministic priority-based ordering that all kanban sections and roadmap task tables inherit.

**Scope note**: Original task was broader (sort all INDEX sections). Investigation found that `list_tasks()` already sorts deterministically, so only "Recently Done" actually needs a different sort. Downscoped accordingly.

## Acceptance Criteria

- [ ] "Recently Done" section sorted by `updated` date descending, then ID ascending as tiebreaker
- [ ] Verify sort order is stable across repeated syncs

## Notes

Two-line fix in `crates/markplane-core/src/index.rs`: add `.sort_by()` on the `recently_done` vec (around line 215) after the filter, sorting by `updated` desc then `id` asc.
