---
id: TASK-jc69g
title: Create items from web UI
status: done
priority: high
type: feature
effort: medium
tags:
- web-ui
epic: EPIC-4bjs9
plan: null
depends_on: []
blocks: []
assignee: null
position: a0xV
related: []
created: 2026-02-23
updated: 2026-02-25
---

# Create items from web UI

## Description

Currently the only way to create items is via the CLI or MCP tools — the web UI can view and edit but not create. This is the biggest workflow gap in the web UI.

The backend `POST /api/tasks` endpoint already exists and the `useCreateTask()` mutation hook is defined in `use-mutations.ts` but unused. Backend endpoints for `POST /api/epics`, `POST /api/notes`, and `POST /api/plans` may need to be added to `serve.rs`.

Add creation dialogs/forms accessible from:
- A "New Task" button on the backlog page (title required; type, priority, effort, epic, tags optional with sensible defaults)
- A "New Epic" option on the roadmap page
- A "New Note" option on the notes page
- A "New Plan" option from a task's detail sheet (plans are always linked to a task, so the entry point is the task detail rather than a standalone page)
- The command palette (e.g. "New Task", "New Epic", "New Note" actions)

Use shadcn/ui Dialog component. Keep the forms minimal — title is required, everything else has defaults. Created items should appear immediately via optimistic update or WebSocket refresh.

## Acceptance Criteria

- [x] "New Task" button on the backlog page opens a creation dialog
- [x] "New Epic" creation available from the roadmap page
- [x] "New Note" creation available from the notes page
- [x] "New Plan" creation available from a task's detail sheet (linked to that task)
- [x] Created items appear immediately via optimistic update or WebSocket refresh
- [x] Creation actions available in the command palette

## Implementation Notes

### Backend (serve.rs)
- Added `POST /api/epics`, `POST /api/plans`, `POST /api/notes` endpoints following the existing `create_task` pattern
- Request structs: `CreateEpicRequest` (title, priority), `CreatePlanRequest` (title, task_id), `CreateNoteRequest` (title, note_type, tags)
- Plan creation with `task_id` links the plan back to the task via `link_items()` with error logging (not silent `let _ =`)

### Frontend
- **`CreateDialog`** (`create-dialog.tsx`) — single reusable Dialog component for all 4 entity kinds. Form fields adapt per kind (task: type/priority/effort/epic; epic: priority; note: type; plan: optional task selector). Uses `"none"` sentinel for Radix Select empty values.
- **Mutation hooks** — `useCreateEpic()`, `useCreatePlan()`, `useCreateNote()` in `use-mutations.ts`, each with toast notifications and appropriate cache invalidation
- **Types** — `CreateEpicRequest`, `CreatePlanRequest`, `CreateNoteRequest` in `types.ts`

### Page integration
- Each page (backlog, roadmap, plans, notes) has an entity-colored "New X" button using `color-mix(in oklch, var(--entity-*) 8%, transparent)` for tinted backgrounds
- Created items open their detail sheet immediately via `onCreated` callback

### Command palette
- Added "Create" group with New Task/Epic/Note/Plan actions
- Uses `CustomEvent("create-item")` pattern dispatched to `GlobalCreateDialog` (rendered in layout.tsx) which handles navigation after creation

### Task detail — Plan field
- Replaced the read-only Plan field with `EntityCombobox` (matching the Epic field pattern) — searchable dropdown to link existing plans, with a "Create new plan" action at the bottom
- Extended `EntityCombobox` with optional `onCreateNew` / `createNewLabel` props to support the create action within the dropdown

### Design decisions
- Plans page allows creating plans without a linked task (task selector shown in dialog); task detail always pre-links the plan
- Entity-colored buttons use inline `style` with CSS custom properties for theming consistency
- No optimistic updates for creation — cache invalidation + WebSocket refresh handles immediate appearance

## References
