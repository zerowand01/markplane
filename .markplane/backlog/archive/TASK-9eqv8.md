---
id: TASK-9eqv8
title: Toast-based undo for mutations
status: done
priority: medium
type: feature
effort: medium
epic: EPIC-4bjs9
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- web-ui
position: Zz
created: 2026-02-23
updated: 2026-02-25
---

# Toast-based undo for mutations

## Description

When a user changes a task's status, priority, or other properties, there's no way to undo the action if it was a mistake. Add toast-based undo following the Gmail/Slack pattern: after a mutation succeeds, show a toast with an "Undo" button that reverts the change within a ~5 second window. No full undo stack or Cmd+Z needed.

Sonner (already installed) supports action buttons on toasts. The optimistic update pattern in `use-mutations.ts` already captures previous state in `onMutate` — the undo handler can reuse this snapshot.

**Approach — write-then-revert:** Apply the mutation immediately (current behavior), then if the user clicks Undo, write the old values back via a second API call. This is simpler and safer than delaying the server write, since it doesn't require a pending-write queue or risk data loss if the browser closes.

Key file: `crates/markplane-web/ui/src/lib/hooks/use-mutations.ts`

## Acceptance Criteria

- [x] Status changes show a toast with an "Undo" action button
- [x] Clicking "Undo" reverts the item to its previous state
- [x] Undo window expires after ~5 seconds (toast auto-dismisses)
- [x] Works for status, priority, and effort changes at minimum
- [x] If the undo API call fails, an error toast is shown

## Implementation Notes

**Scope exceeded AC minimum** — undo covers all 4 entity types (tasks, epics, plans, notes) across these fields:
- Tasks: status, priority, effort, type, assignee, tags, epic, position
- Epics: status, priority, tags, started, target
- Plans: status
- Notes: status, type, tags
- Archive/unarchive: single item and batch archive

**Key design decisions:**
- `buildUndoPayload()` helper extracts previous field values from the `onMutate` snapshot
- `NULLABLE_FIELDS` set maps `null` previous values to `""` so the server interprets them as `Patch::Clear` (not `Patch::Unchanged`)
- `detectField()` helper with ordered `[field, label]` arrays replaced deeply nested ternary chains for field label detection
- `invalidateAllEntities()` extracted to DRY up archive/unarchive hooks
- Non-undoable fields (title, body) show a plain toast without the undo button

**Bug found during review:** Nullable fields (assignee, epic, position, started, target) where the previous value was `null` would silently fail to undo — JSON `null` maps to `Patch::Unchanged` on the server. Fixed by substituting `""` which maps to `Patch::Clear`.

## References
