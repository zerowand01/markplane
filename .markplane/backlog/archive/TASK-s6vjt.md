---
id: TASK-s6vjt
title: Implement Now/Next/Later epic lifecycle model
status: done
priority: high
type: feature
effort: large
epic: EPIC-c5uem
plan: null
depends_on: []
blocks:
- TASK-xvx5f
related: []
assignee: null
tags:
- roadmap
- data-model
- web-ui
position: a0Z
created: 2026-02-22
updated: 2026-02-24
---

# Implement Now/Next/Later epic lifecycle model

## Description

Replace the epic `status` field values (`planned | active | done`) with Now/Next/Later time horizons (`now | next | later | done`), making the NNL roadmap pattern a first-class concept in the data model. See [[NOTE-v3uet]] for research and architectural rationale.

Currently the web UI derives Now/Next/Later columns by mapping `status: active` → Now, then splitting `status: planned` by priority (critical/high → Next, medium/low/someday → Later). This conflates priority (importance) with time horizon (planning intent). The fix is to make horizon explicit in the data model so priority and horizon are independent dimensions.

### 1. Core data model (`markplane-core/src/models.rs`)

Replace `EpicStatus` enum:

```rust
// Before
pub enum EpicStatus { Planned, Active, Done }

// After
pub enum EpicStatus { Now, Next, Later, Done }
```

Update `Display`, `FromStr`, and serde (`rename_all = "kebab-case"` should handle serialization). The lifecycle is `later → next → now → done`.

Default for new epics: `later` (previously `planned`).

### 2. Core update/status logic (`markplane-core/src/project.rs`)

- `update_status()`: epic branch now accepts `now | next | later | done`
- `create_epic()`: default status changes from `planned` to `later`
- `update_epic()`: no structural changes (status field stays `status`, validation via `EpicStatus::from_str`)

### 3. Core index generation (`markplane-core/src/index.rs`)

Update `generate_roadmap_index()` to render sections matching the new values:

- **Now** (was "Active Epics")
- **Next** (new — was lumped into "Planned Epics")
- **Later** (new — was lumped into "Planned Epics")
- **Done** (unchanged)

Within each section, sort by priority rank then ID (existing behavior).

### 4. Core context generation

Update any `.context/` generation that references epic statuses to use the new values.

### 5. MCP tool descriptions (`markplane-cli/src/mcp/tools.rs`)

Update `markplane_update` tool schema — the `status` field description should list allowed values per entity type:

```
"Status value. Task: draft/backlog/planned/in-progress/done/cancelled. Epic: now/next/later/done. Plan: draft/approved/in-progress/done. Note: draft/active/archived."
```

This follows the existing pattern where type-specific fields include their scope (e.g. `"Effort size (xs, small, medium, large, xl). Tasks only."`).

Update `markplane_start` description: clarify it sets tasks to `in-progress` (not applicable to epics — epics use `markplane_update --status now`).

Update `markplane_done` description: works for all entity types (sets status to `done`).

### 6. CLI commands

- `markplane epic` (create): default status → `later`
- `markplane update --status`: accepts new epic values
- `markplane start`: currently calls `update_status(id, "in-progress")` — this already fails for epics since "in-progress" is not a valid EpicStatus. No change needed, but consider improving the error message to suggest using `update --status now` for epics.
- CLI dashboard/formatting: update any status display logic

### 7. Web UI (`markplane-web/ui/`)

- **Types** (`lib/types.ts`): update `EpicStatus` type to `"now" | "next" | "later" | "done"`
- **Roadmap page** (`app/roadmap/page.tsx`): simplify the `useMemo` grouping — filter directly by status value instead of the current priority-based derivation:
  ```typescript
  const now = sortByPriority(epics.filter(e => e.status === "now"));
  const next = sortByPriority(epics.filter(e => e.status === "next"));
  const later = sortByPriority(epics.filter(e => e.status === "later"));
  const done = epics.filter(e => e.status === "done");
  ```
- **Epic detail sheet**: update status selector/display to show new values
- **Any other components** that reference EpicStatus values (search for `"active"`, `"planned"` in the UI codebase)

### 8. Tests

- Unit tests: update `EpicStatus` parsing tests, index generation tests, update tests
- Integration tests: update any CLI/MCP tests that create or modify epics
- Verify the web API responses use new status values

### 9. Migration

Clean break — no backward-compatible parsing. The `EpicStatus::FromStr` implementation should only accept the new values (`now | next | later | done`) — old values produce an `InvalidStatus` error.

Existing epic files with `status: planned` or `status: active` must be migrated manually (by the implementer or via a one-off script) as part of this task. The mapping is:

- `status: planned` → `status: later`
- `status: active` → `status: now`

This is not a versioned migration — no version bump in `config.yaml`, no `markplane migrate` integration, no built-in sync step. Just a direct rewrite of the affected files in this project's `.markplane/`.

## Acceptance Criteria

- [ ] `EpicStatus` enum uses `now | next | later | done`
- [ ] New epics default to `later`
- [ ] Roadmap INDEX renders Now/Next/Later/Done sections
- [ ] MCP `markplane_update` status description lists per-type allowed values
- [ ] Web UI roadmap groups by status directly (no priority-based derivation)
- [ ] Migration rewrites existing epic files (`planned` → `later`, `active` → `now`)
- [ ] All tests pass, clippy clean

## Notes

- Priority remains an independent dimension — it ranks epics *within* a horizon column, not across columns
- `markplane_start` and `markplane_done` are task-oriented conveniences; epic lifecycle management uses `markplane_update --status <value>`
- The `started` and `target` date fields on epics remain unchanged — they're optional metadata, not lifecycle drivers

## References

- [[NOTE-v3uet]]
