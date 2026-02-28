---
id: TASK-8am45
title: Add relationship editing to epic and plan detail sheets
status: done
priority: medium
type: enhancement
effort: medium
epic: EPIC-4bjs9
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- web-ui
position: a9
created: 2026-02-25
updated: 2026-02-25
---

# Add relationship editing to epic and plan detail sheets

## Description

The task detail sheet has full inline editing for all its relationship fields (epic, plan, depends_on, blocks). The note detail sheet has editable `related`. But the epic and plan detail sheets are missing relationship editing entirely:

- **Epic detail sheet**: `depends_on` is not displayed or editable. There's no way to set inter-epic dependencies from the web UI.
- **Plan detail sheet**: `epic` and `implements` are displayed as read-only WikiLinkChips but cannot be edited.

This means relationship changes for epics and plans can only be made via CLI (`markplane link`) or MCP — the web UI has a gap.

### Architecture note

Follow the established adapter pattern from [[TASK-t2hvn]]: add relationship fields to the PATCH request structs, use `diff_vec()` + `link_items()` in the backend handler, and call the existing `useUpdate{Type}()` mutation hooks from the frontend. This matches how tasks and notes already handle relationships. See `task-detail-sheet.tsx` lines 299-310 for the depends_on editor pattern.

Backend changes needed in `serve.rs`:
- Add `depends_on` to `UpdateEpicRequest` and the `update_epic` handler
- Add `epic` to `UpdatePlanRequest` and the `update_plan` handler
- `implements` on plans is managed from the task side (task.plan → plan.implements), so no plan PATCH change needed for that

## Acceptance Criteria

- [ ] Epic detail sheet displays `depends_on` list (other epics this epic depends on)
- [ ] Epic detail sheet allows adding/removing epic dependencies
- [ ] Plan detail sheet allows editing which epic the plan belongs to
- [ ] Plan detail sheet's `implements` list remains read-only (managed from task side)
- [ ] Backend `UpdateEpicRequest` accepts `depends_on` and routes through `link_items()`
- [ ] Backend `UpdatePlanRequest` accepts `epic` and routes through `link_items()`
- [ ] Existing tests pass, new relationship editing works end-to-end

## References

- [[TASK-t2hvn]] — established the PATCH adapter pattern for relationship fields
- `task-detail-sheet.tsx` — reference implementation for depends_on editing
- `note-detail-sheet.tsx` — reference implementation for related editing
