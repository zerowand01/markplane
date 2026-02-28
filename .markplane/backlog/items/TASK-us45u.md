---
id: TASK-us45u
title: Configurable task statuses with status categories
status: backlog
priority: medium
type: feature
effort: large
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- core
- config
position: Zzx
created: 2026-02-10
updated: 2026-02-26
---

# Configurable task statuses with status categories

## Description

The `TaskStatus` enum is hardcoded with 6 variants (draft, backlog, planned, in-progress, done, cancelled), preventing teams from customizing their workflow — for example, adding `in-review`, `blocked`, `in-qa`, or `deployed` statuses. Unlike item types and note types (which are pure labels — see [[TASK-eyg6c]]), task statuses have deep semantic dependencies throughout the codebase: progress bars, kanban column grouping, INDEX.md section layout, archive triggers, blocking logic, dashboard display, and AI context generation all pattern-match on specific status values.

The industry-standard solution is the **status category pattern**: a fixed set of system-defined categories that custom status names map to. System logic operates on categories (e.g., "is this item completed?"), while users work with their custom status names (e.g., "deployed"). This provides flexibility without breaking the system's ability to reason about workflow state.

### Status categories (6, fixed)

| Category | Semantic meaning | Board behavior | Styling | Default statuses |
|----------|-----------------|---------------|---------|-----------------|
| `draft` | Not ready, needs work | Not shown | Muted | draft |
| `backlog` | Ready to work, not scheduled | Not shown | Normal | backlog |
| `planned` | Scheduled, on the board | Column | Normal | planned |
| `active` | Being worked on | Column | Accent | in-progress |
| `completed` | Done | Column | Success | done |
| `cancelled` | Rejected/abandoned | Under done column | Muted | cancelled |

Each category has distinct system behavior — no two can be merged without losing functionality. Categories are internal (never appear in frontmatter). Statuses are user-facing (frontmatter, dropdowns, CLI). Users add custom statuses to whichever category matches their semantic meaning — e.g., `in-review` → active, `deployed` → completed, `triage` → draft, `groomed` → backlog.

### Current semantic dependencies on TaskStatus

| Behavior | Current dependency | Category replacement |
|----------|-------------------|---------------------|
| Epic progress bars | `== TaskStatus::Done` | `category == Completed` |
| Backlog INDEX sections | Hardcoded section per variant | Section per category, items grouped by status within |
| Archive `--all-done` | `Done \| Cancelled` | `Completed \| Cancelled` |
| Blocking logic | `depends_on` items not `Done` | `category != Completed` |
| Dashboard "In Progress" | `== TaskStatus::InProgress` | `category == Active` |
| AI context generation | Different sections per status | Sections per category |
| Summary API counts | Named fields per status | Dynamic counts keyed by category |
| CLI colorization | Status string → color | Category → color (custom statuses inherit) |
| Web UI kanban columns | Hardcoded 3-column array | Columns from `planned` + `active` + `completed` categories; cancelled under completed |
| Web UI list view | Shows draft + backlog items | Shows `draft` + `backlog` category items with category-inherited styling |
| Recently Done section | `== TaskStatus::Done` | `category == Completed` |
| Muted styling | Hardcoded for draft | `category == Draft \|\| category == Cancelled` |

## Acceptance Criteria

### Config schema
- [ ] `config.yaml` supports `workflows.task` section mapping categories to custom status lists
- [ ] Default config provides the current 6 statuses mapped to the 6 categories
- [ ] `markplane init` scaffolds the default workflow config
- [ ] Each status is a kebab-case string; each maps to exactly one category
- [ ] Display order within a category is implicit in the YAML list order

### Core
- [ ] `StatusCategory` enum added with 6 variants (draft, backlog, planned, active, completed, cancelled) — hardcoded, not configurable
- [ ] `TaskStatus` enum replaced with `String` field validated against config
- [ ] `config.category_of(status: &str) -> StatusCategory` lookup method on Project/Config
- [ ] All semantic logic refactored to use categories instead of specific status values
- [ ] `create_task()` and `update_task()` validate status against config
- [ ] Reading files with unknown statuses succeeds (graceful degradation)

### INDEX.md generation
- [ ] Backlog INDEX groups tasks by category, with per-status subsections within each category
- [ ] Roadmap INDEX epic progress uses `category == Completed` for done count
- [ ] Recently Done section uses `category == Completed`

### Blocking logic
- [ ] `find_blocked_items()` uses `category != Completed` instead of `!= TaskStatus::Done`

### Archive
- [ ] `--all-done` archives items where `category == Completed || category == Cancelled`

