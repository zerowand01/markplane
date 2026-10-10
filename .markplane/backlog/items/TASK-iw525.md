---
id: TASK-iw525
title: Show full title on hover for truncated titles in web UI lists
status: planned
priority: medium
type: enhancement
effort: small
epic: null
plan: null
depends_on: []
blocks: []
related:
- TASK-mh3rv
assignee: null
tags:
- web-ui
position: a7
created: 2026-10-10
updated: 2026-10-10
---

# Show full title on hover for truncated titles in web UI lists

## Description

Several list views in the web UI truncate long task and epic titles with an ellipsis
(dashboard rows, backlog list rows, kanban cards via `line-clamp-2`, roadmap task rows,
epic detail sheet tables). Once truncated, the only way to read the full title is to open
the detail sheet. Hovering should reveal the full text so a user can scan a list without
clicking into each item.

This pairs with [[TASK-mh3rv]]: once the backlog keeps to the viewport width, more titles
will truncate, so hover-reveal becomes the primary way to read them in place.

Constraints:
- Rows and cards are drag sources (dnd-kit). The hover affordance must not interfere with
  drag start or add per-row interaction overhead.
- Only show the affordance when the text is actually truncated, or accept that a native
  `title` attribute is harmless when it is not.

## Acceptance Criteria

- [ ] Hovering a truncated title in the dashboard, backlog list, kanban board, roadmap, and
      epic detail sheet reveals the full title.
- [ ] Drag and drop on the backlog list and kanban board behaves as before.
- [ ] No visible change for titles that fit.
- [ ] Approach is consistent across all list views (one pattern, not a mix).

## Notes

- A native `title` attribute is the lowest-cost option: no JS, no extra elements, no
  conflict with dnd-kit pointer listeners. Downsides are the browser's hover delay and no
  keyboard access.
- The shadcn `Tooltip` primitive exists in `components/ui/tooltip.tsx` if a styled tooltip
  is preferred, but wrapping every row in a Radix tooltip adds overhead on long lists and
  needs care around drag activation.
- Existing truncation sites: `grep -rn "truncate\|line-clamp" src/app src/components/domain`.

## References

- [[TASK-mh3rv]]
