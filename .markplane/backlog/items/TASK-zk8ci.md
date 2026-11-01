---
id: TASK-zk8ci
title: Detect duplicate IDs across items/ and archive/ and refuse to overwrite on archive
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
- check
- archive
- data-integrity
position: aE
created: 2026-11-01
updated: 2026-11-01
---

# Detect duplicate IDs across items/ and archive/ and refuse to overwrite on archive

## Description

Nothing detects an item whose file exists in both `{dir}/items/` and `{dir}/archive/`. `next_id()` checks both directories, so fresh IDs never collide; the state arises from a merge that resolves a rename-versus-edit conflict by keeping both files, or from a manual copy. Once it exists every view disagrees and `markplane archive` destroys data. Found and reproduced during the review of [[TASK-tretc]].

## Steps to Reproduce

1. `markplane init --empty`, `markplane epic "Epic"` (EPIC-E), `markplane add "Dup" --epic EPIC-E` (TASK-D)
2. `cp .markplane/backlog/items/TASK-D.md .markplane/backlog/archive/TASK-D.md` and set the archived copy's status to `done`
3. `markplane check`
4. `markplane sync` and read `roadmap/INDEX.md`
5. `markplane archive TASK-D`

## Expected Behavior

- Step 3: `check` reports TASK-D as present in both `items/` and `archive/` and exits non-zero.
- Step 5: `archive` refuses because `archive/TASK-D.md` already exists.

## Actual Behavior

- `check` passes with no warning.
- The roadmap INDEX lists TASK-D twice and reports the epic as 1/2 done (50%); `show` resolves to the active copy while the INDEX links to the archived one.
- `archive` prints `Archived TASK-D` and `fs::rename` silently overwrites the archived file. The archived version is lost.

## Notes

Two small, independent fixes:

1. `check`: after the existing scans, compare file stems across `items/` and `archive/` for each entity directory and report any overlap as a new validation category. No `--fix` for this; the user must choose which copy wins.
2. `archive_item()` in `crates/markplane-core/src/project.rs`: return an error if the destination path already exists, before the rename. Consider the same guard in `unarchive_item()`.

`item_path()` prefers `items/`, `LinkResolver` in `index.rs` prefers `archive/`, and `load_frontmatter_by_id()` in `references.rs` lets the archived copy overwrite the active one. Those do not need to change once the state is detected and prevented.

## Acceptance Criteria

- [ ] `markplane check` reports an ID whose file exists in both `items/` and `archive/` of the same directory, for all four entity types, and exits non-zero
- [ ] `markplane archive <ID>` fails with a clear error when `archive/<ID>.md` already exists and leaves both files untouched
- [ ] `markplane unarchive <ID>` fails the same way when `items/<ID>.md` already exists
- [ ] Unit tests for the check and for both guards
- [ ] `docs/cli-reference.md` check section lists the new validation
- [ ] `cargo test --workspace` and `cargo clippy --workspace` pass

## References

- [[TASK-tretc]] (archive preserves links)
- `crates/markplane-core/src/project.rs` `archive_item()`, `unarchive_item()`, `item_path()`
- `crates/markplane-cli/src/commands/check.rs`
