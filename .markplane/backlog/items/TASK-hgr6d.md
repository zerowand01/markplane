---
id: TASK-hgr6d
title: Add high-level move/reorder MCP tool for task positioning
status: backlog
priority: high
type: enhancement
effort: small
tags:
- mcp
- positioning
epic: null
plan: null
depends_on: []
blocks: []
assignee: null
position: a0l
created: 2026-02-20
updated: 2026-02-20
---

# Add high-level move/reorder MCP tool for task positioning

## Description

AI agents currently cannot reorder tasks within a priority group without manually computing fractional-indexing position keys. The existing `markplane_update` tool accepts a raw `position` string, but to use it the agent must: (1) query all tasks in the target priority group, (2) inspect their position keys, (3) compute a valid fractional index that sorts correctly. The web UI handles this transparently via `generateKeyBetween` from the `fractional-indexing` library during drag-and-drop.

Add a new `markplane_move` MCP tool that expresses positioning intent directly — "move to top," "move before TASK-X," "move after TASK-X" — and handles the fractional-indexing math server-side. This mirrors what the web UI does on drag-and-drop but makes it accessible to AI agents without requiring them to understand the position key system.

The tool should support at minimum:
- `top` / `bottom` — move to the top or bottom of the item's current priority group
- `before: ID` — position immediately before a specific task
- `after: ID` — position immediately after a specific task

The core already has fractional-indexing logic (used by `normalize_positions()`). The new tool queries the relevant tasks, computes the position key, and writes it to frontmatter — same as the web UI's PATCH endpoint but with a higher-level interface.

A corresponding CLI command (`markplane move TASK-5wph3 --to top`) would also be valuable but is secondary to the MCP tool.

## Acceptance Criteria

- [ ] New `markplane_move` MCP tool accepts an item ID and a positioning directive (top, bottom, before ID, after ID)
- [ ] Tool computes the correct fractional-indexing position key and updates the item's frontmatter
- [ ] Errors clearly when the target ID doesn't exist or the items are in different priority groups
- [ ] Existing tests pass, new tool has test coverage

## Notes

Existing code to build on:
- `normalize_positions()` in core — already does fractional-indexing for bulk initialization
- Web UI: `crates/markplane-web/ui/src/app/backlog/page.tsx` lines 698-714 — `generateKeyBetween` usage
- Serve endpoint: `crates/markplane-cli/src/commands/serve.rs` — PATCH handler already accepts `position`
- MCP tool registration: `crates/markplane-cli/src/mcp/` — follow existing tool patterns

## References
