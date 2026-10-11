---
id: TASK-sqnz9
title: Fix React Compiler lint errors blocking the eslint-config-next 16.4 update
status: done
priority: medium
type: chore
effort: xs
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- web-ui
- dependencies
position: aA
created: 2026-10-11
updated: 2026-10-11
---

# Fix React Compiler lint errors blocking the eslint-config-next 16.4 update

## Description

Dependabot's npm minor/patch group PR bumps eslint-config-next from 16.3.6 to 16.4.0,
which upgrades eslint-plugin-react-hooks from 7.0.1 to 7.1.1. That version promotes two
React Compiler rules to errors, and five existing spots in the web UI trip them, so the
Linux CI check (the only one that runs `npm run lint`) fails on the Dependabot PR.

The offending patterns:
- `setState` called synchronously inside `useEffect` (`use-mobile.ts`, `inline-edit.tsx`,
  and the URL-to-state sync in the dashboard and backlog pages)
- a ref written during render (`graph-view.tsx`)

Fixing these on `master` first, rather than on the Dependabot branch, keeps Dependabot
able to rebase its PR and makes the fixes land regardless of the upgrade.

## Acceptance Criteria

- [x] `npm run lint` passes with eslint-plugin-react-hooks 7.1.1 (verified in a worktree
      of the Dependabot branch) and with the current 7.0.1.
- [x] `useIsMobile` still reports the viewport state and updates on resize.
- [x] Opening `?task=ID` in the URL still opens the task sheet on the dashboard and backlog,
      and closing it clears the param.
- [x] Inline title editing starts from the current saved value and keeps typed text
      across background refetches.
- [x] Graph focus view behaves the same as on master (checked `?focus=` with and without
      visible nodes).

## Notes

- `useIsMobile` now uses `useSyncExternalStore` over one shared `MediaQueryList`, reading
  `matches` for both the snapshot and the subscription so they cannot disagree.
- The dashboard and backlog no longer mirror the URL into state at all; the open task is
  derived from `searchParams` and open/close only update the URL, matching the plans,
  notes, and archive pages.
- `InlineEdit` no longer syncs its draft from props at all. The draft only matters while
  editing, so it is seeded from the saved value when editing starts; a background refetch
  cannot wipe typed text.
- `GraphCanvas` reads the current nodes from React Flow (`getNodes()`) inside its timer
  instead of mirroring them into a ref.
- After this merges, comment `@dependabot rebase` on the npm group PR.

## References

- Dependabot npm group PR #83
