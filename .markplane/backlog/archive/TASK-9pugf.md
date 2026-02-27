---
id: TASK-9pugf
title: Add markplane edit command to open items in editor
status: done
priority: medium
type: feature
effort: small
tags:
- cli
epic: EPIC-6zdf4
plan: null
depends_on: []
blocks: []
assignee: null
position: Zy
created: 2026-02-10
updated: 2026-02-26
---

# Add markplane edit command to open items in editor

## Description

After creating an item with `markplane add`, users must open the markdown file directly in their editor to fill in content. There's no `markplane edit TASK-eduur` command that resolves the ID to a file path and opens it in `$EDITOR`. This is a basic quality-of-life feature that every file-based tool provides (e.g., `git commit` opens `$EDITOR`, `kubectl edit` opens a resource in `$EDITOR`).

## Acceptance Criteria

- [ ] New `edit` subcommand added to CLI: `markplane edit TASK-eduur`
- [ ] Resolves item ID to file path via `project.item_path()`
- [ ] Opens the file in `$EDITOR` (falls back to `$VISUAL`, then `vi`)
- [ ] Works for all entity types (TASK, EPIC, PLAN, NOTE)
- [ ] Prints an error if the item doesn't exist

## Notes

Pure convenience command — resolves ID to file path and opens `$EDITOR`. No post-edit processing (no timestamp bumping). Consistent with direct file editing being first-class. Consider using `std::process::Command` to spawn the editor. The `$EDITOR` / `$VISUAL` fallback chain is a Unix convention worth following.
