---
id: TASK-ks5a8
title: Convert position.rs panics to Result error handling
status: done
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
- crash
- position
- pre-release
position: a3
created: 2026-03-02
updated: 2026-03-03
---

# Convert position.rs panics to Result error handling

## Description

Multiple functions in `crates/markplane-core/src/position.rs` use `panic!()` and `assert!()` on invalid input. Since the `position:` field lives in user-editable YAML frontmatter, a hand-edited or corrupted value will crash the process on `markplane move` or via the web server's drag-and-drop.

**Production panic paths in position.rs** (High)
- `digit_value()` (line 47): `panic!()` on unexpected byte values — reachable via `midpoint()`, `increment_integer()`, `decrement_integer()`
- `get_integer_length()` (line 59): `panic!()` on unexpected head character — called from `get_integer_part()` which is called unconditionally in `generate_key_between()`
- `get_integer_part()` (line 65): indexes `key.as_bytes()[0]` with no empty-string guard — index panic on `""`
- `get_integer_part()` (line 67): `assert!(len <= key.len())` — runs in release mode

**Debug-only (no production fix needed)**
- `validate_order_key()` (lines 73-81): `assert!()` macros — only called inside `cfg!(debug_assertions)` at `generate_key_between()` lines 240-247, compiled out in release builds

**`.expect()` in `move_item()`** (High)
At `project.rs:918-919`, `move_item()` uses `.expect("moved task must be in list")`. Server code should not `.expect()` — replace with `.ok_or_else(|| MarkplaneError::NotFound(...))?`.

All panic paths should be converted to return `Result<_, MarkplaneError>` and errors propagated through `move_item()` and callers.

## Acceptance Criteria

- [ ] No `panic!()`, `assert!()`, or `.expect()` on production paths in `position.rs` — all return `Result` (debug-only assertions in `validate_order_key()` may remain)
- [ ] `move_item()` uses `?` instead of `.expect()`
- [ ] Malformed `position:` values in frontmatter produce a user-facing error, not a crash
- [ ] Malformed position keys via the web UI's drag-and-drop return HTTP 400/422, not HTTP 500
- [ ] All existing tests pass
- [ ] New tests cover invalid position key inputs

## Implementation Notes

**New error variant**: `MarkplaneError::InvalidPosition(String)` added to `error.rs`. Mapped to HTTP 422 (Unprocessable Entity) in `serve.rs` `map_core_error()`. Diagram in `docs/architecture.md` updated.

**position.rs changes** — all internal functions converted from panic to `Result`:
- `digit_value()` → `Result<usize>`
- `get_integer_length()` → `Result<usize>`
- `get_integer_part()` → `Result<&str>` (with empty-string guard)
- `midpoint()` → `Result<String>`
- `increment_integer()` / `decrement_integer()` → `Result<Option<String>>`
- `generate_key_between()` → `Result<Option<String>>` (public API change)

Module-local `type Result<T>` alias and `pos_err()` helper keep the code concise.

`validate_order_key()` retained as debug-only (assert-based) — unchanged per task spec.

**project.rs**: `.expect("moved task must be in list")` replaced with `.ok_or_else(|| MarkplaneError::NotFound(...))`.

**Tests**: 12 new test cases covering invalid inputs to `digit_value`, `get_integer_length`, `get_integer_part`, `increment_integer`, `decrement_integer`, and `midpoint`. All 429 tests pass, clippy clean.

## References

- Source: Pre-release audit (2026-03-02)
