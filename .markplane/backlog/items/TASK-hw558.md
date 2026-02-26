---
id: TASK-hw558
title: Include archived tasks in epic progress and detail sheets
status: backlog
priority: medium
type: bug
effort: medium
tags:
- web-ui
- core
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
assignee: null
position: Zz
created: 2026-02-25
updated: 2026-02-25
---

# Include archived tasks in epic progress and detail sheets

## Description

Archived tasks are excluded from epic progress calculations and epic detail sheets. When a done task is archived, it retains its `epic` field in frontmatter but becomes invisible to all progress calculations because `list_tasks(&QueryFilter::default())` uses `ScanScope::Active` (scans only `items/`, not `archive/`).

Example: an epic with 5 tasks where 3 are completed and archived shows progress as 0/2 instead of 3/5.

## Steps to Reproduce

1. Create an epic with several tasks
2. Complete some tasks and archive them
3. Check epic progress — archived-done tasks are missing from the count

## Expected Behavior

- Epic progress should include archived tasks (especially done ones) in the total and completed counts
- Epic detail sheets in the web UI should show archived tasks in a separate, visually distinct section

## Actual Behavior

- Archived tasks vanish from epic progress entirely
- Epic detail sheet only shows active tasks

## Fix

Two changes needed:

### 1. Core: Epic progress uses `ScanScope::All`
- `epic_progress()` in `index.rs` and equivalent logic in `context.rs`, `serve.rs` should scan both active and archived tasks when calculating progress
- Alternatively, add an `epic_tasks()` helper that always uses `ScanScope::All`

### 2. Web UI: Show archived tasks in epic detail sheet
- `epic-detail-sheet.tsx` should fetch archived tasks for the epic (separate query or combined)
- Display them in a collapsed/dimmed "Archived" section below active tasks
- Keep them visually distinct from active work

## References

- `crates/markplane-core/src/query.rs` — `ScanScope`, `list_tasks()`
- `crates/markplane-core/src/index.rs:250` — `generate_roadmap_index()` progress
- `crates/markplane-core/src/context.rs:20-45` — context summary progress
- `crates/markplane-cli/src/commands/serve.rs:502-516` — `epic_to_response()`
- `crates/markplane-web/ui/src/components/domain/epic-detail-sheet.tsx:68-70` — detail sheet query
