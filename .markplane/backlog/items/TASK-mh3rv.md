---
id: TASK-mh3rv
title: Backlog page scrolls horizontally when task titles are long
status: done
priority: medium
type: bug
effort: xs
epic: null
plan: null
depends_on: []
blocks: []
related:
- TASK-iw525
- TASK-svkm7
assignee: null
tags:
- web-ui
position: a8
created: 2026-10-10
updated: 2026-10-10
---

# Backlog page scrolls horizontally when task titles are long

## Description

On the Backlog tab of the backlog page, a project with long task titles makes the whole
page wider than the viewport. The epic chip and effort badge on each row sit past the
right edge, and the Sort control in the toolbar is clipped. Row titles use `truncate` and
should shrink to fit, but they never get the chance because the shell's content area grows
to the content's intrinsic width instead.

Root cause: `SidebarInset` (the `<main>` from shadcn's sidebar) is a flex item with
`min-width: auto`. Flex items default to their min-content width, and a `nowrap`
truncated title contributes its full untruncated width to that. The main area therefore
expands, the rows fill it, and the document scrolls horizontally. Verified in a headless
browser: setting `min-width: 0` on that `<main>` brings the document width back to the
viewport at both 1440px and 1100px, and titles truncate in place with the epic and effort
columns visible.

This is the same class of bug as the dashboard right-column fix (missing `min-w-0`), but at
the shell level, so fixing it there covers every page.

## Acceptance Criteria

- [ ] Backlog tab shows no horizontal scrollbar with long task titles at 1440px and 1100px.
- [ ] Epic chip, effort badge, and the toolbar Sort control stay within the viewport.
- [ ] Long titles truncate with an ellipsis rather than pushing the row wider.
- [ ] Other pages (dashboard, roadmap, plans, notes, graph, archive) render unchanged.

## Notes

- Expected fix is a single class on the shell: `<SidebarInset className="min-w-0">` in
  `src/app/layout.tsx`, or `min-w-0` added to the `SidebarInset` base classes in
  `components/ui/sidebar.tsx`. The former avoids diverging from upstream shadcn.
- The kanban board already has `overflow-x-auto` on its column container, so it is not
  affected by this bug and should keep that behaviour.

## References

- [[TASK-iw525]]
