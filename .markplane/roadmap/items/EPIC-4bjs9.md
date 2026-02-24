---
id: EPIC-4bjs9
title: "Web UI Polish & Features"
status: planned
priority: medium
started: null
target: null
tags: []
depends_on: []
---

# Web UI Polish & Features

## Objective

Bring the web UI from functional to polished. Fix drag-and-drop bugs that undermine confidence in the board, add the most-requested missing feature (item creation), and layer in quality-of-life improvements like undo, better search, and visual refinements. The goal is a web UI that feels complete enough to be someone's primary Markplane interface.

## Key Results

- [ ] Backlog drag-and-drop works correctly: board status changes, promote-to-board animation, in-progress drops
- [ ] Users can create tasks, epics, notes, and plans directly from the web UI
- [ ] Command palette finds actions (e.g. "Sync") while typing, not just in browse mode
- [ ] Status changes and other mutations offer toast-based undo within a short time window
- [ ] Detail sheet can be widened beyond 960px; inline code renders with proper monospace styling

## Notes

All tasks are frontend-only changes in `crates/markplane-web/ui/`. The backend API already supports everything needed (creation endpoint exists, mutations work). The DnD bugs ([[TASK-y5pi8]]) are the highest priority since they affect existing functionality. Item creation ([[TASK-jc69g]]) is the biggest workflow gap. The remaining tasks are incremental polish.
