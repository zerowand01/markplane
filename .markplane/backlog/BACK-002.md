---
id: BACK-002
title: Add missing PLAN and NOTE resource templates to MCP
status: backlog
priority: high
type: bug
effort: xs
tags:
- mcp
epic: EPIC-001
plan: null
depends_on: []
blocks: []
assignee: null
created: 2026-02-10
updated: 2026-02-10
---

# Add missing PLAN and NOTE resource templates to MCP

## Description

The MCP `resources/list` response includes `resourceTemplates` for `markplane://backlog/{id}` and `markplane://epic/{id}`, but omits plans and notes. An LLM can read any plan or note via the `markplane_show` tool, but the resource interface — which is the standard MCP way to expose readable data — doesn't advertise them. This is an oversight from the initial implementation.

## Acceptance Criteria

- [ ] `list_resources()` includes `markplane://plan/{id}` in `resourceTemplates`
- [ ] `list_resources()` includes `markplane://note/{id}` in `resourceTemplates`
- [ ] `read_resource()` handles `markplane://plan/PLAN-NNN` URIs
- [ ] `read_resource()` handles `markplane://note/NOTE-NNN` URIs
- [ ] Both validate the ID prefix and return appropriate errors for mismatches
- [ ] New integration tests for plan and note resource reads

## Notes

The implementation follows the exact same pattern as `read_backlog_item()` and `read_epic_item()` in `resources.rs` — validate prefix, resolve path, read file.
