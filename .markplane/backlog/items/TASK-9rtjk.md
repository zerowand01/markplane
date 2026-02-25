---
id: TASK-9rtjk
title: Add drag-and-drop for epics between Now/Next/Later columns on roadmap page
status: backlog
priority: medium
type: enhancement
effort: medium
tags:
- web-ui
- dnd
epic: EPIC-4bjs9
plan: null
depends_on: []
blocks: []
assignee: null
position: a8
created: 2026-02-24
updated: 2026-02-24
---

# Add drag-and-drop for epics between Now/Next/Later columns on roadmap page

## Description

The roadmap page displays epics in a kanban-style Now/Next/Later layout, but there's no way to move epics between columns without opening the detail sheet and using the status dropdown. Drag-and-drop is the natural interaction for a kanban board and is already implemented for tasks on the backlog page.

The existing `@dnd-kit` infrastructure and `useUpdateEpic` mutation can be reused — no new API endpoints needed. Dropping an epic on a column should update its status to match that column (now/next/later).

## Acceptance Criteria

- [ ] Epics can be dragged between Now, Next, and Later columns on the roadmap page
- [ ] Dropping an epic on a column updates its status via the existing PATCH endpoint
- [ ] Drag overlay shows the epic card being dragged (consistent with backlog kanban)
- [ ] Optimistic update — column changes are reflected immediately before server confirms
- [ ] Done section remains non-droppable (use "Mark done" action instead)

## Notes

- Backlog kanban DnD implementation in `crates/markplane-web/ui/src/app/backlog/page.tsx` (lines ~367–495) is the reference pattern
- No within-column reordering needed initially — epics don't have a `position` field
- `useUpdateEpic` hook already handles `{ status }` mutations

## References
