---
id: TASK-tg4kq
title: Web dashboard blocked-by label lists done dependencies
status: draft
priority: low
type: bug
effort: small
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- web-ui
- dashboard
position: aA
created: 2026-11-01
updated: 2026-11-01
---

# Web dashboard blocked-by label lists done dependencies

## Description

The Blocked Items section of the web dashboard renders `blocked by <ids>` from the task's full `depends_on` list, done dependencies included. The CLI `dashboard` filters out done dependencies and lists only the real blockers, so the two views disagree for the same task. Pre-existing; noted during the review of [[TASK-tretc]].

The UI cannot filter on its own: `/api/summary` returns each blocked task as a plain `TaskResponse`, and the UI does not hold the archived task list needed to resolve done status.

## Steps to Reproduce

1. `markplane add "A"`, `markplane add "B"`, `markplane add "C"`
2. `markplane link C A -r depends-on`, `markplane link C B -r depends-on`
3. `markplane update A --status done`
4. `markplane dashboard`: C is blocked by B only
5. `markplane serve`, open the dashboard

## Expected Behavior

The web dashboard shows C as blocked by B.

## Actual Behavior

The web dashboard shows C as blocked by A, B.

## Notes

Suggested fix: add a `blocked_by: Vec<String>` field to each entry of `blocked_tasks` in `SummaryResponse` (`get_summary` in `serve.rs`), computed with `done_task_ids()` over all tasks exactly as the CLI does, and render that in `dashboard-content.tsx`. Keep `depends_on` on the response for compatibility.

## Acceptance Criteria

- [ ] `/api/summary` blocked entries carry the unresolved dependency IDs
- [ ] The web dashboard label and tooltip list only unresolved dependencies, matching `markplane dashboard`
- [ ] A dependency that is done and archived is not listed
- [ ] Serve unit test or integration test covers the new field
- [ ] UI typecheck and lint pass

## References

- [[TASK-tretc]] (archive preserves links)
- `crates/markplane-web/ui/src/app/dashboard/dashboard-content.tsx`
- `crates/markplane-cli/src/commands/serve.rs` `get_summary()`
