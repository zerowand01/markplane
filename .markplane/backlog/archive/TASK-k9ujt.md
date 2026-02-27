---
id: TASK-k9ujt
title: Fix fractional-indexing crash on uppercase-prefix position keys
status: done
priority: critical
type: bug
effort: medium
tags: []
epic: null
plan: null
depends_on: []
blocks: []
assignee: null
position: a0
related: []
created: 2026-02-26
updated: 2026-02-26
---

# Fix fractional-indexing crash on uppercase-prefix position keys

## Description

The Rust `position.rs` only implemented the "right half" of the Rocicorp fractional-indexing spec — lowercase prefixes `a`–`z` (values going up from `a0`). The npm `fractional-indexing` package used by the web UI kanban implements the full spec, including uppercase prefixes `A`–`Z` (values going down below `a0`). When a user dragged a task above the first item in the web UI, the npm package produced keys like `Zz`, `Zy`, `Zzx`. When Rust later encountered these keys in `move_item()`, `get_integer_part()` panicked on a `u8` underflow (`b'Z' - b'a'`), crashing the MCP server.

4 tasks had uppercase-prefix positions: TASK-9pugf (`Zy`), TASK-hw558 (`Zz`), TASK-us45u (`Zzx`), TASK-8xvxq (`ZzV`).

## Steps to Reproduce

1. In the web UI kanban, drag a task above the first item in a priority group
2. The npm package generates a `Z`-prefix position key (e.g. `Zz`)
3. Run any MCP or CLI operation that calls `get_integer_part()` on that key (e.g. `markplane_move`)
4. Rust panics on `b'Z' - b'a'` underflow

## Fix

Ported the full Rocicorp fractional-indexing algorithm from the npm package to Rust in `crates/markplane-core/src/position.rs`:

- **`get_integer_length(head)`** — New helper supporting both `a`–`z` (length = head-a+2) and `A`–`Z` (length = Z-head+2)
- **`get_integer_part(key)`** — Rewritten to use `get_integer_length` instead of broken `prefix - b'a' + 1`
- **`increment_integer(x)`** — Full carry-based port with `Z→a0` crossover and `z→None` ceiling
- **`decrement_integer(x)`** — Full borrow-based port with `a→Zz` crossover and `A→None` floor
- **`validate_order_key(key)`** — New validation helper used in debug assertions
- **`generate_key_between`** — Updated to match npm logic for uppercase range
- **Removed** `key_to_integer` (dead code after rewrite)

34 position tests (up from 20), all 380 workspace tests pass, clippy clean.
