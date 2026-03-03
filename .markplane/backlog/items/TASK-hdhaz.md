---
id: TASK-hdhaz
title: MCP protocol compliance fixes
status: done
priority: critical
type: bug
effort: medium
epic: null
plan: null
depends_on: []
blocks: []
related:
- TASK-hv5xv
assignee: null
tags:
- mcp
- protocol
- pre-release
position: a0
created: 2026-03-02
updated: 2026-03-02
---

# MCP protocol compliance fixes

## Description

Multiple MCP protocol and tool behavior issues identified in the pre-release audit. All are in `crates/markplane-cli/src/mcp/` and should be addressed in a single pass.

**`read_line()` allocates before length check — OOM** (Critical)
`BufReader::read_line()` at `mod.rs:34` reads the entire line into memory before the `MAX_LINE_LENGTH` check at line 43. A multi-GB line without a newline causes unbounded allocation. Use `BufRead::take(MAX_LINE_LENGTH as u64).read_line()` to cap allocation.

**`null` ID treated as notification — spec violation** (Critical)
At `mod.rs:76-77`, requests with `"id": null` are silently dropped. Per JSON-RPC 2.0, only requests without an `id` member are notifications. `null` is a valid ID that MUST receive a response. Remove the `|| v.is_null()` branch.

**Wrong error code for unknown tools** (Medium)
At `tools.rs:440-442`, unknown tool names return `INVALID_PARAMS (-32602)`. Should be `METHOD_NOT_FOUND (-32601)`.

**All tool errors use INTERNAL_ERROR** (Medium)
At `tools.rs:458`, all tool errors return `INTERNAL_ERROR (-32603)`. Input validation errors should return `INVALID_PARAMS (-32602)` or use `isError: true` in tool results.

**Resource URI accepts empty IDs** (Medium)
At `resources.rs:75-90`, `markplane://task/` (no ID) extracts an empty string. Check `item_id.is_empty()` and return a clear error.

**Remove `markplane_start` and `markplane_done` MCP tools** (Medium)
These are convenience wrappers over `update_status` that add tool surface area without adding capability. The AI client already knows valid statuses from the `instructions` in the initialize response, and `markplane_update` with an explicit `status` field covers all cases. Removing them reduces the tool count from 17 to 15, eliminates ambiguity (two ways to do the same thing), and avoids the bugs in their non-task handling (`done` fails on Notes, `start` silently works on Plans). Keep the CLI commands for human ergonomics; remove only the MCP tools.

## Acceptance Criteria

- [x] `read_line()` allocation bounded to `MAX_LINE_LENGTH` before parsing
- [x] Requests with `"id": null` receive a response (not treated as notifications)
- [x] Unknown tools return `METHOD_NOT_FOUND (-32601)`
- [x] User input errors return `INVALID_PARAMS (-32602)` or `isError: true`
- [x] Empty resource URIs return a descriptive error
- [x] `markplane_start` and `markplane_done` removed from MCP tool list and dispatch
- [x] MCP `instructions` updated to not reference start/done tools
- [x] All existing MCP integration tests updated and pass

## Implementation Notes

### `markplane_start` / `markplane_done` removal

**Source code** (`crates/markplane-cli/src/mcp/`):
- `tools.rs` — Remove tool definition JSON, dispatch match arms, and `handle_start()` / `handle_done()` handler functions
- `mod.rs` — Update `build_instructions()` line 207: change `"markplane_update/markplane_start/markplane_done"` to just `"markplane_update"`

**Tests** (`crates/markplane-cli/tests/mcp_integration.rs`):
- Remove `assert!(tool_names.contains(&"markplane_start"))` and `"markplane_done"` assertions
- Update tool count assertion from `17` to `15`
- Remove `test_tool_start()` and `test_tool_done()` test functions

**Documentation**:
- `docs/mcp-setup.md` — Remove the two tool table rows (lines 135-136)
- `docs/architecture.md` — Remove from tool list, update "17 tools" → "15 tools" (line 58)
- `docs/ai-integration.md` — Update example workflow referencing `markplane_done` (line 114) to use `markplane_update`
- Archived docs (`docs/archive/`, `.markplane/backlog/archive/`) — Leave as-is (historical)

**No cleanup needed**: `StatusCategory` import in `tools.rs` is still used by `handle_summary()`.

## References

- Source: Pre-release audit (2026-03-02)
- JSON-RPC 2.0 spec: https://www.jsonrpc.org/specification
