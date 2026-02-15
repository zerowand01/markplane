---
id: TASK-020
title: Sort items in generated INDEX.md sections
status: draft
priority: medium
type: bug
effort: small
tags:
- sync
- index
epic: null
plan: null
depends_on: []
blocks: []
assignee: null
position: a1
created: 2026-02-12
updated: 2026-02-12
---

# Sort items in generated INDEX.md sections

## Description

`scan_directory()` returns items in filesystem order (via `glob::glob`), which is non-deterministic (APFS directory entry ordering). The `generate_backlog_index()` function iterates items without sorting, so sections like "Recently Done" display items in arbitrary order. TASK-019 appeared between TASK-005 and TASK-006 after sync because of this.

All INDEX.md sections should sort items deterministically — by date, priority, ID, or a combination appropriate to each section.

## Acceptance Criteria

- [ ] "Recently Done" section sorted by `updated` date descending, then ID ascending as tiebreaker
- [ ] Kanban status sections (In Progress, Planned, Backlog, Draft) sorted by priority descending, then ID ascending
- [ ] Roadmap INDEX epic task tables sorted consistently (by status rank, then ID)
- [ ] Verify sort order is stable across repeated syncs

## Notes

The fix is in `crates/markplane-core/src/index.rs`. The `recently_done` vec (and other filtered vecs) need a `.sort_by()` call before rendering. Consider whether `list_tasks()` in `query.rs` should return pre-sorted results so all consumers benefit.
