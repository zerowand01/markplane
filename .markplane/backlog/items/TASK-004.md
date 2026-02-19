---
id: TASK-004
title: Add serverInfo.description to MCP initialize response
status: done
priority: high
type: feature
effort: xs
tags:
- mcp
epic: EPIC-001
plan: null
depends_on: []
blocks: []
assignee: null
position: a7
created: 2026-02-10
updated: 2026-02-11
---

# Add serverInfo.description to MCP initialize response

## Description

The MCP spec's `serverInfo` object supports a `description` field that gives the LLM/client a one-liner about what the server does. Our current `serverInfo` only has `name` and `version`. Adding a description helps MCP clients display meaningful info about Markplane in their UI and gives the LLM additional context about the server's purpose.

## Acceptance Criteria

- [ ] `serverInfo` in initialize response includes a `description` field
- [ ] Description clearly communicates what Markplane is (e.g., "AI-native, markdown-first project management. Files are the source of truth, git is the changelog.")
- [ ] Integration test validates the description field is present

## Notes

One-line addition to the `serverInfo` JSON in `handle_initialize()`. Should be done alongside TASK-001 and TASK-003 since they all touch the same function.
