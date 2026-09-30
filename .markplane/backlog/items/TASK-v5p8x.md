---
id: TASK-v5p8x
title: sync fails on fresh clone when an entity directory is missing
status: done
priority: high
type: bug
effort: small
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- core
- sync
position: a7
created: 2026-09-30
updated: 2026-09-30
---

# sync fails on fresh clone when an entity directory is missing

## Description

`markplane sync` fails with `IO error: No such file or directory (os error 2)`
when a whole entity directory (e.g. `.markplane/plans/`) is missing.

That is the normal state of a fresh clone for any entity type with no items:
the directory's only file is `INDEX.md`, which `.markplane/.gitignore` ignores,
and git does not track empty directories (`items/`, `archive/`). So the
directory never reaches the clone.

The `generate_*_index()` functions in `crates/markplane-core/src/index.rs` call
`fs::write(self.root().join("plans/INDEX.md"), ...)` without creating the
parent directory first. Sync runs automatically on `markplane mcp` and
`markplane serve` startup, so both are likely affected as well.

Reported by an agent using markplane in another project, who worked around it
by committing `.gitkeep` files into the archive folders and `plans/items`.

## Steps to Reproduce

1. `markplane init` in a new git repo
2. `rm -rf .markplane/plans` (simulates a clone where plans/ held only the gitignored INDEX.md)
3. `markplane sync`

Verified on 0.1.3. A missing `items/` or `archive/` alone does not trigger it,
and `markplane plan` / `markplane note` recreate their directories fine.

## Expected Behavior

`sync` creates any missing entity directories and completes normally.

## Actual Behavior

```
Syncing...
Error: IO error: No such file or directory (os error 2)
```

## Acceptance Criteria

- [x] `sync` succeeds when any of `backlog/`, `roadmap/`, `plans/`, `notes/` is missing
- [x] `.context/` is also created if missing (it is gitignored too)
- [x] `markplane mcp` and `markplane serve` start cleanly on a fresh clone
- [x] Integration test covering sync with missing entity directories

## Notes

- Fix in the tool rather than having `init` write `.gitkeep` files — existing
  projects and hand-created layouts should also work.
- Consider a single `ensure_layout()` helper on `Project` called at the top of
  `sync_all()`, rather than `create_dir_all` sprinkled across every generator.

## References
