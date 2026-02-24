---
id: TASK-b54gy
title: Add Sprint/Iteration entity type
status: draft
priority: low
type: feature
effort: large
tags:
- core
epic: EPIC-c5uem
plan: null
depends_on: []
blocks: []
assignee: null
position: a6V
created: 2026-02-10
updated: 2026-02-23
---

# Add Sprint/Iteration entity type

## Description

Markplane currently has four entity types: epics (strategic goals), tasks (work), plans (implementation details), and notes (research/ideas). There is no time-boxed grouping concept — no way to say "these 5 items are our focus for the next two weeks." A Sprint or Iteration entity would provide this, giving teams a way to plan capacity and track velocity over fixed time periods.

This was discussed as a future enhancement during design. Plans are the "how" for individual items; sprints are "when" — a time-boxed container that groups multiple tasks into a focused work period.

## Acceptance Criteria

- [ ] New `SPRINT-NNN` ID prefix and `sprints/` directory
- [ ] Sprint entity with frontmatter: `id`, `title`, `status`, `start_date`, `end_date`, `items` (list of TASK IDs), `goals`
- [ ] Sprint statuses: `planning → active → completed → retrospective`
- [ ] `markplane sprint create "Sprint 1" --start 2026-02-10 --end 2026-02-24`
- [ ] `markplane sprint add SPRINT-001 TASK-4c2mh TASK-7cucf` to assign items to a sprint
- [ ] `markplane sprint show SPRINT-001` displays sprint with item statuses and progress
- [ ] Dashboard shows active sprint summary
- [ ] MCP tools for sprint management
- [ ] INDEX.md and context generation updated to include sprints

## Notes

Keep sprints optional — many solo developers and small teams prefer continuous flow over time-boxed iterations. The sprint entity should be additive, not required. Consider whether sprint items should be a frontmatter list on the sprint or a `sprint` field on each task (or both, kept in sync). The design spec mentions this as a future capability.

## Analysis (2026-02-23)

### Merit Assessment

The gap is real — there is no time-boxed commitment mechanism ("these N items are our focus for the next 2 weeks"). However, Markplane already covers most organizational dimensions well: epics (strategic grouping, with upcoming Now/Next/Later horizons), 5-level priority, tags, effort sizes, dependency graph, and assignment. The missing piece is narrow.

The primary audience (solo devs and small teams using markdown-first, git-native tooling) mostly practices continuous flow, not Scrum-style sprints. This limits the feature's reach.

**Verdict**: Merited as a low-priority, additive feature — but not a core need for the primary audience.

### Concerns with Current Proposal

1. **A full entity type is heavyweight.** New directory, Rust struct, status enum, frontmatter schema, INDEX generation, context generation, 3+ CLI commands, 5+ MCP tools, web UI pages — significant ongoing maintenance for what is essentially a time-boxed tag with dates.

2. **The `items[]` sync problem.** All options have friction:
   - On sprint only: sprint file becomes a merge conflict magnet
   - On task only: sprint has no self-contained view; requires scanning all tasks
   - Both with sync: duplicates the epic/plan reciprocal complexity with more moving parts

3. **Over-modeled status workflow.** `planning → active → completed → retrospective` is four states for a time box. In practice a sprint is upcoming, active, or done. "Retrospective" is a meeting, not a sprint state.

### Alternative Approaches

**Option A: Tag-based sprints with config metadata (lightest)**

Sprint config in `config.yaml` (tag name, dates, goals). Tasks tagged `sprint-3`. CLI/MCP/web renders sprint views by querying tasks with that tag + reading config metadata. No new entity type, no sync problem, no new directory. ~80% of the value for ~10% of the effort.

**Option B: Sprint as a Note subtype**

Use the existing Note entity (`NOTE-NNN`, type: `sprint-plan`) with dates in the body, and tag tasks with `sprint-N`. The note captures goals/retro; the tag creates the grouping. Reuses existing infrastructure entirely.

**Option C: Minimal first-class entity (recommended if full sprint support is needed)**

Follow the epic pattern exactly:
- Only 2 states: `active` / `done`
- Tasks get a `sprint` field (like `epic`) — no `items[]` on the sprint side
- Sprint file is just frontmatter + goals/retro markdown body
- Sprint membership derived by scanning tasks (like epic membership already works)
- Reuse existing `link_items()` with a new `Sprint` relation

### Recommendation

Start with **Option A** — it fits Markplane's "progressive complexity" philosophy. If real demand emerges, graduate to **Option C**. The current proposal is over-scoped for a low-priority feature whose core audience mostly prefers continuous flow.
