---
id: BACK-007
title: Add markplane edit command to open items in editor
status: backlog
priority: medium
type: feature
effort: small
tags:
- cli
epic: EPIC-002
plan: null
depends_on: []
blocks: []
assignee: null
created: 2026-02-10
updated: 2026-02-10
---

# Add markplane edit command to open items in editor

## Description

After creating an item with `markplane add`, users must open the markdown file directly in their editor to fill in content. There's no `markplane edit BACK-001` command that resolves the ID to a file path and opens it in `$EDITOR`. This is a basic quality-of-life feature that every file-based tool provides (e.g., `git commit` opens `$EDITOR`, `kubectl edit` opens a resource in `$EDITOR`).

## Acceptance Criteria

- [ ] New `edit` subcommand added to CLI: `markplane edit BACK-001`
- [ ] Resolves item ID to file path via `project.item_path()`
- [ ] Opens the file in `$EDITOR` (falls back to `$VISUAL`, then `vi`)
- [ ] Works for all entity types (BACK, EPIC, PLAN, NOTE)
- [ ] Updates the `updated` date in frontmatter after the editor closes (if file was modified)
- [ ] Prints an error if the item doesn't exist

## Notes

Consider using `std::process::Command` to spawn the editor. Check file modification time before/after to decide whether to update the `updated` timestamp. The `$EDITOR` / `$VISUAL` fallback chain is a Unix convention worth following.
