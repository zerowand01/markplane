---
id: TASK-pvqw6
title: Web UI - Detail sheet editing for frontmatter fields and markdown body
status: done
priority: high
type: feature
effort: xl
epic: EPIC-6zdf4
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- web-ui
- editing
position: a9
created: 2026-02-18
updated: 2026-02-19
---

# Web UI - Detail sheet editing for frontmatter fields and markdown body

## Description

Add inline editing capabilities to all detail sheet slide-overs in the web UI. Users should be able to edit both frontmatter fields (title, status, priority, type, effort, tags, epic/plan/task references) and the markdown body content directly within the sheet, with changes persisted via the REST API and optimistic UI updates.

## Acceptance Criteria

- [x] Inline text editing for title fields across all entity detail sheets
- [x] Dropdown/select editing for status, priority, type, and effort fields
- [x] Tag editor component with add/remove support
- [x] Entity combobox component for selecting epic, plan, and task references
- [x] Rich text markdown editor (Tiptap) for body content with wiki-link support
- [x] Custom Tiptap extension for `[[ID]]` wiki-link syntax with autocomplete
- [x] Mutation hooks for updating body content via PATCH API
- [x] Uniform properties table layout with FieldRow component across all sheets
- [x] WikiLinkChip navigation (clicking references opens the linked entity)
- [x] EntityRefEditor for inline editing of entity reference fields (epic, plan, task links)
- [x] Consistent field styling and layout across task, epic, plan, and note detail sheets

## Notes

- Implemented in two passes: initial editing support (3d46006), then refactored for consistency (83057f4)
- Uses Tiptap as the rich text editor with a custom wiki-link extension (`tiptap-wiki-link.ts`)
- Entity combobox fetches available entities from the API for selection
- Body mutations use `PATCH /api/{entity}/{id}/body` endpoints added to the Axum server
- Optimistic updates via TanStack Query ensure immediate UI feedback

## References

- [[EPIC-6zdf4]]
