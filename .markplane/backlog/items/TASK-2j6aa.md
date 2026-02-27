---
id: TASK-2j6aa
title: Align frontmatter and UI field ordering across all entity types
status: backlog
priority: medium
type: enhancement
effort: medium
tags:
- core
- web
epic: null
plan: null
depends_on:
- TASK-7s7u2
- TASK-e4yqc
blocks: []
related: []
assignee: null
position: Zzj
created: 2026-02-27
updated: 2026-02-27
---

# Align frontmatter and UI field ordering across all entity types

## Description

The Rust struct field order (which controls YAML frontmatter serialization) and the UI FieldRow order in detail sheets are inconsistent for Task, Plan, and Note. This causes confusion — the file and the UI show the same fields in different positions.

Align both sides to these canonical orderings:

- **Task**: `status, priority, type, effort, epic, plan, depends_on, blocks, related, assignee, tags`
- **Epic**: `status, priority, started, target, related, tags`
- **Plan**: `status, implements, related`
- **Note**: `status, type, tags, related`

Principle: status/workflow → classification → parent relationships → peer relationships → people → metadata.

Both struct field order in `models.rs` and FieldRow order in detail sheet components need updating.

## Acceptance Criteria

- [ ] Task struct field order matches canonical order; task-detail-sheet FieldRows match
- [ ] Epic struct field order matches canonical order; epic-detail-sheet FieldRows match
- [ ] Plan struct field order matches canonical order; plan-detail-sheet FieldRows match
- [ ] Note struct field order matches canonical order; note-detail-sheet FieldRows match
- [ ] Existing files reserialize correctly (round-trip test still passes)
- [ ] Web API response types match the canonical field order

## Notes

- Changing struct field order changes YAML serialization order — existing files will get reordered on next write, which is the desired effect
- Should be done after [[TASK-7s7u2]] and [[TASK-e4yqc]] to avoid reordering fields that are about to be removed

## References

- [[TASK-7s7u2]] — Remove depends_on from Epic
- [[TASK-e4yqc]] — Remove epic from Plan
