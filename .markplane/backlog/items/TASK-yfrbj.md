---
id: TASK-yfrbj
title: Performance optimizations
status: backlog
priority: medium
type: enhancement
effort: large
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- performance
- pre-release
position: a2
created: 2026-03-02
updated: 2026-03-02
---

# Performance optimizations

## Description

Performance bottlenecks identified in the pre-release audit. None affect correctness, but they impact responsiveness — especially `sync_all()` which runs on startup and after file changes.

**`sync_all()` reads config ~9x and scans directories ~21x** (Performance)
In `context.rs` and `index.rs`, each sync sub-function independently calls `load_config()` and `list_tasks()`/`list_epics()`/etc. Load config and all item collections once at `sync_all()` entry and pass to sub-functions. Estimated ~80% sync time reduction.

**`create_task()` scans all tasks for position calculation** (Performance)
At `project.rs:242-249`, `append_position()` calls `list_tasks()` — full directory scan + parse of every task — just to count items in a priority group. Use glob count or maintain a lightweight counter.

**Search loads all items with full bodies into memory** (Performance)
At `serve.rs:2007-2178`, `get_search()` loads every item including markdown bodies, then does `to_lowercase().contains()` per field. Lowercase body once; search title/ID first for early return; consider frontmatter-only loading for non-body fields.

**Graph loads full bodies but only uses frontmatter** (Performance)
At `serve.rs:2258-2434`, `build_graph()` loads all entities with full markdown bodies but only reads frontmatter fields. Add a frontmatter-only list method that stops parsing after the `---` delimiter.

**`load_config()` called ~24x across core operations** (Feature Gap)
Beyond sync, normal operations also redundantly load and parse `config.yaml`. A config caching layer (load once, invalidate on file change) would benefit all operations.

## Acceptance Criteria

- [ ] `sync_all()` loads config and item collections once and passes to sub-functions
- [ ] `create_task()` position calculation doesn't require full task list scan
- [ ] Search uses frontmatter-only loading where possible; body lowercased once
- [ ] Graph endpoint uses frontmatter-only loading
- [ ] Measurable improvement in `markplane sync` time on a project with 50+ items

## References

- Source: Pre-release audit (2026-03-02)
