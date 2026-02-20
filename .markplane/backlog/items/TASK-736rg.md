---
id: TASK-736rg
title: Extend link command to support cross-type entity linking
status: backlog
priority: high
type: enhancement
effort: medium
tags:
- cli
- mcp
- linking
epic: null
plan: null
depends_on: []
blocks: []
assignee: null
position: a0
created: 2026-02-19
updated: 2026-02-19
---

# Extend link command to support cross-type entity linking

## Description

The `link` command and `markplane_link` MCP tool currently only support Task-to-Task linking (`blocks`/`depends_on`). Several entity types have relationship fields in the data model that have no CLI/MCP/API support for modification, forcing users to manually edit frontmatter.

## Gaps

**Epic-to-Epic:** Epic has a `depends_on` field but no command wires to it.

**Plan-Task linking:** `markplane plan` creates the initial bidirectional link (`task.plan` + `plan.implements`), but there's no way to unlink, re-link, or add additional tasks to a plan's `implements` list after creation.

**Web UI:** The plan field on task detail sheets and implements field on plan detail sheets are read-only because the API (`UpdatePlanRequest`, `UpdateTaskRequest.plan`) doesn't handle bidirectional consistency. Extending the link tool would unblock making these editable in the UI.

## Acceptance Criteria

- [ ] `markplane link` supports Epic-to-Epic `depends_on` relations
- [ ] `markplane link` supports Plan-Task `implements` relation (sets both `task.plan` and `plan.implements` atomically)
- [ ] `markplane_link` MCP tool updated with same capabilities
- [ ] Unlinking supported (remove a relationship)
- [ ] Web API supports plan-task link management (unblocks UI editing)

## Notes

- Bidirectional consistency is the key challenge for plan-task links — updating one side must update the other
- Consider whether `link` should also support Note `related` field
- The `plan` field on Task is singular (one plan per task) while `implements` on Plan is plural — linking a second plan to a task would need to handle this constraint
