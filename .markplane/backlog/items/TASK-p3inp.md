---
id: TASK-p3inp
title: Fix append_position using count instead of max existing position
status: backlog
priority: high
type: bug
effort: xs
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- position
- backlog
- web-ui
position: a3x
created: 2026-03-02
updated: 2026-03-02
---

# Fix append_position using count instead of max existing position

## Description

`append_position()` in `crates/markplane-core/src/project.rs` computes new task positions using `index_to_key(count)` where `count` is the number of existing tasks in the priority group. After any task is deleted, archived, or moved to a different priority, the count shrinks, causing the next created task to get a position key that already exists in that group.

Runtime error in web UI: `a3 >= a3` from `generateKeyBetween` in `fractional-indexing` when dragging tasks with duplicate positions.

## Steps to Reproduce

1. Create 3 tasks in the same priority group (positions: a0, a1, a2)
2. Archive or delete one task (e.g., the one at a1)
3. Create a new task in the same priority group — it gets `index_to_key(2)` = `a2`, duplicating the existing position
4. Drag a task adjacent to the duplicate in the backlog list view

## Root Cause

`append_position()` uses task count instead of finding the max existing position in the group.

## Fix

Replace `index_to_key(count)` with `generate_key_between(max_position, None)` — find the maximum existing position in the group and generate a key after it.

```rust
fn append_position(&self, priority: &Priority) -> Result<String> {
    let tasks = self.list_tasks(&crate::query::QueryFilter::default())?;
    let max_pos = tasks
        .iter()
        .filter(|t| &t.frontmatter.priority == priority)
        .filter_map(|t| t.frontmatter.position.as_deref())
        .max();
    Ok(crate::position::generate_key_between(max_pos, None)
        .expect("generate_key_between(max, None) cannot fail"))
}
```

## References

- `crates/markplane-core/src/project.rs:243` — `append_position()`
- `crates/markplane-core/src/position.rs` — fractional indexing implementation
- `crates/markplane-web/ui/src/app/backlog/backlog-content.tsx:866` — crash site
