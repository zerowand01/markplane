---
id: TASK-ict2n
title: Add internal Fibonacci point mapping to effort sizes
status: backlog
priority: low
type: enhancement
effort: small
tags:
- core
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
assignee: null
position: Zz
related: []
created: 2026-02-26
updated: 2026-02-26
---

# Add internal Fibonacci point mapping to effort sizes

## Description

The t-shirt effort sizes (xs, small, medium, large, xl) are the right user-facing system for Markplane — human-readable in YAML frontmatter and fast to estimate. However, the current implementation treats effort as a pure label with no numeric semantics, which means we can't compute aggregate metrics like total effort for an epic, completed effort per period, or effort distribution.

Add an internal Fibonacci point mapping so the system can reason about effort numerically while keeping the t-shirt vocabulary as the user-facing interface. The scale remains fixed (not configurable) — the opinionated 5-level default is a feature.

### Mapping

| Size | Points |
|------|--------|
| xs | 1 |
| small | 2 |
| medium | 3 |
| large | 5 |
| xl | 8 |

## Acceptance Criteria

- [ ] `Effort::points()` method returns the Fibonacci point value for each size
- [ ] Epic effort summary in INDEX.md shows total points and breakdown by status (e.g., `12/19 points completed`)
- [ ] `.context/summary.md` includes effort totals for active epics
- [ ] Web API `/api/summary` includes epic-level effort aggregation
- [ ] Web UI displays effort totals on epic cards or detail view
- [ ] Existing effort display (badges, tables, detail sheets) unchanged — points are supplementary, not replacement

## Notes

The `EFFORT_RANK` mapping already exists in the web UI (`backlog-content.tsx:68-74`) for sorting purposes. This task adds the Fibonacci mapping in Rust and uses it for aggregation. The web UI rank values (0-4) are for sort order; the Fibonacci values (1,2,3,5,8) are for effort aggregation — these are distinct concerns.

This is independent of the configurable types/statuses work and can be done at any time.
