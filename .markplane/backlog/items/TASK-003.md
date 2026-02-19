---
id: TASK-003
title: Update MCP protocol version to 2025-11-25
status: done
priority: high
type: chore
effort: xs
tags:
- mcp
epic: EPIC-001
plan: null
depends_on: []
blocks: []
assignee: null
position: a6
created: 2026-02-10
updated: 2026-02-11
---

# Update MCP protocol version to 2025-11-25

## Description

The `handle_initialize()` response currently reports `protocolVersion: "2024-11-05"`. The current MCP spec is `2025-11-25`, which adds support for the `instructions` field, enhanced `serverInfo` with `description` and `icons`, and other improvements. Reporting the old version may cause clients to skip features we support.

## Acceptance Criteria

- [ ] `protocolVersion` in initialize response changed from `"2024-11-05"` to `"2025-11-25"`
- [ ] Verify no breaking changes in the protocol that require code updates
- [ ] Integration tests updated to assert the new version string

## Notes

This is a one-line change in `handle_initialize()` in `crates/markplane-mcp/src/main.rs`. Should be done alongside TASK-001 and TASK-004 since they all modify the same function.
