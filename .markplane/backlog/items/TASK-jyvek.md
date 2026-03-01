---
id: TASK-jyvek
title: Detect and repair orphaned reciprocal links in check command
status: planned
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

- [ ] `markplane check` reports asymmetric reciprocal links (blocks↔depends_on, plan↔implements, related↔related)
- [ ] Report shows source item, target item, and which direction is missing
- [ ] `markplane check --fix` repairs missing reciprocals using `link_items()`
- [ ] MCP `markplane_check` tool also reports asymmetric links
- [ ] Existing broken-reference and invalid-status checks still work unchanged
- [ ] Tests cover: asymmetric blocks/depends_on, asymmetric plan/implements, asymmetric related, no false positives on correct links

## Notes

- Repair should call `link_items()` rather than raw frontmatter manipulation — keeps all write logic centralized
- `link_items()` is already idempotent, so safe to call even if the reciprocal partially exists
- Type constraint violations (e.g. `blocks` pointing to a non-Task) could be a follow-up

## References

- `crates/markplane-core/src/links.rs` — reciprocal management
- `crates/markplane-core/src/references.rs` — existing validation (broken refs, orphans)
- `crates/markplane-cli/src/commands/check.rs` — CLI check command
