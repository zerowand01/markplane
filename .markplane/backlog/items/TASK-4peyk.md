---
id: TASK-4peyk
title: Add --status flag to CLI add
status: backlog
priority: medium
type: enhancement
effort: xs
epic: null
plan: null
depends_on: []
blocks: []
related:
- TASK-5u2bp
assignee: null
tags:
- cli
- dx
position: a5
created: 2026-09-30
updated: 2026-09-30
---

# Add --status flag to CLI add

## Description

New tasks always land in the workflow's default status (the first status in the
earliest category, normally `draft`). A project that works from `backlog`
therefore needs two commands per task: `add` then `update --status backlog`.

The default is technically configurable via the task workflow, but only by
removing statuses from earlier categories. There is no way to keep `draft` and
still create straight into `backlog`.

Add an optional `--status` flag to `markplane add`. The matching MCP `status`
parameter on `markplane_add` is tracked in [[TASK-5u2bp]].

## Acceptance Criteria

- [ ] `markplane add "Title" --status backlog` creates the task in `backlog`
- [ ] Status is validated against the configured task workflow (`validate_task_status`)
- [ ] Omitting `--status` keeps the current default behavior
- [ ] `create_task` in core accepts the status so CLI, MCP and web share one path
- [ ] CLI reference docs updated; integration test added

## Notes

- From external agent feedback on markplane friction.

## References

- [[TASK-5u2bp]]
