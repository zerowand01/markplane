---
id: TASK-aphwq
title: Settings page sidebar layout + General section
status: done
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
updated: 2026-03-01
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

- [x] Settings page has an internal sidebar with 4 sections (General, Task Types, Note Types, Task Workflow)
- [x] Each section is URL-addressable (`/settings/general`, etc.)
- [x] `/settings` redirects to `/settings/general`
- [x] General section allows editing project name, description, documentation paths, and context config
- [x] `GET /api/config` returns all config fields including project, context, and documentation_paths
- [x] `PATCH /api/config` accepts updates to project, context, and documentation_paths
- [x] Existing Task Types, Note Types, and Task Workflow editors work as before
- [x] Layout is responsive — sidebar collapses gracefully on mobile

## Implementation Notes

### Files changed
- **`serve.rs`**: Added `ProjectInfoResponse`, `ContextConfigResponse`, `UpdateProjectRequest`, `UpdateContextRequest` structs. Expanded `ConfigResponse` and `UpdateConfigRequest`. Added validation in `patch_config()`: project name (non-empty, max 200), token budget (1-1M), recent days (1-365), doc paths (trim/dedup/filter empty).
- **`types.ts`**: Added `ProjectInfo`, `ContextConfig`, `UpdateConfigRequest` interfaces. Expanded `ProjectConfig`.
- **`use-mutations.ts`**: Changed `useUpdateConfig` from `Partial<ProjectConfig>` to `UpdateConfigRequest`. Added `mergeConfig()` deep-merge helper for correct optimistic updates of nested objects.
- **`settings/layout.tsx`**: New settings layout with sticky sidebar nav (vertical on desktop, horizontal tabs on mobile). Active state uses `bg-accent` matching the main app sidebar pattern.
- **`settings/page.tsx`**: Replaced with `redirect("/settings/general")`.
- **`settings/sections/`**: Extracted `type-list-editor.tsx` (shared), `task-types-section.tsx`, `note-types-section.tsx`, `workflow-section.tsx`, and new `general-section.tsx`.
- **`settings/{general,task-types,note-types,workflow}/page.tsx`**: Thin wrappers with dynamic imports (`ssr: false`).
- **Deleted**: `settings-content.tsx` (superseded by section files).
- **Added**: shadcn `switch.tsx` and `label.tsx` components.
- **Docs updated**: `web-ui-guide.md`, `web-ui/architecture.md`.

### Key decisions
- **Sticky nav, not fixed-height scroll container**: Desktop sidebar uses `sticky top-6 self-start` for independent scrolling without restructuring the root layout.
- **`trailingSlash: true` compatibility**: Active nav detection uses `pathname === href || pathname === href + "/"` to handle Next.js trailing slash config.
- **Full sub-objects for config updates**: Callers send complete nested objects (e.g., `{ project: { name, description } }`) so the shallow PATCH merge on the server is safe. The `mergeConfig()` helper in the optimistic update handles partial nested objects correctly regardless.
- **Enter-to-save on single-line inputs**: Triggers blur which fires the existing `onBlur` save handler. Textarea left as blur-only since Enter is for newlines.
- **Workflow single column**: Changed from 2-column grid to vertical stack for clearer ordering.

## References
