---
id: TASK-jc69g
title: Create items from web UI
status: backlog
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
created: 2026-02-23
updated: 2026-02-23
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

- [ ] "New Task" button on the backlog page opens a creation dialog
- [ ] "New Epic" creation available from the roadmap page
- [ ] "New Note" creation available from the notes page
- [ ] "New Plan" creation available from a task's detail sheet (linked to that task)
- [ ] Created items appear immediately via optimistic update or WebSocket refresh
- [ ] Creation actions available in the command palette

## References
