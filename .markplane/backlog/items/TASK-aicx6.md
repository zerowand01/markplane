---
id: TASK-aicx6
title: Add universal bidirectional related links for all item types
status: planned
priority: medium
type: enhancement
effort: medium
tags:
- core
- cli
- web
epic: null
plan: null
depends_on: []
blocks: []
assignee: null
position: a4
created: 2026-02-27
updated: 2026-02-27
---

# Add universal bidirectional related links for all item types

## Description

Currently `related` links only work on Notes (one-directional, Note→any). There's no way to express "these two tasks are connected" without using `blocks`/`depends_on`, which implies ordering/dependency that may not exist.

The `related` relationship is purely informational — "these things are connected, you should know about the other one." Unlike `blocks`, `epic`, or `plan`, it drives no system behavior (blocking logic, progress tracking, kanban placement). This makes it the right candidate for a universal, unconstrained link type.

### Design decisions

- **Any→Any**: No prefix constraints. Task↔Task, Task↔Epic, Epic↔Epic, Plan↔Note, etc.
- **Bidirectional**: If A is related to B, B shows related to A. Both items get updated.
- **No behavior attached**: Never affects blocking, progress, kanban, or status. Purely navigational context.
- **`related: Vec<String>`** frontmatter field on all four types (Note already has it; add to Task, Epic, Plan).

### Key use cases

- Task↔Task: two tasks in the same area, not blocking each other
- Task↔Epic (non-parent): "I'm in Epic A but touches Epic B's concerns"
- Epic↔Epic: overlapping scope without dependency ordering
- Note↔anything: already works one-way, becomes bidirectional

## Acceptance Criteria

### Core
- [ ] `Task`, `Epic`, and `Plan` models have `related: Vec<String>` field
- [ ] `link_items()` with `LinkRelation::Related` works for any item type pair
- [ ] Related links are bidirectional — both items' `related` fields are updated
- [ ] Unlinking removes from both sides
- [ ] Idempotent — adding an existing link is a no-op
- [ ] Self-link still rejected

### CLI
- [ ] `markplane link TASK-x TASK-y -r related` works (no CLI changes expected — pass-through)
- [ ] `markplane show` displays related items for all types

### MCP
- [ ] `markplane_link` with `relation: "related"` works for any item pair (no MCP changes expected)
- [ ] `markplane_show` output includes related field for all types

### Web UI
- [ ] Related items displayed in detail sheets for all item types
- [ ] Related items can be added/removed from detail sheets

### Serialization
- [ ] Existing items without `related` field deserialize with empty vec (backward compatible)
- [ ] Templates updated to include `related: []` field

## Notes

The core change is almost entirely in `links.rs` — widen the `LinkRelation::Related` match arm to dispatch on both `from_prefix` and `to_prefix` (same pattern as `DependsOn` handles Task vs Epic). CLI and MCP are thin pass-throughs that already accept `related` as a valid relation string.

## References
