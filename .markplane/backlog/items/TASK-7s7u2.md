---
id: TASK-7s7u2
title: Remove depends_on from Epic model
status: backlog
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
position: Zz
created: 2026-02-27
updated: 2026-02-27
---

# Remove depends_on from Epic model

## Description

Epics are strategic roadmap items using the `later → next → now → done` prioritization model. Dependencies between epics are too coarse to be actionable — when Epic B can't start until Epic A finishes, that manifests as specific tasks in B depending on specific tasks in A, which task-level `blocks`/`depends_on` already captures.

The `depends_on` field on Epic has never been used in this project (all 7 epics have `depends_on: []`). Remove it to simplify the model. Epics become pure strategic groupings: `status, priority, started, target, tags, related`.

Touches: Epic struct in `models.rs`, `LinkRelation::DependsOn` handling in `links.rs`, CLI show, web API types/handlers, web UI detail sheet, graph builder, templates, existing `.markplane/` files.

## Acceptance Criteria

- [ ] `Epic` struct has no `depends_on` field
- [ ] `link_items()` with `DependsOn` rejects Epic as source or target
- [ ] CLI `markplane show` for epics does not display depends_on
- [ ] Web API `EpicResponse` has no `depends_on`; PATCH handler ignores it
- [ ] Graph builder does not emit depends_on edges for epics
- [ ] Epic detail sheet has no "Depends on" FieldRow
- [ ] Existing epic files have `depends_on` removed from frontmatter
- [ ] Epic template updated
- [ ] All tests pass; new tests verify Epic rejection from DependsOn

## Notes

- `DependsOn` currently allows Task|Epic → Task|Epic. After this change it should be Task → Task|Epic only (tasks can still depend on epics, but epics can't depend on anything)
- Actually, reconsider: should tasks be able to `depends_on` an Epic? That means "this task is blocked until that epic is done." Could be useful but also coarse. Decide during implementation.

## References

- [[TASK-2j6aa]] — Field ordering alignment (depends on this completing first)
