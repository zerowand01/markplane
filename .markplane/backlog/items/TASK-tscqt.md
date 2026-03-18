---
id: TASK-tscqt
title: Surface user-configured documentation_paths in web UI docs page
status: draft
priority: low
type: feature
effort: medium
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- web-ui
- docs
position: a6
created: 2026-03-18
updated: 2026-03-18
---

# Surface user-configured documentation_paths in web UI docs page

## Description

The web UI docs page currently only shows Markplane's built-in reference docs (embedded in the binary). Users can configure `documentation_paths` in `markplane.yaml` to point at their own project docs — these are already surfaced in INDEX.md and `.context/summary.md` via `list_documentation_files()` in core, but the web UI ignores them.

The docs page should also display the user's configured documentation alongside the built-in reference docs, with clear visual separation (e.g. "Reference" vs "Project Docs" sections).

## Acceptance Criteria

- [ ] `/api/docs` returns user-configured docs from `documentation_paths` alongside embedded reference docs
- [ ] `/api/docs/{slug}` can serve user-configured doc content from the filesystem
- [ ] Docs page visually distinguishes reference docs from project docs
- [ ] File watcher triggers `doc_changed` events for user-configured doc files
- [ ] Works correctly when `documentation_paths` is empty or unset (only reference docs shown)

## Notes

- Core already has `Project::list_documentation_files()` which handles path resolution and traversal protection
- Slug generation for user docs needs to avoid collisions with reference doc slugs
- Title extraction could parse the first `# heading` from each markdown file

## References
