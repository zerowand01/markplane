---
id: TASK-us45u
title: Implement configurable workflows for statuses, priorities, effort sizes, item types, and note types
status: backlog
priority: medium
type: feature
effort: large
tags:
- core
- config
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
assignee: null
position: Zzx
created: 2026-02-10
updated: 2026-02-26
---

# Implement configurable workflows for statuses, priorities, effort sizes, item types, and note types

## Description

The design spec envisions statuses, priorities, and effort sizes as configurable values defined in `config.yaml`, but the current implementation hardcodes them as Rust enums (`TaskStatus`, `EpicStatus`, `PlanStatus`, `NoteStatus`, `Priority`, `Effort`, `ItemType`, `NoteType`). This means users cannot customize their workflow — for example, adding a `review` status, removing `someday` priority, using t-shirt sizes different from `xs/small/medium/large/xl`, adding a `docs` or `security` task type, or adding a `retro` note type. Moving to config-driven workflows would allow teams to tailor Markplane to their process.

This is a large architectural change that touches models, serialization, validation, CLI display, MCP tool schemas, and the web UI. The hardcoded enums would be replaced with string-based values validated against the config at runtime.

### Hardcoded enums to replace

| Enum | Field | Current values | Config key |
|------|-------|---------------|------------|
| `TaskStatus` | `status` (tasks) | draft, backlog, planned, in-progress, done, cancelled | `workflows.task` |
| `EpicStatus` | `status` (epics) | now, next, later, done | `workflows.epic` |
| `PlanStatus` | `status` (plans) | draft, approved, in-progress, done | `workflows.plan` |
| `NoteStatus` | `status` (notes) | draft, active, archived | `workflows.note` |
| `Priority` | `priority` (tasks, epics) | critical, high, medium, low, someday | `priorities` |
| `Effort` | `effort` (tasks) | xs, small, medium, large, xl | `effort_sizes` |
| `ItemType` | `type` (tasks) | feature, bug, enhancement, chore, research, spike | `item_types` |
| `NoteType` | `type` (notes) | research, analysis, idea, decision, meeting | `note_types` |

## Acceptance Criteria

### Config schema
- [ ] `config.yaml` supports `workflows` section defining valid statuses per entity type
- [ ] `config.yaml` supports `priorities` list defining valid priority values
- [ ] `config.yaml` supports `effort_sizes` list defining valid effort values
- [ ] `config.yaml` supports `item_types` list defining valid task type values
- [ ] `config.yaml` supports `note_types` list defining valid note type values
- [ ] Default config provides the current hardcoded values (backward compatible)
- [ ] `markplane init` scaffolds the default workflow config

### Validation
- [ ] Validation rejects unknown status/priority/effort/item-type/note-type values on write
- [ ] `create_note()` and `update_note()` validate note type against config
- [ ] `create_task()` and `update_task()` validate item type against config

### CLI
- [ ] CLI `--status`, `--priority`, `--type`, `--note-type` validation reads from config

### MCP
- [ ] MCP tool schemas dynamically reflect configured values (descriptions list available options)
- [ ] MCP `instructions` field dynamically lists available values

### Web UI
- [ ] Web UI fetches available types/statuses/priorities from API instead of hardcoded arrays
- [ ] `NoteType` and `ItemType` TypeScript types become `string` (or fetched union)
- [ ] Dropdowns in create dialogs and detail sheets populate from config

### Backward compatibility
- [ ] All existing tests pass with default config values
- [ ] Migration path: existing projects without workflow config use built-in defaults

## Notes

Consider a phased approach: first add the config schema and parsing with defaults matching current enums, then swap enum-based validation to config-based validation. The hardest part is replacing strongly-typed enums with validated strings throughout the codebase while maintaining type safety. A `ValidatedStatus(String)` newtype pattern could help. The MCP `instructions` field ([[TASK-eduur]]) should dynamically list available statuses once this lands.

`ItemType` and `NoteType` have template integration — `resolve_template_body()` uses the type string for template lookup (`manifest.rs` type_defaults). The template system already works with arbitrary strings, so custom types just need corresponding template entries in `manifest.yaml` if the user wants type-specific templates.

### Locations requiring changes (by enum)

**All enums**: `models.rs` (enum definition, Display, FromStr, struct fields), `project.rs` (create/update methods), `mcp/tools.rs` (schema descriptions, handlers), `serve.rs` (web API request/response types), Web UI (`types.ts`, `constants.ts`, component dropdowns)

**ItemType additionally**: `commands/mod.rs` (CLI `--type` arg), `create-dialog.tsx`, task detail sheet
**NoteType additionally**: `commands/note.rs` + `commands/mod.rs` (CLI `--type`/`--note-type` args), `create-dialog.tsx`, `note-detail-sheet.tsx`, `manifest.rs` (template type_defaults)

## References

- Design spec workflow section: `docs/ai-native-pm-system-design.md`
