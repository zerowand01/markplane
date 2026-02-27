---
id: TASK-e4yqc
title: Remove epic field from Plan model
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
blocks:
- TASK-2j6aa
related: []
assignee: null
position: ZzU
created: 2026-02-27
updated: 2026-02-27
---

# Remove epic field from Plan model

## Description

The `epic` field on Plan is redundant denormalization. Plans implement tasks via the `implements` field, and tasks belong to epics. The epic context is derivable. Worse, if a plan implements tasks across multiple epics, a single `epic` field is misleading.

Remove `epic` from the Plan model entirely. Plans relate to epics only through the tasks they implement.

Touches: Plan struct in `models.rs`, `LinkRelation::Epic` handling in `links.rs` (remove Plan as valid source), CLI show, web API types/handlers, web UI detail sheet, graph builder, templates, existing `.markplane/` files.

## Acceptance Criteria

- [ ] `Plan` struct has no `epic` field
- [ ] `link_items()` with `LinkRelation::Epic` rejects Plan as source (only Task → Epic)
- [ ] CLI `markplane show` for plans does not display epic
- [ ] Web API `PlanResponse` has no `epic`; PATCH handler does not accept it
- [ ] Graph builder does not emit epic edges from plans
- [ ] Plan detail sheet has no "Epic" FieldRow
- [ ] Existing plan files have `epic` removed from frontmatter
- [ ] Plan templates updated
- [ ] All tests pass

## Notes

- The MCP `markplane_add` tool currently accepts `epic` param for plans — needs to be removed or ignored
- `markplane_link` with `epic` relation from a Plan should return an error after this change

## References

- [[TASK-2j6aa]] — Field ordering alignment (depends on this completing first)
