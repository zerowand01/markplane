---
id: BACK-010
title: Add Sprint/Iteration entity type
status: backlog
priority: low
type: feature
effort: large
tags:
- core
epic: EPIC-003
plan: null
depends_on: []
blocks: []
assignee: null
created: 2026-02-10
updated: 2026-02-10
---

# Add Sprint/Iteration entity type

## Description

Markplane currently has four entity types: epics (strategic goals), backlog items (work), plans (implementation details), and notes (research/ideas). There is no time-boxed grouping concept — no way to say "these 5 items are our focus for the next two weeks." A Sprint or Iteration entity would provide this, giving teams a way to plan capacity and track velocity over fixed time periods.

This was discussed as a future enhancement during design. Plans are the "how" for individual items; sprints are "when" — a time-boxed container that groups multiple backlog items into a focused work period.

## Acceptance Criteria

- [ ] New `SPRINT-NNN` ID prefix and `sprints/` directory
- [ ] Sprint entity with frontmatter: `id`, `title`, `status`, `start_date`, `end_date`, `items` (list of BACK IDs), `goals`
- [ ] Sprint statuses: `planning → active → completed → retrospective`
- [ ] `markplane sprint create "Sprint 1" --start 2026-02-10 --end 2026-02-24`
- [ ] `markplane sprint add SPRINT-001 BACK-003 BACK-005` to assign items to a sprint
- [ ] `markplane sprint show SPRINT-001` displays sprint with item statuses and progress
- [ ] Dashboard shows active sprint summary
- [ ] MCP tools for sprint management
- [ ] INDEX.md and context generation updated to include sprints

## Notes

Keep sprints optional — many solo developers and small teams prefer continuous flow over time-boxed iterations. The sprint entity should be additive, not required. Consider whether sprint items should be a frontmatter list on the sprint or a `sprint` field on each backlog item (or both, kept in sync). The design spec mentions this as a future capability.
