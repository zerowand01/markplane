---
id: TASK-4qegx
title: Filter command palette actions during search
status: planned
priority: medium
type: enhancement
effort: small
tags:
- web-ui
epic: EPIC-4bjs9
plan: null
depends_on: []
blocks: []
assignee: null
position: ZzV
created: 2026-02-23
updated: 2026-02-25
---

# Filter command palette actions during search

## Description

The command palette (Cmd+K) has a "Sync project" action and navigation shortcuts, but these only appear in browse mode (when the search query is empty or < 2 chars). Once the user starts typing, the palette switches to server-side item search exclusively and hides all actions. Typing "sync" should still show the Sync action alongside any matching items.

The fix is to keep the actions and navigation groups visible in search mode, filtering them client-side against the query string. The `cmdk` library handles filtering natively if items remain in the command list — the current implementation conditionally renders the groups based on query length, which is what hides them.

Key file: `crates/markplane-web/ui/src/components/layout/command-palette.tsx`

## Acceptance Criteria

- [ ] Actions (Sync, navigation shortcuts) are filtered client-side and shown alongside server results
- [ ] Typing "sync" shows the "Sync project" action
- [ ] Typing a navigation target (e.g. "back" for Backlog) shows the navigation shortcut
- [ ] Item search results still work as before when query >= 2 chars

## References
