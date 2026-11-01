---
id: TASK-kfmin
title: Web graph omits dependency edges to archived blockers
status: planned
priority: medium
type: bug
effort: small
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- web-ui
- graph
- archive
position: aD
created: 2026-11-01
updated: 2026-11-01
---

# Web graph omits dependency edges to archived blockers

## Description

`build_graph()` in `crates/markplane-cli/src/commands/serve.rs` loads active tasks only and emits dependency edges solely from each loaded task's `blocks` field. When the blocker has been archived it is never loaded, so no edge is generated at all. The web dashboard (which resolves `depends_on` against all tasks since [[TASK-tretc]]) reports the task as blocked, while the graph shows it as free.

Before [[TASK-tretc]], archiving stripped the link, so both views were wrong in the same way. Now the dashboard is right and the graph silently disagrees with it. Found during the review of that change.

## Steps to Reproduce

1. `markplane add "Blocker"` (TASK-A), `markplane add "Blocked"` (TASK-B)
2. `markplane link TASK-B TASK-A -r depends-on`
3. `markplane update TASK-A --status cancelled` and `markplane archive TASK-A`
4. `markplane serve`, open the dashboard: TASK-B is listed as blocked by TASK-A
5. Open the graph page

## Expected Behavior

The graph shows TASK-A (visibly archived, e.g. dimmed) with a `blocks` edge to TASK-B, so the two views agree.

## Actual Behavior

TASK-A is absent and TASK-B has no incoming edge.

## Notes

Suggested fix: in `build_graph()`, collect the `depends_on` targets of active tasks that are not among the loaded nodes, load those from `archive/`, add them as nodes flagged `archived: true`, and emit the edge. Alternatively emit the edge from `depends_on` whenever the blocker is not a loaded node and let `graph-view.tsx` render a placeholder node. Do not load the whole archive by default; large archives would swamp the graph.

The `graph-view.tsx` filter at the `allowedNodes` check already drops edges whose endpoint is missing, so nothing dangles today; the fix is purely additive.

## Acceptance Criteria

- [ ] An active task whose `depends_on` points at an archived task shows that edge in `/api/graph` and in the graph page
- [ ] Archived nodes added this way are distinguishable from active nodes
- [ ] Archived tasks not referenced by any active item are still excluded
- [ ] Focused graph (`/api/graph/{id}`) behaves the same way
- [ ] `cargo test --workspace` and `cargo clippy --workspace` pass; UI typecheck and lint pass

## References

- [[TASK-tretc]] (archive preserves links)
- `crates/markplane-cli/src/commands/serve.rs` `build_graph()`
- `crates/markplane-web/ui/src/components/domain/graph-view.tsx`
