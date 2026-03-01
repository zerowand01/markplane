---
id: TASK-aphwq
title: Settings page sidebar layout + General section
status: backlog
priority: medium
type: enhancement
effort: medium
epic: EPIC-4bjs9
plan: null
depends_on: []
blocks:
- TASK-kfv9v
related: []
assignee: null
tags:
- web-ui
- settings
position: a5
created: 2026-02-28
updated: 2026-02-28
---

# Settings page sidebar layout + General section

## Description

The settings page currently uses a flat 2-column card grid with 3 sections (Task Types, Note Types, Task Workflow). This doesn't scale well — Task Workflow already spans both columns, and there's no room for new sections without the layout becoming unwieldy.

Refactor the settings page to use an internal sidebar navigation + content area pattern. The sidebar lists all settings sections; the content area shows the selected section at full width. This is the standard approach used by GitHub, VS Code, Linear, etc.

Additionally, add a new **General** section exposing project-level config fields that are currently only editable by hand in `config.yaml`: project name, description, documentation paths, and context generation settings.

### Sidebar sections

1. **General** — project name (text), description (textarea), documentation paths (list editor), context config: token budget (number), recent days (number), auto-generate (toggle)
2. **Task Types** — existing `TypeListEditor`, first item is default
3. **Note Types** — existing `TypeListEditor`, first item is default
4. **Task Workflow** — existing `WorkflowEditor` with 6 category buckets

### Layout

- Internal sidebar (~200px) with vertical nav list on the left, content area takes remaining width
- URL-based routing: `/settings/general`, `/settings/task-types`, `/settings/note-types`, `/settings/workflow`
- `/settings` redirects to `/settings/general`
- Mobile: sidebar collapses to horizontal tabs or a dropdown above the content

### Backend changes

Expand `GET /api/config` and `PATCH /api/config` to include `project` (name, description) and `context` (token_budget, recent_days, auto_generate) and `documentation_paths` fields.

## Acceptance Criteria

- [ ] Settings page has an internal sidebar with 4 sections (General, Task Types, Note Types, Task Workflow)
- [ ] Each section is URL-addressable (`/settings/general`, etc.)
- [ ] `/settings` redirects to `/settings/general`
- [ ] General section allows editing project name, description, documentation paths, and context config
- [ ] `GET /api/config` returns all config fields including project, context, and documentation_paths
- [ ] `PATCH /api/config` accepts updates to project, context, and documentation_paths
- [ ] Existing Task Types, Note Types, and Task Workflow editors work as before
- [ ] Layout is responsive — sidebar collapses gracefully on mobile

## Notes

- Current settings implementation: `crates/markplane-web/ui/src/app/settings/settings-content.tsx`
- Config model: `Config` struct in `crates/markplane-core/src/models.rs`
- API endpoints: `get_config` / `patch_config` in `crates/markplane-cli/src/commands/serve.rs`
- Keep the sidebar lightweight — a simple nav list, not the full shadcn Sidebar component
- [[TASK-kfv9v]] (Template management UI) depends on this task for the sidebar layout

## References
