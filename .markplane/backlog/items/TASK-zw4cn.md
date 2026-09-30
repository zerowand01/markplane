---
id: TASK-zw4cn
title: Add --json output to add, show and ls
status: backlog
priority: low
type: enhancement
effort: medium
epic: null
plan: null
depends_on: []
blocks: []
related: []
assignee: null
tags:
- cli
- dx
position: a9
created: 2026-09-30
updated: 2026-09-30
---

# Add --json output to add, show and ls

## Description

CLI output is human-formatted (tables, `Created TASK-xxxxx — title`), so scripts
and agents without MCP have to scrape it. One agent regexed the new ID out of
the `add` output.

Add a `--json` flag to `add`, `show` and `ls`, emitting the same shapes the MCP
tools already return (`markplane_add`, `markplane_show`, `markplane_query`) so
the two surfaces stay consistent.

## Acceptance Criteria

- [ ] `markplane add --json` prints `{"id": ..., "title": ...}`
- [ ] `markplane show --json` prints frontmatter and body
- [ ] `markplane ls --json` prints an array matching `markplane_query` output
- [ ] JSON shapes are shared with the MCP handlers rather than duplicated
- [ ] No colour codes or table formatting in JSON mode
- [ ] CLI reference docs updated; integration tests added

## Notes

- Start with these three commands; extend to `check` and others only if asked for.
- Agents on MCP already get JSON from `markplane_query`, so this is mainly for scripting.

## References
