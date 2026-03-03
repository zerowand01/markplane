---
id: TASK-ns57e
title: Write path safety - atomic writes and file locking
status: planned
priority: high
type: enhancement
effort: large
epic: null
plan: null
depends_on: []
blocks:
- TASK-jejpq
related: []
assignee: null
tags:
- data-integrity
- pre-release
position: a3l
created: 2026-03-02
updated: 2026-03-03
---

# Write path safety - atomic writes and file locking

## Description

Item file writes are neither atomic nor locked, creating two classes of data integrity risk.

**No file locking on `write_item()`** (High)
At `project.rs:522-527`, the read-modify-write cycle has no locking. Two concurrent web API requests can overwrite each other's changes (lost update). Add `fs2` as a dependency and use `fs2::FileExt::lock_exclusive()` around write operations. The lock must wrap the entire read-modify-write cycle (i.e., at the `update_task()`/`update_epic()`/etc. level), not just the final `fs::write()` call, to prevent TOCTOU between read and write.

**Non-atomic `write_item()`** (Medium)
`fs::write()` truncates then writes. A crash mid-write leaves a corrupted/truncated file. Compare with `write_new_file()` which correctly uses `File::create_new()`. Write to a tempfile in the same directory, then `fs::rename()` (atomic on same filesystem).

**Non-atomic two-file link updates** (Medium)
At `links.rs:200-201`, `link_items()` writes file A then file B sequentially. A crash between writes leaves asymmetric reciprocal links. The `check` command can detect and repair this, but prevention is better. Write both to tempfiles then rename. For `link_items()`, lock both files in a deterministic order (e.g., lexicographic by ID) to prevent deadlocks when two concurrent requests lock the same pair of items in opposite order.

The locking and atomicity fixes should be designed together — the tempfile+rename happens inside the locked section.

## Acceptance Criteria

- [ ] Add `fs2` to `markplane-core/Cargo.toml` dependencies
- [ ] Advisory file locking (`fs2`) wraps the full read-modify-write cycle in `update_task()`, `update_epic()`, `update_plan()`, `update_note()`, and `update_status()`
- [ ] `write_item()` writes to a tempfile then renames for crash safety
- [ ] `link_items()` two-file writes are both atomic (tempfile + rename) with deterministic lock ordering to prevent deadlocks
- [ ] `write_new_file()` behavior unchanged (already uses `File::create_new()`)
- [ ] All existing tests pass

## Implementation Notes

- `tempfile` crate is already a dependency — use `NamedTempFile::new_in(parent_dir)` + `persist()` for atomic writes
- `fs2` needs to be added as a new dependency
- `next_id()` has no locking — it relies on random ID collision avoidance, which is acceptable given the 36^5 keyspace
- Per-item-file locks (lock the target `.md` file itself) are simplest and avoid global contention

## References

- Source: Pre-release audit (2026-03-02)
- Reviewed: 2026-03-03 — corrected `fs2` dependency status, line references, added lock ordering and lock scope guidance
