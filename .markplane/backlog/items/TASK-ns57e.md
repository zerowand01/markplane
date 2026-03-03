---
id: TASK-ns57e
title: Write path safety - atomic writes and file locking
status: backlog
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
updated: 2026-03-02
---

# Write path safety - atomic writes and file locking

## Description

Item file writes are neither atomic nor locked, creating two classes of data integrity risk.

**No file locking on `write_item()`** (High)
At `project.rs:518-522`, the read-modify-write cycle has no locking. Two concurrent web API requests can overwrite each other's changes (lost update). `fs2` is already a dependency (used in `next_id()`). Add `fs2::FileExt::lock_exclusive()` around write operations.

**Non-atomic `write_item()`** (Medium)
`fs::write()` truncates then writes. A crash mid-write leaves a corrupted/truncated file. Compare with `write_new_file()` which correctly uses `File::create_new()`. Write to a tempfile in the same directory, then `fs::rename()` (atomic on same filesystem).

**Non-atomic two-file link updates** (Medium)
At `links.rs:196-201`, `link_items()` writes file A then file B sequentially. A crash between writes leaves asymmetric reciprocal links. The `check` command can detect and repair this, but prevention is better. Write both to tempfiles then rename.

The locking and atomicity fixes should be designed together — the tempfile+rename happens inside the locked section.

## Acceptance Criteria

- [ ] `write_item()` uses advisory file locking (`fs2`) to prevent concurrent overwrites
- [ ] `write_item()` writes to a tempfile then renames for crash safety
- [ ] `link_items()` two-file writes are both atomic (tempfile + rename)
- [ ] `write_new_file()` behavior unchanged (already uses `File::create_new()`)
- [ ] All existing tests pass

## References

- Source: Pre-release audit (2026-03-02)
- `fs2` already used in `next_id()` for config file locking
