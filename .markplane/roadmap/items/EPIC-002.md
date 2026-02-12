---
id: EPIC-002
title: "Developer Experience"
status: planned
priority: medium
started: null
target: null
tags: []
depends_on: []
---

# Developer Experience

## Objective

Improve the day-to-day experience of using Markplane from the command line. The CLI works but has friction points — there's no `edit` command to open items in `$EDITOR`, and the deprecated `serde_yaml` dependency should be swapped before it becomes a security liability. These are quality-of-life improvements that make Markplane feel polished.

## Key Results

- [ ] Users can open any item in their editor with `markplane edit TASK-001`
- [ ] `serde_yaml` replaced with `serde_yaml_ng` (maintained fork, identical API)
- [ ] All existing tests continue to pass after dependency swap

## Notes

Both items in this epic are small, independent tasks. The `edit` command is the more user-facing improvement; the serde_yaml swap is invisible maintenance. Neither has external dependencies or blocks other work.
