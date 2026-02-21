---
id: TASK-7cucf
title: Add markplane_write MCP tool for updating markdown body content
status: done
priority: high
type: feature
effort: medium
tags:
- mcp
- ai-integration
epic: EPIC-ji4z3
plan: null
depends_on: []
blocks:
- TASK-pj4ga
assignee: null
position: a8
created: 2026-02-10
updated: 2026-02-11
---

# Add markplane_write MCP tool for updating markdown body content

## Description

Currently the MCP server can create items and update frontmatter fields (status, priority, assignee), but has no way to write or update the markdown body content. When an LLM creates an item via `markplane_add`, the file contains placeholder text like `[What needs to be done and why]`. The LLM has no MCP tool to fill that in — it would have to fall back to raw file editing outside the MCP contract.

A `markplane_write` tool should accept an item ID and new body content (everything below the frontmatter), allowing LLMs to fully manage items through MCP without needing filesystem access.

## Acceptance Criteria

- [ ] New `markplane_write` tool added to `tools.rs` with `id` and `body` parameters
- [ ] Tool replaces only the markdown body, preserving YAML frontmatter untouched
- [ ] Tool updates the `updated` date in frontmatter
- [ ] Tool listed in `list_tools()` with proper JSON Schema
- [ ] Works for all entity types (TASK, EPIC, PLAN, NOTE)
- [ ] Integration tests for writing body content and verifying frontmatter preservation
- [ ] Error handling for invalid IDs

## Notes

This requires a core library function to write the body while preserving frontmatter — currently `write_item()` serializes the full document. Consider adding a `update_body(id, body)` method to `Project` that reads the existing frontmatter, updates the `updated` field, and writes the full document back with the new body.
