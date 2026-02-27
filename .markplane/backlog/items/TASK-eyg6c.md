---
id: TASK-eyg6c
title: Configurable item types and note types
status: done
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
updated: 2026-02-27
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
- [x] `config.yaml` supports `item_types` list defining valid task type values
- [x] `config.yaml` supports `note_types` list defining valid note type values
- [x] Default config provides the current hardcoded values (backward compatible)
- [x] `markplane init` scaffolds the default type lists in config
- [x] Each type entry has a `name` (kebab-case string used in frontmatter)

### Core
- [x] `ItemType` and `NoteType` enums replaced with `String` fields validated against config
- [x] `create_task()` and `update_task()` validate item type against config
- [x] `create_note()` and `update_note()` validate note type against config
- [x] Validation rejects unknown type values on write with a clear error listing valid options
- [x] Reading files with unknown types succeeds (graceful degradation — don't break on stale data)

### CLI
- [x] CLI `--type` and `--note-type` validation reads from config
- [x] Error messages list available types from config

### MCP
- [x] MCP tool schemas dynamically list configured type values in descriptions
- [x] MCP `instructions` field lists available item types and note types

### Web UI
- [x] New API endpoint exposes configured types (or included in existing config/summary endpoint)
- [x] `NoteType` and `ItemType` TypeScript types become `string`
- [x] Dropdowns in create dialogs and detail sheets populate from config via API
- [x] Hardcoded `NOTE_TYPE_CONFIG` in `constants.ts` replaced with dynamic data

### Backward compatibility
- [x] All existing tests pass with default config values
- [x] Projects without type config in `config.yaml` use built-in defaults

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

**Resolved**: All entry points (CLI, MCP, web API) now use `config.default_item_type()` / `config.default_note_type()` which returns the first value in the configured list.

## Implementation Summary

### Approach
Replaced `ItemType` and `NoteType` Rust enums with `String` fields validated against `config.yaml` on write. Read path accepts any string (graceful degradation). The first value in each config list is the default everywhere (CLI, MCP, web API).

### Key design decisions
- **Write-time validation only** — `validate_item_type()` / `validate_note_type()` on `Project` check against config. No validation on read, so stale data never breaks.
- **Config defaults via serde** — `#[serde(default = "default_item_types")]` ensures backward compatibility for configs without the new fields.
- **CLI `--type` is Optional** — No compile-time default; resolved from config at runtime. This avoids the previous inconsistency where CLI, MCP, and web had different hardcoded defaults.
- **Web API `/api/config`** — New endpoint exposes `item_types` and `note_types` to the UI. `useConfig()` hook with 5-minute stale time. WebSocket `config_changed` event invalidates.
- **`NOTE_TYPE_CONFIG` kept as display helper** — Still maps known types to labels for the UI, but all lookups use `?.label ?? capitalize(type)` fallback so custom types display cleanly.

### Files changed (24)
**Core**: `models.rs`, `project.rs`, `lib.rs`, `context.rs`, `references.rs`, `index.rs`, `query.rs`, `links.rs`, `frontmatter.rs`
**CLI**: `commands/mod.rs`, `commands/add.rs`, `commands/note.rs`, `commands/promote.rs`
**MCP**: `mcp/tools.rs`, `mcp/mod.rs`
**Web server**: `commands/serve.rs`
**Web UI**: `lib/types.ts`, `lib/hooks/use-config.ts` (new), `lib/hooks/use-websocket.ts`, `components/domain/create-dialog.tsx`, `components/domain/task-detail-sheet.tsx`, `components/domain/note-detail-sheet.tsx`, `app/notes/notes-content.tsx`
**Docs**: `file-format.md`, `cli-reference.md`, `getting-started.md`, `mcp-setup.md`, `architecture.md`
**MCP integration tests**: `tests/mcp_integration.rs`

### Test results
402 tests passing, clippy clean. Smoke tested: custom types default correctly, invalid types rejected with clear error, backward-compatible with existing configs.

## Settings Page (follow-up)

Added a web UI Settings page (`/settings`) so users can manage task types and note types visually instead of editing `config.yaml` by hand.

- **Backend**: `PATCH /api/config` endpoint with server-side validation (trim, lowercase, dedup, min 1 entry)
- **Frontend**: `useUpdateConfig()` mutation with optimistic cache updates; `TypeListEditor` component with dnd-kit drag-to-reorder, inline add/remove
- **Sidebar**: Settings gear icon in footer using `SidebarMenuButton` (same pattern as main nav items), `g+s` keyboard shortcut
- **Files added**: `settings/page.tsx`, `settings/settings-content.tsx`
- **Files modified**: `serve.rs` (endpoint + route), `use-mutations.ts` (hook), `app-sidebar.tsx` (nav), `use-keyboard-nav.ts` (shortcut), `sidebar.tsx` (rail cursor fix)
- **Docs updated**: `web-ui-guide.md`, `web-ui/architecture.md`, `file-format.md`, `getting-started.md`

## Config rename: `item_types` → `task_types` (follow-up)

Renamed the config field from `item_types` to `task_types` for consistency — every user-facing surface already says "Task", not "Item". The singular `item_type` field on the Rust `Task` struct is unchanged since it's internal (serde-renamed to `type` in YAML/JSON) and used in shared code paths (`UpdateFields`, `QueryFilter`).

- **Rust**: `Config.item_types` → `.task_types`, `default_item_types()` → `default_task_types()`, `default_item_type()` → `default_task_type()`, `validate_item_type()` → `validate_task_type()`
- **API/Web**: `ConfigResponse`, `UpdateConfigRequest`, `ProjectConfig` interface, all component references
- **MCP**: Tool descriptions and instructions ("Item type" → "Task type")
- **Config**: `.markplane/config.yaml` field name
- **Docs**: All 6 doc files updated

## References

- [[TASK-us45u]] — Related: configurable task statuses (category pattern, separate effort)
- [[TASK-ict2n]] — Related: effort point mapping (independent)
