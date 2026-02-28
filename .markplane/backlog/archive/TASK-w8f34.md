---
id: TASK-w8f34
title: Remove markplane_write MCP tool and update instructions to prefer direct file editing
status: done
priority: high
type: enhancement
effort: small
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- mcp
position: a3
created: 2026-02-12
updated: 2026-02-12
---

# Remove markplane_write MCP tool and update instructions to prefer direct file editing

## Description

The `markplane_write` MCP tool sends entire markdown bodies as JSON parameters, creating large opaque tool calls that users can't easily review (no diff visibility). Since `.markplane/` items are plain markdown files in the repo, AI tools should edit them directly using their native file editing capabilities. This gives users visible diffs and targeted edits. MCP tools should handle structural operations (create, status, linking, sync) while content editing happens directly on files.

## Acceptance Criteria

- [ ] Remove `markplane_write` tool definition from `crates/markplane-mcp/src/tools.rs`
- [ ] Remove `handle_write` handler and `update_body` call from tools.rs
- [ ] Update `build_instructions()` in `crates/markplane-mcp/src/main.rs` to guide AI toward direct file editing for content, with file path patterns
- [ ] Remove `markplane_write` integration tests from `crates/markplane-mcp/tests/integration.rs`
- [ ] Update tool count references if any exist
- [ ] All remaining tests pass, clippy clean

## Notes

Keep all other MCP tools — they provide value that direct file editing can't replicate (ID sequencing, bidirectional linking, frontmatter type safety, index regeneration). The distinction: use MCP tools for structured/structural operations, edit files directly for free-form markdown content.
