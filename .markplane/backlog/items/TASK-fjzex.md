---
id: TASK-fjzex
title: Rewrite matching body H1 when a title is updated
status: backlog
priority: medium
type: enhancement
effort: small
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- core
- dx
position: a6
created: 2026-09-30
updated: 2026-09-30
---

# Rewrite matching body H1 when a title is updated

## Description

Templates render the title twice: once in frontmatter (`title:`) and once as the
body's `# {TITLE}` heading. `update --title` (CLI, MCP and web) changes only the
frontmatter, so the H1 silently goes stale.

When a title is updated, rewrite the body's first H1 **only if it exactly
matches the old title**. If the H1 has been customised, leave it alone — the body
is human-authored content.

## Acceptance Criteria

- [ ] Updating a title rewrites the first `# ` heading when it equals the old title
- [ ] An H1 that differs from the old title is left untouched
- [ ] Bodies without an H1 are left untouched
- [ ] Applies to all entity types through the shared update path, so CLI, MCP and web all get it
- [ ] MCP guidance / docs state the rule: the H1 follows the title unless edited by hand
- [ ] Unit tests for the three cases above

## Notes

- From external agent feedback on markplane friction.
- Keep the rewrite inside the same locked read-modify-write as the title change.

## References
