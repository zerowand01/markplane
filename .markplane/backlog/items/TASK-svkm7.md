---
id: TASK-svkm7
title: Dashboard columns collapse when task titles are long
status: done
priority: medium
type: bug
effort: xs
epic: null
plan: null
depends_on: []
blocks: []
related:
- TASK-mh3rv
assignee: null
tags:
- web-ui
position: a9
created: 2026-10-10
updated: 2026-10-10
---

# Dashboard columns collapse when task titles are long

## Description

On the dashboard, a project with long task titles squeezes the left column (Blocked Items,
Active Work, Recent Completions) down to a sliver and draws the right column (Epic
Progress, Up Next) over the top of it. The two-column grid is declared as a 3fr/2fr split
but the split is not honoured.

Root cause: the grid uses a custom track list (`lg:grid-cols-[3fr_2fr]`), which compiles
to plain `3fr 2fr`. Plain `fr` tracks have an automatic minimum of their content's
intrinsic width, and a truncated (`nowrap`) title in the right column contributes its full
untruncated width. Tailwind's built-in `grid-cols-N` utilities avoid this by emitting
`minmax(0, 1fr)`, which is why the roadmap page is unaffected. The left column already had
`min-w-0`; the right column did not.

Two smaller overflows surfaced in the same view: the "blocked by …" dependency list on a
Blocked Items row was `shrink-0` and could push out of its card, and the epic ID in Epic
Progress could wrap mid-ID at narrower widths.

## Acceptance Criteria

- [ ] Dashboard renders a 3fr/2fr split at 1440px and 1024px regardless of title length.
- [ ] Up Next and Recent Completions titles truncate with an ellipsis instead of widening
      the column.
- [ ] A Blocked Items row with many dependencies keeps its title visible and stays inside
      its card.
- [ ] Epic IDs in Epic Progress never wrap.

## Notes

- Same class of bug as [[TASK-mh3rv]], at the grid level rather than the shell level.

## References

- [[TASK-mh3rv]]
