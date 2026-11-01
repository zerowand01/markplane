---
id: TASK-tretc
title: Archive strips relationship links from archived and linked items
status: planned
priority: high
type: bug
effort: medium
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- archive
- data-integrity
- references
position: a9
created: 2026-11-01
updated: 2026-11-01
---

# Archive strips relationship links from archived and linked items

## Description

`markplane archive` empties `depends_on`, `blocks`, `related`, `plan` and `implements` on the archived item and removes the archived ID from those fields on every active item that pointed at it. The active items' `updated` date is bumped. `unarchive` moves the file back but restores nothing. Reported by a user of markplane 0.1.7 via the CLI and the `markplane_archive` MCP tool; reproduced on master.

```
markplane init
markplane add "Alpha task"          # TASK-A
markplane add "Beta task"           # TASK-B
markplane link TASK-A TASK-B -r related
markplane link TASK-B TASK-A -r depends-on
markplane update TASK-A --status done
markplane archive TASK-A
# TASK-A (archive/) and TASK-B both now read depends_on [], blocks [], related []
markplane unarchive TASK-A
# both still empty
```

### Root cause

`archive_item()` in `crates/markplane-core/src/project.rs` calls `find_inbound_references()` and then `link_items(..., LinkAction::Remove)` for each hit. Because `link_items` maintains reciprocals, the removal also wipes the archived file's own side of every link. Only the archived task's `epic` field survives, since epics hold no back-reference.

Added in commit 0944e0e (TASK-p2tub, March 2026 pre-release audit). The audit saw an open task show as "blocked by" a done, archived dependency and fixed it by mutating the data instead of the view. Documented in `docs/cli-reference.md` (archive section) and `docs/architecture.md` ("Archive reference cleanup"), but not in the CLI help, CLI output or MCP tool description.

### Why it is wrong

Archive is a filing move, not a deletion. The current behaviour contradicts four of the project's own contracts:

- `docs/file-format.md` states "Archive and unarchive do not change timestamps." Linked items get `updated` bumped.
- Files the user did not name are edited, so `archive --all-done` produces unexplained diffs across the active backlog.
- It is lossy and irreversible. `markplane link` accepts an archived target today and `check` is happy with it, yet archiving destroys the same link.
- Archiving a done epic clears `epic:` on its tasks and archiving a done plan empties its `implements` list. Epic and plan membership is read by scanning tasks, so these never had a dangling-reference problem. Pure provenance loss.

The web dashboard (`get_dashboard` in `serve.rs`) already computes blocked items over all tasks including archived ones and needs no stripping, which demonstrates the correct design.

### What actually breaks if the links stay (verified)

- `find_blocked_items()` is fed active tasks only by most callers, so a done archived dependency is not in the done set and the open task shows as blocked in `dashboard`, backlog INDEX, `.context/summary.md` and the MCP `blocked` resource.
- Backlog INDEX renders the blocker as `../backlog/items/TASK-x.md`, which does not exist because `cross_link()` and `item_link()` never check `archive/`.
- `check` passes: `validate_references` resolves archived IDs via `item_path()`.
- The web graph already drops edges whose target node is absent (`graph-view.tsx`). Nothing to do.

### Alternatives considered

- Keep stripping and warn: still lossy. Rejected.
- Opt-in `--unlink` flag: keeps a destructive path with no use case once views are fixed. Rejected.
- Strip then restore on unarchive via a journal: hidden state in a system whose point is that the file is the record. Rejected.
- Archive is a pure move and views resolve archived targets: the only option consistent with every contract above. Chosen.

## Implementation Plan

1. **Core**: remove the inbound-reference cleanup from `archive_item()`. Delete `find_inbound_references()` and the two tests `test_archive_cleans_inbound_blocks` / `test_archive_cleans_inbound_related` in `references.rs`, which exist only to serve it. Add a test asserting archive and unarchive leave every linked file byte-identical.
2. **Blocked detection**: change `find_blocked_items()` to resolve dependencies against all tasks (active and archived) while keeping candidates to active tasks. Status-aware, not "absent means done": a cancelled dependency must keep blocking after it is archived, exactly as it does while active. Update all nine callers (`dashboard.rs`, `index.rs`, `context.rs` x2, `serve.rs`, `mcp/resources.rs`, `project.rs` tests). Several callers already hold the all-scope list.
3. **INDEX links**: make `cross_link()` and `item_link()` in `index.rs` emit `archive/` paths when the target lives there.
4. **Docs**: rewrite the archive paragraph in `docs/cli-reference.md` and the "Archive reference cleanup" bullet in `docs/architecture.md` to describe archive as a pure move. Note in the MCP tool description that links are preserved.
5. No data version bump: the file format is unchanged. Items archived under the old behaviour cannot be repaired automatically; the release note should say so and point at git history.

## Acceptance Criteria

- [ ] `markplane archive <ID>` changes nothing except the file location; linked items are byte-identical before and after
- [ ] `markplane unarchive <ID>` is a pure inverse; no repair step needed
- [ ] Archiving a done epic or plan leaves `epic:`, `plan:` and `implements` on its tasks intact
- [ ] An active task that depends on a done, archived task is not reported as blocked by `dashboard`, backlog INDEX, `.context/summary.md`, MCP `blocked` resource or `/api/dashboard`
- [ ] An active task that depends on a cancelled, archived task is still reported as blocked
- [ ] Backlog INDEX and roadmap INDEX links to archived items point at `archive/<ID>.md`
- [ ] `markplane check` reports no broken references or asymmetric links for links to archived items
- [ ] `find_inbound_references()` and its tests are removed
- [ ] `docs/cli-reference.md`, `docs/architecture.md` and the `markplane_archive` MCP description no longer describe link cleanup
- [ ] `cargo test --workspace` passes and `cargo clippy --workspace` is warning-free

## References

- Introduced by commit 0944e0e, TASK-p2tub (archived)
- `docs/file-format.md` "Archive and unarchive do not change timestamps"
