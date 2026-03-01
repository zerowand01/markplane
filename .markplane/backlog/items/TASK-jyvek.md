---
id: TASK-jyvek
title: Detect and repair orphaned reciprocal links in check command
status: done
priority: medium
type: enhancement
effort: medium
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- links
- validation
position: a5l
created: 2026-03-01
updated: 2026-03-01
---

# Detect and repair orphaned reciprocal links in check command

## Description

`link_items()` correctly manages reciprocal links at operation time, but there is no
post-hoc validation that reciprocals remain consistent. If frontmatter is manually edited,
an item is deleted without cleanup, or a bug occurs, one side of a reciprocal pair can
exist without the other.

The `check` command currently validates broken references (target doesn't exist) and
invalid task statuses, but does not detect asymmetric reciprocal links where both items
exist but one is missing the back-link.

### Reciprocal pairs to validate

| Field on A        | Expected reciprocal on B | Constraint    |
|--------------------|--------------------------|---------------|
| `blocks: [B]`     | `depends_on: [A]`        | Task → Task   |
| `depends_on: [B]` | `blocks: [A]`            | Task → Task   |
| `plan: B`         | `implements: [A]`        | Task → Plan   |
| `related: [B]`    | `related: [A]`           | Any → Any     |

Note: `epic` on Task is one-directional (no reciprocal field on Epic), so no check needed.

### Approach

- **Detection**: Add a "Link integrity" validation pass to `check` (core + CLI + MCP).
- **Repair**: Add a `--fix` flag to CLI `check` that uses existing `link_items()` to
  add missing reciprocals (idempotent, already handles all write logic).
- **Not in sync**: `sync` regenerates derived views and should never mutate frontmatter.

## Acceptance Criteria

- [x] `markplane check` reports asymmetric reciprocal links (blocks↔depends_on, plan↔implements, related↔related)
- [x] Report shows source item, target item, and which direction is missing
- [x] `markplane check --fix` repairs missing reciprocals using `link_items()`
- [x] MCP `markplane_check` tool also reports asymmetric links
- [x] Existing broken-reference and invalid-status checks still work unchanged
- [x] Tests cover: asymmetric blocks/depends_on, asymmetric plan/implements, asymmetric related, no false positives on correct links

## Notes

- Repair should call `link_items()` rather than raw frontmatter manipulation — keeps all write logic centralized
- `link_items()` is already idempotent, so safe to call even if the reciprocal partially exists
- Type constraint violations (e.g. `blocks` pointing to a non-Task) could be a follow-up

## Implementation Summary

### Core (`markplane-core/src/references.rs`)
- `AsymmetricLink` struct: source_id, target_id, forward_field, missing_field
- `validate_reciprocal_links(project)` loads all items into HashMaps, checks each reciprocal pair, returns sorted results. Only checks targets that exist (non-existent targets are caught by `validate_references`).
- `get_related()` helper returns `&[String]` (zero-copy) for cross-type related lookups
- `collect_asymmetric_related()` helper avoids code repetition across all 4 entity types

### CLI (`markplane-cli/src/commands/check.rs`)
- Added `--fix` flag to `Check` command variant
- `field_to_relation()` maps field names back to `LinkRelation` for repair calls
- Error exit logic: unfixable issues (broken refs, invalid statuses) always error; asymmetric links only error when `--fix` isn't used

### MCP (`markplane-cli/src/mcp/tools.rs`)
- `handle_check()` calls `validate_reciprocal_links()` and appends results
- Success message updated to reflect all three check types

### Tests (418 total, all passing)
- 4 core unit tests: blocks↔depends_on, plan↔implements, related, no false positives
- 2 CLI integration tests: detect asymmetric links, fix and verify repair
- 1 MCP integration test: detect via MCP tool
- Manual test: created temp project, corrupted 2 reciprocal pairs, verified detect → fix → clean

### Docs updated
- `docs/cli-reference.md` — `--fix` flag, updated description and examples
- `docs/architecture.md` — expanded reference validation flow diagram
- `docs/mcp-setup.md` — updated `markplane_check` tool description
- `docs/file-format.md` — updated check reference to mention all validations

## References

- `crates/markplane-core/src/links.rs` — reciprocal management
- `crates/markplane-core/src/references.rs` — existing validation (broken refs, orphans) + new reciprocal validation
- `crates/markplane-cli/src/commands/check.rs` — CLI check command