### CLI
- [ ] CLI `--status` validation reads from config
- [ ] Error messages list available statuses from config
- [ ] Status colorization maps category → color (custom statuses inherit their category's color)
- [ ] Align CLI colors with web UI colors (web UI is source of truth; notably `in-progress` is blue in web UI but yellow in CLI)

### MCP
- [ ] MCP tool schemas dynamically list configured status values in descriptions
- [ ] MCP `instructions` field lists available statuses grouped by category

### Web UI
- [ ] API endpoint exposes workflow config (statuses with category mappings)
- [ ] `TaskStatus` TypeScript type becomes `string`
- [ ] Kanban columns built from config — one column per status (or per category, configurable)
- [ ] Status dropdowns in create dialog and detail sheet populate from config
- [ ] Status badges inherit category-based colors/icons
- [ ] Summary counts use categories instead of hardcoded field names
- [ ] Workflow configuration UI in settings (add/remove/reorder statuses within category buckets)

### Backward compatibility
- [ ] All existing tests pass with default config values
- [ ] Projects without workflow config in `config.yaml` use built-in defaults
- [ ] `markplane check` reports items with statuses not in the current config

## Notes

### Config format

```yaml
workflows:
  task:
    draft:        [draft]          # user adds: triage, idea
    backlog:      [backlog]        # user adds: groomed, refined
    planned:      [planned]        # user adds: ready, next-up
    active:       [in-progress]    # user adds: in-review, in-qa
    completed:    [done]           # user adds: deployed, verified
    cancelled:    [cancelled]      # user adds: wont-fix, duplicate
  # epic, plan, note statuses stay fixed (hardcoded enums)
```

The first status in the earliest category (`draft`) is the default for new tasks. Only task statuses are configurable — epic, plan, and note statuses remain as hardcoded enums since they have few values and deliberate lifecycle semantics.

### Implementation approach

Consider a phased approach: first add the `StatusCategory` enum and config parsing with defaults matching current `TaskStatus` variants, then refactor semantic logic to use `config.category_of()` instead of direct enum matches. The `TaskStatus` enum can be removed last once all consumers use the category lookup.

A `ValidatedStatus(String)` newtype pattern could help maintain type safety after removing the enum. The MCP `instructions` field ([[TASK-eduur]]) should dynamically list available statuses once this lands.

### Preserving existing styling

The current visual identity for the default 6 statuses must be preserved exactly in the transition to categories. Custom statuses inherit their category's default icon; per-status icon selection is a future enhancement.

**Source of truth for current styling:**

- **Web UI colors**: `globals.css` — CSS custom properties `--status-{name}` (lines 114-120 light, 203-209 dark). Status badges use these via `var(--status-{status})` in `status-badge.tsx`.
- **Web UI icons**: `constants.ts` — `STATUS_CONFIG` record maps each status to a Lucide icon (lines 15-22).
- **Backlog list draft styling**: `backlog-content.tsx` — `BacklogRow` applies amber background to draft items: `bg-amber-50 border-amber-200/60` light / `bg-amber-950/30 border-amber-800/40` dark (lines 989-992). Draft rows also show a `StatusBadge`; backlog rows do not.
- **Board cancelled subsection**: `backlog-content.tsx` — Cancelled items render as a subsection under the Done column with muted heading (lines 628-646).
- **Board demote button**: `backlog-content.tsx` — Planned column cards have a "Send to Backlog" button (line 516).
- **CLI colors**: `formatting.rs` — `colorize_status()` maps status strings to terminal colors (lines 15-26).

**For custom statuses**, the category provides defaults:
- Color: custom statuses use their category's CSS color variable (e.g., a new `in-review` status in the `active` category uses `--status-in-progress` color, or a new `--status-active` category-level variable).
- Icon: custom statuses use their category's default icon (e.g., `active` → LoaderCircle).
- Row styling: custom statuses in the `draft` category inherit the amber background treatment.

### Key files requiring changes

- `models.rs` — Remove `TaskStatus` enum, add `StatusCategory` enum, change `Task.status` to `String`
- `project.rs` — Add `category_of()` method, validate status in create/update
- `index.rs` — Refactor all 6 hardcoded kanban sections to category-driven loop (~100 lines)
- `context.rs` — Refactor status-specific sections to category-based filtering
- `query.rs` — Status filtering/sorting via config lookup
- `archive.rs` — Terminal state detection via category
- `dashboard.rs` — Active work detection via category
- `mcp/tools.rs` — Dynamic schema descriptions
- `serve.rs` — Dynamic summary counts, config endpoint
- Web UI: `types.ts`, `constants.ts`, `backlog-content.tsx` (kanban columns), `task-detail-sheet.tsx`, `create-dialog.tsx`

## References

- [[TASK-eyg6c]] — Prerequisite: configurable item types and note types (simpler, same config pipeline)
- [[TASK-ict2n]] — Related: effort point mapping (independent)
