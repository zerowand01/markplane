---
id: TASK-haky9
title: 'Clean up context command: remove --item, fix --focus'
status: backlog
priority: medium
type: chore
effort: small
tags:
- context
- cleanup
epic: EPIC-a5vs9
plan: null
depends_on: []
blocks: []
assignee: null
position: Zzz
created: 2026-02-10
updated: 2026-02-23
---

# Clean up context command: remove --item, fix --focus

## Description

Two issues with the `context` command:

**1. `--item` is redundant and should be removed.** The MCP tool version literally just reads the file — identical to `markplane_show`. The CLI version does a slightly richer display (item + epic + plan + deps) but nothing that `show` + `graph` don't already cover. AI agents naturally compose context by calling `show` on the items they need, guided by `graph` for discovery.

**2. `--focus` is broken on CLI and should be fixed.** The MCP tool's `--focus` parameter works correctly — it generates and returns a specific context view (active-work, blocked, metrics, or summary). But the CLI `--focus` flag silently ignores its value and regenerates all context files, same as calling `context` with no flags. The CLI should match MCP behavior: generate and print the requested context view to stdout.

The `context` command should focus on its actual job: generating and displaying `.context/` views (summary, active-work, blocked, metrics).

## Acceptance Criteria

### Remove --item
- [ ] Remove `--item` flag from CLI `context` command (clap definition + handler)
- [ ] Remove `generate_item_context()` from `commands/context.rs`
- [ ] Remove `item` parameter from `markplane_context` MCP tool schema and handler
- [ ] Remove CLI integration test `test_context_for_item`
- [ ] Remove MCP integration test `test_tool_context_for_item`
- [ ] Update `markplane_context` tool description (remove "or a specific item")

### Fix --focus on CLI
- [ ] `markplane context --focus active-work` generates and prints active-work context to stdout
- [ ] `markplane context --focus blocked` generates and prints blocked-items context to stdout
- [ ] `markplane context --focus metrics` generates and prints metrics context to stdout
- [ ] `markplane context --focus summary` generates and prints summary context to stdout
- [ ] `markplane context` (no flags) regenerates all `.context/` files (existing behavior, unchanged)
- [ ] Add/update CLI integration tests for `--focus` variants

### Docs
- [ ] Update cli-reference.md, mcp-setup.md, getting-started.md

## Notes

Repurposed from the original "rich context bundles" task. Analysis showed the bundling feature isn't needed — `markplane_show` + `markplane_graph` already give AI agents everything they need with better precision and less wasted tokens. The `--focus` fix aligns the CLI with the MCP tool, which already works correctly.
