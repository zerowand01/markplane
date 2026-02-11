---
id: BACK-012
title: Add domain-focused context generation
status: backlog
priority: medium
type: feature
effort: medium
tags:
- context
- ai-integration
epic: EPIC-004
plan: null
depends_on: []
blocks: []
assignee: null
created: 2026-02-10
updated: 2026-02-10
---

# Add domain-focused context generation

## Description

The current context generation is project-wide — it summarizes everything. When working on a specific area of the codebase (e.g., the MCP server), most of that context is noise. Domain-focused context generation would filter context by tag, epic, or custom grouping, producing a summary relevant only to the area you're working in.

For example, `markplane context --tag mcp` would produce a summary of only MCP-related items, their statuses, blockers, and plans — exactly what an LLM needs when working on MCP features.

## Acceptance Criteria

- [ ] `markplane context --tag <tag>` generates context filtered to items with that tag
- [ ] `markplane context --epic EPIC-001` generates context scoped to a single epic and its items
- [ ] Filtered context includes relevant metrics (only for the filtered set)
- [ ] Filtered context includes cross-epic dependencies that affect the filtered items
- [ ] MCP resource supports domain-filtered context (e.g., `markplane://context/tag/mcp`)
- [ ] Multiple filters can be combined (e.g., `--tag mcp --status in-progress`)

## Notes

This builds on the existing `QueryFilter` infrastructure used by `markplane ls`. The context generation functions in `context.rs` already accept filtered item lists — the main work is adding CLI flags, wiring them through, and ensuring the output is coherent when scoped (e.g., blocked-items should only show blockers relevant to the filtered domain, not all blocked items project-wide).
