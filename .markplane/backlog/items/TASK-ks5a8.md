---
id: TASK-ks5a8
title: Convert position.rs panics to Result error handling
status: backlog
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
updated: 2026-03-02
---

# Convert position.rs panics to Result error handling

## Description

Multiple functions in `crates/markplane-core/src/position.rs` use `panic!()` and `assert!()` on invalid input. Since the `position:` field lives in user-editable YAML frontmatter, a hand-edited or corrupted value will crash the process on `markplane move` or via the web server's drag-and-drop.

**Panic paths in position.rs** (High)
- `digit_value()` (line 47): `panic!()` on unexpected byte values
- `get_integer_length()` (line 59): `panic!()` on unexpected head character
- `get_integer_part()` (line 65): indexes `key.as_bytes()[0]` with no empty-string guard
- `validate_order_key()` (lines 73-81): `assert!()` macros

**`.expect()` in `move_item()`** (High)
At `project.rs:918-919`, `move_item()` uses `.expect("moved task must be in list")`. A concurrent web request that deletes the task between the read and find causes a server crash. Replace with `.ok_or_else(|| MarkplaneError::NotFound(...))?`.

All panic paths should be converted to return `Result<_, MarkplaneError>` and errors propagated through `move_item()` and callers.

## Acceptance Criteria

- [ ] No `panic!()`, `assert!()`, or `.expect()` in `position.rs` — all return `Result`
- [ ] `move_item()` uses `?` instead of `.expect()`
- [ ] Malformed `position:` values in frontmatter produce a user-facing error, not a crash
- [ ] All existing tests pass
- [ ] New tests cover invalid position key inputs

## References

- Source: Pre-release audit (2026-03-02)
