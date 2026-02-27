---
id: TASK-xwfhp
title: Add created/updated dates to epics
status: done
priority: medium
type: enhancement
effort: medium
tags:
- consistency
- data-model
epic: null
plan: null
depends_on:
- TASK-qgxin
blocks: []
assignee: null
position: Zx
related: []
created: 2026-02-26
updated: 2026-02-26
---

# Add created/updated dates to epics

## Description

Epic is the only entity type without `created` and `updated` date fields. Task, Plan, and Note all have them as required `NaiveDate` fields that are set on creation and bumped on every modification. This is an accidental omission — epics should follow the same pattern.

This is a clean break with no backward compatibility concerns. `created` and `updated` will be required fields (not `serde(default)`), and existing epic files will be updated manually.

## Acceptance Criteria

- [ ] `Epic` struct has required `created: NaiveDate` and `updated: NaiveDate` fields
- [ ] `create_epic()` sets both to today's date
- [ ] `update_epic()` bumps `updated` to today (body is handled via typed update per [[TASK-qgxin]])
- [ ] `link_items()` bumps `updated` on epics when relationships change
- [ ] `show_epic` CLI command displays Created and Updated rows
- [ ] MCP `markplane_query` includes `created`/`updated` in epic JSON output
- [ ] `EpicResponse` and `epic_to_response()` in serve.rs include the fields
- [ ] Web UI `Epic` TypeScript interface and `epic-detail-sheet.tsx` display the dates
- [ ] `test_epic_serde` and MCP integration tests updated
- [ ] All existing epic files in `.markplane/roadmap/items/` have dates added
- [ ] `cargo test --workspace` and `cargo clippy --workspace` pass clean

## Files to Change

| Layer | File | Change |
|-------|------|--------|
| Model | `markplane-core/src/models.rs` | Add fields to `Epic` struct, update `test_epic_serde` |
| Core | `markplane-core/src/project.rs` | `create_epic()`, `update_epic()` |
| Core | `markplane-core/src/links.rs` | Bump `updated` on epics in link operations |
| CLI | `markplane-cli/src/commands/show.rs` | Display dates in `show_epic` |
| MCP | `markplane-cli/src/mcp/tools.rs` | Include dates in epic query output |
| Web API | `markplane-cli/src/commands/serve.rs` | `EpicResponse` struct + `epic_to_response()` |
| Web UI | `markplane-web/ui/src/lib/types.ts` | `Epic` interface |
| Web UI | `markplane-web/ui/src/components/domain/epic-detail-sheet.tsx` | FieldRow display |
| Tests | `markplane-cli/tests/mcp_integration.rs` | Assert epic query includes dates |
| Data | `.markplane/roadmap/items/EPIC-*.md` | Add frontmatter dates to existing files |

## References
