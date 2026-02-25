---
id: TASK-y5pi8
title: Fix backlog drag-and-drop bugs
status: done
priority: high
type: bug
effort: small
tags:
- web-ui
epic: EPIC-4bjs9
plan: null
depends_on: []
blocks: []
assignee: null
position: a0x
created: 2026-02-23
updated: 2026-02-25
---

# Fix backlog drag-and-drop bugs

## Description

Two drag-and-drop bugs on the backlog page need fixing.

**Bug 1 — Promote-to-board animation glitch:** When dragging a backlog item to the "Drop here to move to Board" zone, the status change works but the animation shows the card sliding back to its original position before disappearing. The `DragOverlay` disappears on drop, and the original card (rendered at 0.5 opacity during drag) briefly snaps back before the optimistic update removes it from the list. The `skipDropAnimation` and custom `animateLayoutChanges` logic in the backlog list view may need adjustment — either hide the original card entirely during drag or ensure the optimistic cache update fires before the drag-end animation completes.

**Bug 2 — Drag to In Progress doesn't work:** On the board (kanban) view, dragging a card to the "In Progress" column has no effect. The `handleDragEnd` in `KanbanView` determines target status by checking if dropped on a column header vs. a card. Likely causes: the In Progress column's droppable ID doesn't map correctly to the `in-progress` status string, or `closestCorners` collision detection resolves to a card within the column (whose status is already `planned`) rather than the column itself.

Key file: `crates/markplane-web/ui/src/app/backlog/page.tsx`

## Acceptance Criteria

- [ ] Dragging a backlog item to the promote zone shows a clean transition (no snap-back animation)
- [ ] Dragging a card to the In Progress column on the board changes its status to `in-progress`
- [ ] Existing drag-to-Planned and drag-to-Done behavior still works correctly

## References
