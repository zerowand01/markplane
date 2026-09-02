---
id: TASK-8y5pq
title: Replace fs2 with std File::lock
status: backlog
priority: someday
type: chore
effort: small
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- dependencies
position: a5
created: 2026-09-02
updated: 2026-09-02
---

# Replace fs2 with std File::lock

## Description

`markplane-core` depends on `fs2` solely for advisory file locking
(`FileExt::lock_exclusive` in `project.rs`). The crate has had no release since
2019 and is effectively unmaintained.

The standard library gained equivalent functionality in Rust 1.89
(`File::lock`, `File::try_lock`, `File::unlock`). The workspace pins
`rust-version = "1.93.0"`, so this is available today with no MSRV change —
verified compiling against the pinned toolchain.

This removes a dependency outright rather than swapping one for another. The
call sites are `lock_item`/`lock_items` and their five callers, all within
`project.rs` and `links.rs`.

Context: the sidecar-lock rework in PR #35 (Windows `os error 33`) touches this
same code. Do that first; this is a mechanical follow-up on top of it.

## Acceptance Criteria

- [ ] `fs2` removed from `crates/markplane-core/Cargo.toml` and `Cargo.lock`
- [ ] All advisory locking uses `std::fs::File` methods
- [ ] Locking behavior unchanged on Linux, macOS, and Windows
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean

## Notes

- std API: `File::lock()`, `File::try_lock()`, `File::unlock()` — stabilized
  1.89.0 (`file_lock`).
- `fs2`'s `lock_exclusive()` returns `io::Result<()>`; the std equivalent
  matches, so error handling should map straight across.
- Sequence after PR #35 to avoid conflicting edits in `project.rs`.

## References
