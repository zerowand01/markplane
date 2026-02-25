---
id: TASK-63ufm
title: Add keyboard shortcut to toggle Board/Backlog view
status: backlog
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

- Implementation lives in `use-keyboard-nav.ts` — add a page-aware handler, or create a local `useEffect` in the backlog page component
- The `changeView` callback in `backlog/page.tsx` already handles view switching and URL sync
- Consider showing the shortcut hint on the tab buttons (e.g. a subtle "V" badge) for discoverability
