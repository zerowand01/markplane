---
id: TASK-9eqv8
title: Toast-based undo for mutations
status: backlog
priority: medium
type: feature
effort: medium
tags:
- web-ui
epic: EPIC-4bjs9
plan: null
depends_on: []
blocks: []
assignee: null
position: Zz
created: 2026-02-23
updated: 2026-02-23
---

# Toast-based undo for mutations

## Description

When a user changes a task's status, priority, or other properties, there's no way to undo the action if it was a mistake. Add toast-based undo following the Gmail/Slack pattern: after a mutation succeeds, show a toast with an "Undo" button that reverts the change within a ~5 second window. No full undo stack or Cmd+Z needed.

Sonner (already installed) supports action buttons on toasts. The optimistic update pattern in `use-mutations.ts` already captures previous state in `onMutate` — the undo handler can reuse this snapshot.

**Approach — write-then-revert:** Apply the mutation immediately (current behavior), then if the user clicks Undo, write the old values back via a second API call. This is simpler and safer than delaying the server write, since it doesn't require a pending-write queue or risk data loss if the browser closes.

Key file: `crates/markplane-web/ui/src/lib/hooks/use-mutations.ts`

## Acceptance Criteria

- [ ] Status changes show a toast with an "Undo" action button
- [ ] Clicking "Undo" reverts the item to its previous state
- [ ] Undo window expires after ~5 seconds (toast auto-dismisses)
- [ ] Works for status, priority, and effort changes at minimum
- [ ] If the undo API call fails, an error toast is shown

## References
