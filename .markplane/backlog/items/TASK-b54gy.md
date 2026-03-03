---
id: TASK-b54gy
title: Add Sprint entity type
status: backlog
priority: low
type: feature
effort: xl
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- core
position: a4
created: 2026-02-10
updated: 2026-02-28
---

# Add Sprint entity type

## Description

Add a Sprint entity type to Markplane — a minimal first-class entity for time-boxed iterations, following the Epic pattern. Sprints are the "when" — a time-boxed container that groups tasks into a focused work period. Epics are "what" (strategic goals), plans are "how" (implementation details), sprints are "when" (time-boxed commitment).

Sprints are opt-in via config. Teams that prefer continuous flow never see sprint UI or fields.

## Design

### Data Model

Minimal first-class entity following the Epic pattern exactly:

**Sprint frontmatter:**

```yaml
id: SPRINT-xxxxx
title: Sprint 1
status: planning          # planning | active | done
start_date: 2026-02-24
end_date: 2026-03-06
created: 2026-02-28
updated: 2026-02-28
```

No priority, tags, goals, or related in frontmatter. Sprints are temporal containers — their only structural relationship is to tasks. Goals, retro notes, and context are free-form body content (markdown). Cross-references to epics or other items use `[[ID]]` wiki-links in the body.

**Task linkage** — follows the Epic pattern:

| Aspect | Epic (existing) | Sprint (new) |
|--------|-----------------|--------------|
| Task field | `epic: Option<String>` | `sprint: Option<String>` |
| Link relation | `LinkRelation::Epic` | `LinkRelation::Sprint` |
| Membership | Derived by scanning tasks | Same |
| Progress | Computed (task_count, done_count, etc.) | Same |
| Replacement | `link_items()` clears old epic when setting new one | Same |
| Items list on parent | None | None |

**Constraint**: Only one sprint can be `active` at a time. Multiple `planning` sprints are fine. Enforced at the core level on status change.

**Directory**: `.markplane/sprints/` with `items/` and `archive/` subdirs.

### Opt-in Config

```yaml
features:
  sprints: true    # default: false
```

When disabled: no `sprints/` directory, no `sprint` field in task templates, no sprint UI elements, no sprint MCP tools advertised, CLI sprint commands hidden or return guidance to enable.

When enabled: full sprint support across CLI, MCP, and web UI.

### Web UI — Board

- Sprint added as a filter in the filter bar (like priority, epic, assignee)
- Single-select (one sprint at a time)
- When an active sprint exists, the board auto-selects it as the default filter on page load
- When a sprint filter is active, a sprint header appears above columns showing: title, date range, days remaining, progress (done/total)
- User can clear the filter to see all board items
- Cards show a sprint indicator when viewing unfiltered

### Web UI — Backlog

- Backlog always shows all backlog/draft items (no behavioral change to current functionality)
- Sprint selector at the top (alongside existing filter menus) to select a sprint (single-select, one at a time)
- Selecting a sprint reveals its planning zone above the existing "Drop here to move to Board" zone
- Planning zone shows: sprint title, dates, current tasks in that sprint, and a drop target
- Drag task into sprint planning zone → status becomes `planned` + `sprint` field set to that sprint
- "Drop here to move to Board" zone unchanged — promotes to `planned`, no sprint assignment

### Web UI — Dashboard

- When sprints enabled and one is active, a "Sprint" section appears prominently (above Active Work)
- Shows: sprint name, date range, progress bar, remaining task count
- When no sprint is active, section doesn't appear (no empty-state clutter)

### Web UI — Sprint Page

- Dedicated `/sprints` page showing list of sprints
- Active sprint at top, then planning, then recent done
- Each with progress bar and date range
- Sprint detail sheet for full view (tasks, body content, progress)

### Sprint Completion Flow

- When a sprint is marked `done`, incomplete tasks keep their `sprint` field (historical record)
- Task statuses don't change automatically — no magic, no surprises
- User explicitly reassigns incomplete tasks to the next sprint or clears the sprint field

### Sprint Dates

- Not auto-enforced — Markplane doesn't run a scheduler
- If end_date passes, the sprint doesn't auto-complete
- The user explicitly marks it done (file-based, user-driven, consistent with everything else)

## Acceptance Criteria

### Core (`markplane-core`)
- [ ] `Sprint` struct with frontmatter: `id`, `title`, `status` (`SprintStatus`), `start_date`, `end_date`, `created`, `updated`
- [ ] `SprintStatus` enum: `planning`, `active`, `done`
- [ ] `IdPrefix::Sprint` variant — `SPRINT-xxxxx` IDs, `.markplane/sprints/` directory
- [ ] `sprint: Option<String>` field on `Task`
- [ ] `LinkRelation::Sprint` variant — follows Epic pattern (replacement cleanup, reciprocals via `link_items()`)
- [ ] Sprint CRUD methods on `Project` (create, read, update, delete, scan)
- [ ] Sprint progress computed at query time (task_count, done_count, progress, status_breakdown)
- [ ] One-active-sprint constraint enforced on status change
- [ ] Sprint INDEX.md generation (active sprint with task table, planning sprints, recent done)
- [ ] Context generation updated to include sprint section when active
- [ ] `QueryFilter` extended with `sprint` filter
- [ ] Opt-in gated by `features.sprints` config

### CLI (`markplane-cli`)
- [ ] `markplane sprint create "Sprint 1" --start 2026-02-24 --end 2026-03-06`
- [ ] `markplane sprint start SPRINT-xxx` (transitions to active, enforces one-active constraint)
- [ ] `markplane sprint done SPRINT-xxx`
- [ ] `markplane sprint show SPRINT-xxx` (displays sprint with task statuses and progress)
- [ ] `markplane sprint list` (active, planning, recent done)
- [ ] `markplane link TASK-xxx SPRINT-yyy -r sprint` works via existing link command
- [ ] `markplane update TASK-xxx --sprint SPRINT-yyy` works via existing update command
- [ ] Sprint commands gated by config (guidance message when disabled)

### MCP
- [ ] Sprint management tools (create, start, done) — gated by config
- [ ] Existing `markplane_query`, `markplane_show`, `markplane_update` extended for sprints
- [ ] `sprint` filter param on `markplane_query`
- [ ] Sprint resource for AI context

### Web UI
- [ ] Sprint filter on board (single-select, auto-defaults to active sprint)
- [ ] Sprint header on board when sprint filter active (title, dates, days remaining, progress)
- [ ] Sprint planning zone on backlog (single-select sprint selector, drag-to-assign)
- [ ] "Drop here to move to Board" zone unchanged
- [ ] Dashboard sprint section (when active sprint exists)
- [ ] `/sprints` page with sprint list and progress bars
- [ ] Sprint detail sheet
- [ ] Sprint badge on task cards
- [ ] All sprint UI hidden when `features.sprints` is disabled

## Notes

- Purely additive to the existing data model — no changes to existing fields or behaviors
- Existing task files don't need migration — `sprint` field is optional, defaults to null when absent
- A task can belong to both a sprint and an epic simultaneously (epic = strategic goal, sprint = time commitment)
- Sprint templates added to `templates/` when feature is enabled
