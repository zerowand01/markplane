---
id: TASK-63ufm
title: Add keyboard shortcut to toggle Board/Backlog view
status: done
priority: low
type: enhancement
effort: xs
tags:
- web-ui
- keyboard-nav
epic: EPIC-4bjs9
plan: null
depends_on: []
blocks: []
assignee: null
position: ZzG
created: 2026-02-25
updated: 2026-02-25
---

# Add keyboard shortcut to toggle Board/Backlog view

## Description

The backlog page has two views — Board (kanban) and Backlog (priority-grouped list) — but switching between them requires clicking the tab. Add a `v` keyboard shortcut to toggle between the two views. This follows the existing modifier-free shortcut convention (`g+letter` chords, `?` for command palette) and reuses the `isInputFocused()` guard in `use-keyboard-nav.ts`.

Precedent: Linear uses `Cmd+B` for board toggle; Jira uses number keys. We chose a single unmodified `v` key ("view") because it's consistent with our chord-free shortcuts, avoids modifier conflicts (e.g. `Ctrl+B` = bold in TipTap), and can later be upgraded to a chord prefix (`v+b`, `v+l`) if more view modes are added.

## Acceptance Criteria

- [ ] Pressing `v` on the backlog page toggles between Board and Backlog views
- [ ] Shortcut is suppressed when an input, textarea, or contenteditable element is focused
- [ ] URL query param `?view=` updates to match (consistent with clicking the tab)
- [ ] Shortcut is a no-op on pages that don't have a view toggle

## Notes

- Implemented as a local `useEffect` in `BacklogContent` (not in the global `use-keyboard-nav.ts`) — keeps the shortcut page-scoped
- `isInputFocused()` was exported from `use-keyboard-nav.ts` for reuse
- Discoverability: decided against inline `<kbd>` badge (visual clutter) and tooltip (distracting on hover). Shortcut will be documented in `docs/web-ui-guide.md` and surfaced via [[TASK-4wi4p]] (docs viewer)
- Also fixed two stale entries in `docs/web-ui-guide.md` (`g then e`, `g then s`) that didn't exist in code
