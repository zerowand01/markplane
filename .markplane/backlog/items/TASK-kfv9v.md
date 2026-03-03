---
id: TASK-kfv9v
title: Template management UI
status: backlog
priority: medium
type: feature
effort: medium
epic: EPIC-4bjs9
plan: null
depends_on:
- TASK-aphwq
blocks: []
related: []
assignee: null
tags:
- web-ui
- settings
- templates
position: a3
created: 2026-02-28
updated: 2026-03-02
---

# Template management UI

## Description

Add a Templates section to the settings page (in the sidebar created by [[TASK-aphwq]]) that exposes the template system through the web UI. Currently, templates are only manageable by editing files in `.markplane/templates/` and `manifest.yaml` by hand or via AI tools.

The template system has:
- A **manifest** (`manifest.yaml`) mapping kinds (task, epic, plan, note) to named templates
- **Template files** on disk (`task.md`, `task-bug.md`, `note-research.md`, etc.) containing markdown body content with `{PLACEHOLDER}` tokens
- A **resolution chain**: explicit override → type_defaults → kind default → builtin

### UI design

Display the manifest grouped by kind (Task, Epic, Plan, Note). For each kind show:
- The default template name
- Type-specific default mappings (e.g., bug tasks use the "bug" template)
- Available templates with their descriptions

Clicking a template opens an inline editor for the template body using the existing `MarkdownEditor` component (same one used in detail sheets — TipTap with rich/source mode toggle, explicit save).

Users should be able to:
- Edit template body content
- Change which template is the default for a kind
- Edit type-specific default mappings
- Add new templates
- Edit template descriptions

### Backend changes

New API endpoints needed:
- `GET /api/templates` — returns manifest + template bodies
- `PATCH /api/templates/{kind}/{name}` — update a template's body content
- `PATCH /api/templates/manifest` — update default/type_defaults/description mappings
- `POST /api/templates/{kind}` — create a new template

## Acceptance Criteria

- [ ] Templates section appears in settings sidebar (after Task Workflow)
- [ ] Templates are displayed grouped by kind with default and type-specific mappings visible
- [ ] Clicking a template opens the `MarkdownEditor` for editing its body
- [ ] Users can change default templates per kind
- [ ] Users can edit type-specific default mappings
- [ ] Users can add new templates
- [ ] API endpoints exist for reading and writing templates and manifest
- [ ] Template placeholder tokens (`{TITLE}`, etc.) are preserved through editing

## Notes

- Depends on [[TASK-aphwq]] for the settings sidebar layout
- Reuse `MarkdownEditor` from `crates/markplane-web/ui/src/components/domain/markdown-editor.tsx`
- Template manifest: `crates/markplane-core/src/manifest.rs`
- Template files: `.markplane/templates/` directory
- Built-in templates: `crates/markplane-core/src/templates.rs`
- MCP resource `markplane://templates` already reads the manifest — follow the same pattern
- 8 template files currently exist: task, task-bug, epic, plan-implementation, plan-refactor, note, note-research, note-analysis

## References
