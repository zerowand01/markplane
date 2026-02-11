---
id: BACK-011
title: Implement rich context bundles for individual items
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

# Implement rich context bundles for individual items

## Description

The `markplane context` command currently generates project-wide summaries (summary.md, active-work.md, blocked-items.md, metrics.md). There is no way to generate a focused context bundle for a single item — gathering the item itself, its parent epic, linked plan, dependencies, blockers, and related notes into one coherent document. This is exactly what an LLM needs when starting work on a specific task: all relevant context in one place, without loading the entire project.

The design spec describes `markplane context --item BACK-042` as a key feature for AI-assisted development workflows.

## Acceptance Criteria

- [ ] `markplane context --item BACK-042` generates a focused context bundle
- [ ] Bundle includes: the item's full content, parent epic summary, linked plan (if any), items it depends on, items it blocks
- [ ] Bundle includes cross-referenced items found via `[[ID]]` syntax in the body
- [ ] Output is a single markdown document optimized for LLM consumption (~2000 tokens target)
- [ ] Works for all entity types (passing an epic ID shows the epic + all its backlog items)
- [ ] MCP resource or tool exposes item context bundles
- [ ] Context bundle respects the 2000-token-per-file design constraint

## Notes

The existing `extract_references()` function in core already parses `[[ID]]` references from markdown bodies — use this to discover related items. The challenge is building the dependency graph without circular references (A depends on B depends on A). Consider a depth limit (e.g., 1 level of transitive dependencies). Output format should be markdown with clear section headers so the LLM can parse the structure.
