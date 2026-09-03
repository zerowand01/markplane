---
id: TASK-jcijm
title: normalize_positions writes items without holding a lock
status: backlog
priority: low
type: bug
effort: xs
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- core
position: a8
created: 2026-09-02
updated: 2026-09-02
---

# normalize_positions writes items without holding a lock

## Description

`Project::normalize_positions()` (`crates/markplane-core/src/context.rs:558`)
performs a read-modify-write cycle on task files without taking an advisory
lock:

```rust
let tasks = self.list_tasks(...)?;      // read
updated_doc.frontmatter.position = ...; // modify
self.write_item(&doc.frontmatter.id, &updated_doc)?;  // write — no lock
```

This contradicts the documented contract on `write_item`:

> The caller is responsible for holding an advisory lock if this is part of a
> read-modify-write cycle — see `lock_item` and `lock_items`.

Every other mutation path (`update_status`, the four typed `update_*`
methods, `link_items`) takes the lock. This one does not.

The exposure is narrow but real: `markplane sync` calls
`normalize_positions()` (`commands/sync.rs:9`), and sync runs automatically
at `markplane mcp` and `markplane serve` startup. A server starting up can
therefore race a concurrent `markplane update` from the CLI, and the loser's
write is silently lost.

Found while explaining why `markplane sync` does not create `.markplane/.locks/`
— it never acquires a lock, so it never creates one.

## Acceptance Criteria

- [ ] `normalize_positions()` holds an advisory lock for each item it rewrites,
      or documents explicitly why it does not need one
- [ ] The locking discipline is consistent across every path that calls
      `write_item` on an existing item
- [ ] Existing tests still pass on both CI platforms

## Notes

- Deadlock ordering matters if locking several items: `lock_items()` sorts IDs
  for exactly this reason. `normalize_positions` walks tasks grouped by
  priority, so it should either lock per-item as it goes or use
  `lock_items()`.
- Low severity for single-user CLI use; the concurrency window only opens with
  a server running alongside CLI commands.
- Related: [[TASK-8y5pq]] touches the same locking code and should land first
  or be sequenced with this.

## References

- [[TASK-8y5pq]]
