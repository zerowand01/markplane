---
id: TASK-eyg6c
title: Configurable item types and note types
status: backlog
priority: medium
type: feature
effort: medium
tags:
- core
- config
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
assignee: null
position: Zz
created: 2026-02-26
updated: 2026-02-26
---

# Configurable item types and note types

## Description

The `ItemType` and `NoteType` enums are hardcoded in Rust, preventing teams from customizing their task and note taxonomies. A team might want task types like `docs`, `security`, `design`, or `infrastructure`, or note types like `retro`, `standup`, `adr`, or `brainstorm`. Unlike statuses (which have deep semantic dependencies — progress bars, kanban columns, blocking logic), item types and note types are essentially **classification labels** with no system behavior tied to specific values. This makes them straightforward candidates for configuration.

The template system already works with arbitrary type strings (`resolve_template_body()` looks up type_defaults in `manifest.yaml`), so custom types just need corresponding template entries if the user wants type-specific templates.

### Current hardcoded values

| Enum | Current values | Config key |
|------|---------------|------------|
| `ItemType` | feature, bug, enhancement, chore, research, spike | `item_types` |
| `NoteType` | research, analysis, idea, decision, meeting | `note_types` |

## Acceptance Criteria

### Config schema
- [ ] `config.yaml` supports `item_types` list defining valid task type values
- [ ] `config.yaml` supports `note_types` list defining valid note type values
- [ ] Default config provides the current hardcoded values (backward compatible)
- [ ] `markplane init` scaffolds the default type lists in config
- [ ] Each type entry has a `name` (kebab-case string used in frontmatter)

### Core
- [ ] `ItemType` and `NoteType` enums replaced with `String` fields validated against config
- [ ] `create_task()` and `update_task()` validate item type against config
- [ ] `create_note()` and `update_note()` validate note type against config
- [ ] Validation rejects unknown type values on write with a clear error listing valid options
- [ ] Reading files with unknown types succeeds (graceful degradation — don't break on stale data)

### CLI
- [ ] CLI `--type` and `--note-type` validation reads from config
- [ ] Error messages list available types from config

### MCP
- [ ] MCP tool schemas dynamically list configured type values in descriptions
- [ ] MCP `instructions` field lists available item types and note types

### Web UI
- [ ] New API endpoint exposes configured types (or included in existing config/summary endpoint)
- [ ] `NoteType` and `ItemType` TypeScript types become `string`
- [ ] Dropdowns in create dialogs and detail sheets populate from config via API
- [ ] Hardcoded `NOTE_TYPE_CONFIG` in `constants.ts` replaced with dynamic data

### Backward compatibility
- [ ] All existing tests pass with default config values
- [ ] Projects without type config in `config.yaml` use built-in defaults

## Notes

### Locations requiring changes

**Both types**: `models.rs` (enum → String), `project.rs` (create/update validation), `mcp/tools.rs` (schema descriptions), `serve.rs` (request/response types), Web UI (`types.ts`, `constants.ts`)

**ItemType additionally**: `commands/mod.rs` (CLI `--type` arg), `create-dialog.tsx`, `task-detail-sheet.tsx`

**NoteType additionally**: `commands/note.rs` + `commands/mod.rs` (CLI `--note-type` arg), `create-dialog.tsx`, `note-detail-sheet.tsx`, `manifest.rs` (template type_defaults)

### Config format

```yaml
item_types:
  - feature
  - bug
  - enhancement
  - chore
  - research
  - spike

note_types:
  - research
  - analysis
  - idea
  - decision
  - meeting
```

Simple string lists — no category mapping needed (unlike statuses). The first value in each list is the default for new items.

### Default inconsistency to fix

Currently the CLI defaults note type to `idea`, MCP defaults to `research`, and the web UI defaults to `research`. This should be unified — the first value in the config list becomes the default everywhere.

## References

- [[TASK-us45u]] — Related: configurable task statuses (category pattern, separate effort)
- [[TASK-ict2n]] — Related: effort point mapping (independent)
